use axum::{Json, extract::State, http::StatusCode};
use repository::{OrderRepository, SeatBookingRepository};
use serde_json::Value;

use crate::routes::AppState;

pub async fn stripe_webhook(
    State(state): State<AppState>,
    Json(event): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let event_id = event
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    let event_type = event
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    if event_id.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Missing event id" })),
        ));
    }

    tracing::info!(
        stripe_event_id = %event_id,
        event_type = %event_type,
        "Received Stripe webhook event"
    );

    // 1. Idempotency Check (§12 locked decision):
    let already_processed = OrderRepository::exists_payment_event_by_stripe_event_id(&state.pool, &event_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    if already_processed {
        tracing::info!(
            stripe_event_id = %event_id,
            "Event already processed, returning 200 immediately"
        );
        return Ok(Json(serde_json::json!({ "status": "already_processed" })));
    }

    // Extract payment intent id from the event object
    let payment_intent_id = event
        .get("data")
        .and_then(|d| d.get("object"))
        .and_then(|o| o.get("id"))
        .and_then(|id| id.as_str())
        .unwrap_or_default();

    let mut tx = state.pool.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    // Find the associated order
    let order_id = if !payment_intent_id.is_empty() {
        OrderRepository::find_id_by_payment_intent_for_update(&mut *tx, payment_intent_id)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
            })?
    } else {
        None
    };

    // 2. Insert event row in the same transaction as state updates (§12)
    OrderRepository::save_payment_event(&mut *tx, &event_id, order_id, &event_type)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    // 3. Process status transition based on event type
    match event_type.as_str() {
        "payment_intent.succeeded" => {
            if let Some(oid) = order_id {
                OrderRepository::update_status(&mut *tx, oid, "paid")
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({ "error": e.to_string() })),
                        )
                    })?;

                SeatBookingRepository::update_status_by_order_id(&mut *tx, oid, "confirmed")
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({ "error": e.to_string() })),
                        )
                    })?;

                tracing::info!(order_id = %oid, "Payment confirmed; seats confirmed.");
            }
        }
        "payment_intent.payment_failed" | "payment_intent.canceled" => {
            if let Some(oid) = order_id {
                OrderRepository::update_status(&mut *tx, oid, "cancelled")
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({ "error": e.to_string() })),
                        )
                    })?;

                SeatBookingRepository::update_status_by_order_id(&mut *tx, oid, "released")
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({ "error": e.to_string() })),
                        )
                    })?;

                tracing::info!(order_id = %oid, "Payment failed/canceled; seats released.");
            }
        }
        _ => {
            tracing::debug!(event_type = %event_type, "Unhandled webhook event type");
        }
    }

    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    Ok(Json(serde_json::json!({ "status": "processed" })))
}
