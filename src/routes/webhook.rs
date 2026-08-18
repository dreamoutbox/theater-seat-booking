use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde_json::Value;
use uuid::Uuid;

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
    // Check payment_events table for incoming stripe_event_id
    let already_processed: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM payment_events WHERE stripe_event_id = $1)",
    )
    .bind(&event_id)
    .fetch_one(&state.pool)
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
    let order_id: Option<Uuid> = if !payment_intent_id.is_empty() {
        sqlx::query_scalar(
            "SELECT id FROM orders WHERE stripe_payment_intent_id = $1 FOR UPDATE",
        )
        .bind(payment_intent_id)
        .fetch_optional(&mut *tx)
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
    sqlx::query(
        "INSERT INTO payment_events (stripe_event_id, order_id, type)
         VALUES ($1, $2, $3)",
    )
    .bind(&event_id)
    .bind(order_id)
    .bind(&event_type)
    .execute(&mut *tx)
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
                sqlx::query("UPDATE orders SET status = 'paid' WHERE id = $1")
                    .bind(oid)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({ "error": e.to_string() })),
                        )
                    })?;

                sqlx::query(
                    "UPDATE seat_bookings
                     SET status = 'confirmed'
                     WHERE id IN (SELECT seat_booking_id FROM order_seats WHERE order_id = $1)",
                )
                .bind(oid)
                .execute(&mut *tx)
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
                sqlx::query("UPDATE orders SET status = 'cancelled' WHERE id = $1")
                    .bind(oid)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({ "error": e.to_string() })),
                        )
                    })?;

                sqlx::query(
                    "UPDATE seat_bookings
                     SET status = 'released'
                     WHERE id IN (SELECT seat_booking_id FROM order_seats WHERE order_id = $1)",
                )
                .bind(oid)
                .execute(&mut *tx)
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
