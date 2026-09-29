use axum::{Json, extract::State, http::StatusCode};
use repository::{OrderRepository, SeatBookingRepository, SequenceRepository, TicketRepository};
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
    let already_processed =
        OrderRepository::exists_payment_event_by_stripe_event_id(&state.pool, &event_id)
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

    // For checkout.session.completed, the payment_intent is nested inside the session object
    let data_object = event.get("data").and_then(|d| d.get("object"));

    // Primary: extract order_id from event metadata (set by us on both PI and checkout session).
    // This is reliable regardless of which PaymentIntent Stripe creates internally.
    let metadata_order_id: Option<uuid::Uuid> = data_object
        .and_then(|o| o.get("metadata"))
        .and_then(|m| m.get("order_id"))
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok());

    // Fallback: resolve via PI ID lookup (covers old orders created before this change).
    let pi_id_for_lookup = if event_type == "checkout.session.completed" {
        data_object
            .and_then(|o| o.get("payment_intent"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
    } else {
        data_object
            .and_then(|o| o.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
    };

    let mut tx = state.pool.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    // Find the associated order: prefer metadata lookup, fall back to PI ID lookup
    let order_id: Option<uuid::Uuid> = if let Some(oid) = metadata_order_id {
        tracing::debug!(order_id = %oid, "Resolved order from event metadata");
        Some(oid)
    } else if !pi_id_for_lookup.is_empty() {
        OrderRepository::find_id_by_payment_intent_for_update(&mut *tx, pi_id_for_lookup)
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
        "payment_intent.succeeded" | "checkout.session.completed" => {
            if let Some(oid) = order_id {
                // Guard: check current status to handle both events firing for the same order
                let current_status = OrderRepository::find_status_by_id_for_update(&mut *tx, oid)
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({ "error": e.to_string() })),
                        )
                    })?;

                if current_status.as_deref() == Some("pending_payment") {
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

                    // Assign sequential Thai tax invoice running number (INV-YYYYMM-XXXXX)
                    if let Ok(Some(ev_id)) =
                        OrderRepository::find_event_id_by_order_id(&mut *tx, oid).await
                    {
                        let period = chrono::Utc::now().format("%Y%m").to_string();
                        if let Ok(tax_invoice_number) = SequenceRepository::next_running_number(
                            &mut *tx, ev_id, "order", "INV", &period,
                        )
                        .await
                        {
                            let _ = OrderRepository::update_tax_invoice_number(
                                &mut *tx,
                                oid,
                                &tax_invoice_number,
                            )
                            .await;
                        }
                    }

                    TicketRepository::create_tickets_for_paid_order(&mut *tx, oid)
                        .await
                        .map_err(|e| {
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(serde_json::json!({ "error": format!("Failed to generate tickets: {e}") })),
                            )
                        })?;

                    tracing::info!(order_id = %oid, event_type = %event_type, "Payment confirmed; seats confirmed and tickets created.");
                } else {
                    tracing::info!(order_id = %oid, event_type = %event_type, status = ?current_status, "Order already processed, skipping confirm.");
                }
            } else {
                tracing::warn!(event_type = %event_type, "No order found for payment event; skipping.");
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

    if event_type == "payment_intent.succeeded" || event_type == "checkout.session.completed" {
        if let Some(oid) = order_id {
            let post_pay_state = state.clone();
            tokio::spawn(async move {
                match crate::routes::order::generate_and_upload_order_pdf(&post_pay_state, oid)
                    .await
                {
                    Ok(pdf_bytes) => {
                        if let Err(e) = post_pay_state
                            .email
                            .send_order_confirmation(&post_pay_state.pool, oid, pdf_bytes)
                            .await
                        {
                            tracing::warn!(
                                "Failed to send order confirmation email via webhook: {e}"
                            );
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to generate/upload order receipt PDF via webhook: {e}"
                        );
                    }
                }
            });
        }
    }

    Ok(Json(serde_json::json!({ "status": "processed" })))
}
