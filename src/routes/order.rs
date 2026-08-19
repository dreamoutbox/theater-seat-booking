use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use repository::{
    OrderDetail, OrderRepository, RefundRepository, SeatBookingRepository, TicketRepository,
    UserRole,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::AuthenticatedUser, routes::AppState};

#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub seat_booking_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct CreateOrderResponse {
    pub order_id: Uuid,
    pub total_amount_cents: i64,
    pub stripe_payment_intent_id: String,
    pub stripe_client_secret: String,
}

pub async fn create_order(
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateOrderRequest>,
) -> Result<(StatusCode, Json<CreateOrderResponse>), (StatusCode, Json<serde_json::Value>)> {
    if payload.seat_booking_ids.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "At least one seat hold must be selected" })),
        ));
    }

    let mut tx = state.pool.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    // 1. Fetch and validate all requested seat_bookings:
    let holds = SeatBookingRepository::find_active_holds_by_user_and_seat_ids(
        &mut *tx,
        &payload.seat_booking_ids,
        auth_user.id,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    if holds.len() != payload.seat_booking_ids.len() {
        tx.rollback().await.ok();
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "One or more selected seat holds are expired, invalid, or do not belong to you"
            })),
        ));
    }

    // 2. Validate one order = one showtime (§12 locked decision)
    let first_showtime_id = holds[0].showtime_id;
    let cross_showtime = holds.iter().any(|h| h.showtime_id != first_showtime_id);
    if cross_showtime {
        tx.rollback().await.ok();
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "All seats in an order must belong to the exact same showtime"
            })),
        ));
    }

    // 3. Compute total
    let total_amount_cents: i64 = holds.iter().map(|h| h.price_cents).sum();
    let order_id = Uuid::new_v4();

    // 4. Create Stripe PaymentIntent
    let stripe_res = state
        .stripe
        .create_payment_intent(total_amount_cents, "usd", &order_id.to_string())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Payment provider error: {e}") })),
            )
        })?;

    // 5. Insert order
    OrderRepository::save(
        &mut *tx,
        order_id,
        auth_user.id,
        first_showtime_id,
        total_amount_cents,
        Some(&stripe_res.id),
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    // 6. Insert order_seats
    for hold in holds {
        OrderRepository::save_order_seat(&mut *tx, order_id, hold.id, hold.price_cents)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
            })?;
    }

    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    tracing::info!(
        order_id = %order_id,
        user_id = %auth_user.id,
        amount_cents = total_amount_cents,
        "Order created successfully"
    );

    Ok((
        StatusCode::CREATED,
        Json(CreateOrderResponse {
            order_id,
            total_amount_cents,
            stripe_payment_intent_id: stripe_res.id,
            stripe_client_secret: stripe_res.client_secret,
        }),
    ))
}

pub async fn list_orders(
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<OrderDetail>>, (StatusCode, Json<serde_json::Value>)> {
    let is_elevated = auth_user.role == UserRole::Admin
        || auth_user.role == UserRole::Staff
        || auth_user.role == UserRole::EventOrganizer;

    let order_rows = if is_elevated {
        OrderRepository::find_all_order_details(&state.pool).await
    } else {
        OrderRepository::find_order_details_by_user_id(&state.pool, auth_user.id).await
    }
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let mut result = Vec::with_capacity(order_rows.len());

    for o in order_rows {
        let seats = OrderRepository::find_order_seats_by_order_id(&state.pool, o.id)
            .await
            .unwrap_or_default();

        result.push(OrderDetail {
            id: o.id,
            user_id: o.user_id,
            user_email: o.user_email,
            showtime_id: o.showtime_id,
            event_title: o.event_title,
            theater_name: o.theater_name,
            starts_at: o.starts_at,
            total_amount_cents: o.total_amount_cents,
            refunded_amount_cents: o.refunded_amount_cents,
            status: o.status,
            stripe_payment_intent_id: o.stripe_payment_intent_id,
            stripe_client_secret: None,
            created_at: o.created_at,
            seats,
        });
    }

    Ok(Json(result))
}

pub async fn get_order(
    Path(id): Path<Uuid>,
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<OrderDetail>, (StatusCode, Json<serde_json::Value>)> {
    let order_opt = OrderRepository::find_order_detail_by_id(&state.pool, id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    let o = match order_opt {
        Some(row) => row,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Order not found" })),
            ));
        }
    };

    let is_elevated = auth_user.role == UserRole::Admin
        || auth_user.role == UserRole::Staff
        || auth_user.role == UserRole::EventOrganizer;

    if !is_elevated && o.user_id != auth_user.id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Access denied to this order" })),
        ));
    }

    let seats = OrderRepository::find_order_seats_by_order_id(&state.pool, o.id)
        .await
        .unwrap_or_default();

    Ok(Json(OrderDetail {
        id: o.id,
        user_id: o.user_id,
        user_email: o.user_email,
        showtime_id: o.showtime_id,
        event_title: o.event_title,
        theater_name: o.theater_name,
        starts_at: o.starts_at,
        total_amount_cents: o.total_amount_cents,
        refunded_amount_cents: o.refunded_amount_cents,
        status: o.status,
        stripe_payment_intent_id: o.stripe_payment_intent_id,
        stripe_client_secret: None,
        created_at: o.created_at,
        seats,
    }))
}

// Mock confirmation endpoint for seamless manual testing & POC demo
pub async fn confirm_mock_order(
    Path(order_id): Path<Uuid>,
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut tx = state.pool.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    // 1. Fetch order
    let order_status = OrderRepository::find_status_by_id_and_user_for_update(
        &mut *tx,
        order_id,
        auth_user.id,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let status = match order_status {
        Some(s) => s,
        None => {
            tx.rollback().await.ok();
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Order not found" })),
            ));
        }
    };

    if status != "pending_payment" {
        tx.rollback().await.ok();
        return Err((
            StatusCode::BAD_REQUEST,
            Json(
                serde_json::json!({ "error": format!("Order cannot be paid (status: {status})") }),
            ),
        ));
    }

    // 2. Mark order as paid
    OrderRepository::update_status(&mut *tx, order_id, "paid")
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    // 3. Mark held seats as confirmed
    SeatBookingRepository::update_status_by_order_id(&mut *tx, order_id, "confirmed")
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    // 4. Create tickets for the confirmed order
    TicketRepository::create_tickets_for_paid_order(&mut *tx, order_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to generate tickets: {e}") })),
            )
        })?;

    // 5. Log mock payment event
    let mock_event_id = format!("evt_mock_{}", Uuid::new_v4());
    OrderRepository::save_payment_event(
        &mut *tx,
        &mock_event_id,
        Some(order_id),
        "payment_intent.succeeded",
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    tracing::info!(order_id = %order_id, "Order confirmed via mock payment");

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Payment confirmed successfully",
        "order_id": order_id
    })))
}

#[derive(Debug, Deserialize)]
pub struct RefundOrderRequest {
    pub ticket_ids: Vec<Uuid>,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RefundOrderResponse {
    pub refund_id: Uuid,
    pub order_id: Uuid,
    pub amount_cents: i64,
    pub stripe_refund_id: Option<String>,
    pub status: String,
    pub refunded_ticket_count: usize,
}

pub async fn refund_order(
    Path(order_id): Path<Uuid>,
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<RefundOrderRequest>,
) -> Result<Json<RefundOrderResponse>, (StatusCode, Json<serde_json::Value>)> {
    if payload.ticket_ids.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "At least one ticket must be selected for refund" })),
        ));
    }

    let mut tx = state.pool.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    // 1. Fetch order with event ownership info
    let order_info = OrderRepository::find_order_with_event_owner_for_update(&mut *tx, order_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    let (_, status, stripe_payment_intent_id, event_owner_id, _total_amount_cents, current_refunded_cents) = match order_info {
        Some(info) => info,
        None => {
            tx.rollback().await.ok();
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Order not found" })),
            ));
        }
    };

    // 2. Authorization check
    let is_admin_or_staff = auth_user.role == UserRole::Admin || auth_user.role == UserRole::Staff;
    let is_event_owner = auth_user.role == UserRole::EventOrganizer && event_owner_id == Some(auth_user.id);

    if !is_admin_or_staff && !is_event_owner {
        tx.rollback().await.ok();
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "You do not have permission to refund tickets for this order" })),
        ));
    }

    // 3. Status check: order must be paid or partially_refunded
    if status != "paid" && status != "partially_refunded" {
        tx.rollback().await.ok();
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": format!("Cannot refund order with status '{status}'") })),
        ));
    }

    // 4. Fetch & lock tickets to refund
    let tickets = TicketRepository::find_by_ids_and_order_id_for_update(&mut *tx, &payload.ticket_ids, order_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    if tickets.len() != payload.ticket_ids.len() {
        tx.rollback().await.ok();
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "One or more selected tickets do not belong to this order" })),
        ));
    }

    for t in &tickets {
        if t.status != "active" {
            tx.rollback().await.ok();
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": format!("Ticket for Row {}, Seat {} cannot be refunded because its status is '{}'", t.row_label, t.seat_number, t.status)
                })),
            ));
        }
    }

    // 5. Calculate total refund amount
    let refund_amount_cents: i64 = tickets.iter().map(|t| t.price_cents).sum();
    let seat_booking_ids: Vec<Uuid> = tickets.iter().map(|t| t.seat_booking_id).collect();

    // 6. Call Stripe refund if stripe_payment_intent_id exists
    let stripe_refund_id = match &stripe_payment_intent_id {
        Some(pi) if !pi.is_empty() => {
            let res = state.stripe.create_refund(pi, refund_amount_cents, payload.reason.as_deref())
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": format!("Payment gateway refund failed: {e}") })),
                    )
                })?;
            Some(res)
        }
        _ => None,
    };

    // 7. Save refund record
    let refund_id = Uuid::new_v4();
    RefundRepository::save(
        &mut *tx,
        refund_id,
        order_id,
        refund_amount_cents,
        stripe_refund_id.as_deref(),
        auth_user.id,
        payload.reason.as_deref(),
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to create refund record: {e}") })),
        )
    })?;

    // 8. Save refund tickets
    for t in &tickets {
        RefundRepository::save_refund_ticket(&mut *tx, refund_id, t.id, t.price_cents)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Failed to link refund ticket: {e}") })),
                )
            })?;
    }

    // 9. Mark tickets as refunded
    TicketRepository::mark_tickets_refunded(&mut *tx, &payload.ticket_ids, auth_user.id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to mark tickets as refunded: {e}") })),
            )
        })?;

    // 10. Release the seat bookings so seats are available on the seat map again
    SeatBookingRepository::release_by_ids(&mut *tx, &seat_booking_ids)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to release seat bookings: {e}") })),
            )
        })?;

    // 11. Check remaining active tickets to determine new order status
    let remaining_active = TicketRepository::count_active_tickets_by_order_id(&mut *tx, order_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    let new_status = if remaining_active == 0 {
        "refunded"
    } else {
        "partially_refunded"
    };

    let new_total_refunded = current_refunded_cents + refund_amount_cents;
    OrderRepository::update_refund_status(&mut *tx, order_id, new_status, new_total_refunded)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to update order status: {e}") })),
            )
        })?;

    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    tracing::info!(
        order_id = %order_id,
        refund_id = %refund_id,
        amount_cents = refund_amount_cents,
        new_status = new_status,
        "Order refunded successfully"
    );

    Ok(Json(RefundOrderResponse {
        refund_id,
        order_id,
        amount_cents: refund_amount_cents,
        stripe_refund_id,
        status: new_status.to_string(),
        refunded_ticket_count: tickets.len(),
    }))
}
