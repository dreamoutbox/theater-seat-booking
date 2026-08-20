use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use repository::{
    OrderDetail, OrderRepository, RefundDetail, RefundRepository, SeatBookingRepository,
    SeatChangeRepository, SeatRepository, TicketRepository, TicketSeatChangeDetail, UserRole,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::AuthenticatedUser, routes::AppState};

#[derive(Debug, Deserialize)]
pub struct CreateOrderSeatItem {
    pub seat_booking_id: Uuid,
    pub attendee_name: Option<String>,
    pub attendee_phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub seat_booking_ids: Option<Vec<Uuid>>,
    pub seats: Option<Vec<CreateOrderSeatItem>>,
    pub billing_name: Option<String>,
    pub tax_id: Option<String>,
    pub billing_address: Option<String>,
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
    let seat_booking_ids: Vec<Uuid> = if let Some(ref seats) = payload.seats {
        seats.iter().map(|s| s.seat_booking_id).collect()
    } else if let Some(ref ids) = payload.seat_booking_ids {
        ids.clone()
    } else {
        Vec::new()
    };

    if seat_booking_ids.is_empty() {
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
        &seat_booking_ids,
        auth_user.id,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    if holds.len() != seat_booking_ids.len() {
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
        payload.billing_name.as_deref(),
        payload.tax_id.as_deref(),
        payload.billing_address.as_deref(),
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    // 6. Insert order_seats
    for hold in &holds {
        let (att_name, att_phone) = if let Some(ref seats) = payload.seats {
            seats
                .iter()
                .find(|s| s.seat_booking_id == hold.id)
                .map(|s| (s.attendee_name.as_deref(), s.attendee_phone.as_deref()))
                .unwrap_or((None, None))
        } else {
            (None, None)
        };

        OrderRepository::save_order_seat(
            &mut *tx,
            order_id,
            hold.id,
            hold.price_cents,
            att_name,
            att_phone,
        )
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
            event_id: o.event_id,
            event_title: o.event_title,
            theater_name: o.theater_name,
            starts_at: o.starts_at,
            total_amount_cents: o.total_amount_cents,
            refunded_amount_cents: o.refunded_amount_cents,
            status: o.status,
            stripe_payment_intent_id: o.stripe_payment_intent_id,
            stripe_client_secret: None,
            billing_name: o.billing_name,
            tax_id: o.tax_id,
            billing_address: o.billing_address,
            organizer_tax_business_name: o.organizer_tax_business_name,
            organizer_tax_id: o.organizer_tax_id,
            organizer_tax_address: o.organizer_tax_address,
            organizer_tax_email: o.organizer_tax_email,
            organizer_tax_phone: o.organizer_tax_phone,
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
        event_id: o.event_id,
        event_title: o.event_title,
        theater_name: o.theater_name,
        starts_at: o.starts_at,
        total_amount_cents: o.total_amount_cents,
        refunded_amount_cents: o.refunded_amount_cents,
        status: o.status,
        stripe_payment_intent_id: o.stripe_payment_intent_id,
        stripe_client_secret: None,
        billing_name: o.billing_name,
        tax_id: o.tax_id,
        billing_address: o.billing_address,
        organizer_tax_business_name: o.organizer_tax_business_name,
        organizer_tax_id: o.organizer_tax_id,
        organizer_tax_address: o.organizer_tax_address,
        organizer_tax_email: o.organizer_tax_email,
        organizer_tax_phone: o.organizer_tax_phone,
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

pub async fn get_order_refunds(
    Path(id): Path<Uuid>,
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<RefundDetail>>, (StatusCode, Json<serde_json::Value>)> {
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

    let refunds = RefundRepository::find_by_order_id(&state.pool, id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(refunds))
}

#[derive(Debug, Deserialize)]
pub struct ChangeSeatRequest {
    pub old_seat_booking_id: Uuid,
    pub new_seat_id: Uuid,
    pub reason: Option<String>,
}

pub async fn change_seat(
    Path(order_id): Path<Uuid>,
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<ChangeSeatRequest>,
) -> Result<Json<OrderDetail>, (StatusCode, Json<serde_json::Value>)> {
    let mut tx = state.pool.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    // 1. Fetch order info & lock
    let order_info = OrderRepository::find_order_with_event_owner_for_update(&mut *tx, order_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    let (order_user_id, status, _, event_owner_id, _, _) = match order_info {
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
    let is_owner = auth_user.id == order_user_id;
    let is_admin_or_staff = auth_user.role == UserRole::Admin || auth_user.role == UserRole::Staff;
    let is_event_organizer = auth_user.role == UserRole::EventOrganizer && event_owner_id == Some(auth_user.id);

    if !is_owner && !is_admin_or_staff && !is_event_organizer {
        tx.rollback().await.ok();
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Unauthorized to modify this order" })),
        ));
    }

    // 3. Status check
    if status != "paid" && status != "partially_refunded" {
        tx.rollback().await.ok();
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": format!("Cannot change seats on order with status '{status}'") })),
        ));
    }

    // 4. Verify old seat booking exists and is confirmed
    let old_booking = SeatBookingRepository::find_by_id(&mut *tx, payload.old_seat_booking_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    let booking = match old_booking {
        Some(b) if b.status == "confirmed" => b,
        _ => {
            tx.rollback().await.ok();
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "The specified seat booking is not an active confirmed booking" })),
            ));
        }
    };

    // 5. Check if new_seat_id is taken
    let is_taken = SeatBookingRepository::check_seat_is_taken_for_update(&mut *tx, booking.showtime_id, payload.new_seat_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    if is_taken {
        tx.rollback().await.ok();
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "The requested new seat is already booked or held by another customer" })),
        ));
    }

    // 6. Fetch new seat details
    let new_seat = SeatRepository::find_seat_with_zone_by_id(&mut *tx, payload.new_seat_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Target new seat was not found in theater layout" })),
            )
        })?;

    // 7. Fetch old seat details
    let old_seat_opt = SeatRepository::find_seat_with_zone_by_id(&mut *tx, booking.seat_id)
        .await
        .unwrap_or(None);

    let (old_row_label, old_seat_number, old_zone_name, old_price_cents) = if let Some(os) = old_seat_opt {
        (os.row_label, os.seat_number, os.zone_name, os.price_cents)
    } else {
        let os_list = OrderRepository::find_order_seats_by_order_id(&mut *tx, order_id).await.unwrap_or_default();
        let matched = os_list.into_iter().find(|s| s.seat_booking_id == payload.old_seat_booking_id);
        if let Some(m) = matched {
            (m.row_label, m.seat_number, m.zone_name, m.price_cents)
        } else {
            ("—".to_string(), 0, "Standard".to_string(), 0)
        }
    };

    // 8. Find associated ticket if one exists
    let ticket_opt = TicketRepository::find_by_seat_booking_id(&mut *tx, payload.old_seat_booking_id)
        .await
        .unwrap_or(None);

    // 9. Update seat_id on seat_booking
    SeatBookingRepository::update_seat_id(&mut *tx, payload.old_seat_booking_id, payload.new_seat_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to update seat booking: {e}") })),
            )
        })?;

    // 10. Update snapshot on order_seats
    OrderRepository::update_order_seat_full(
        &mut *tx,
        order_id,
        payload.old_seat_booking_id,
        &new_seat.row_label,
        new_seat.seat_number,
        &new_seat.zone_name,
        new_seat.price_cents,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to update order seat record: {e}") })),
        )
    })?;

    // 11. Record seat change log entry
    let change_log_id = Uuid::new_v4();
    let metadata = serde_json::json!({
        "old_zone_name": old_zone_name,
        "new_zone_name": new_seat.zone_name,
        "price_difference_cents": new_seat.price_cents - old_price_cents,
        "changed_by_role": auth_user.role.as_str(),
        "ticket_code": ticket_opt.as_ref().map(|t| t.code.clone()),
    });

    SeatChangeRepository::save(
        &mut *tx,
        change_log_id,
        order_id,
        ticket_opt.map(|t| t.id),
        Some(payload.old_seat_booking_id),
        booking.showtime_id,
        Some(booking.seat_id),
        Some(payload.new_seat_id),
        &old_row_label,
        old_seat_number,
        &old_zone_name,
        old_price_cents,
        &new_seat.row_label,
        new_seat.seat_number,
        &new_seat.zone_name,
        new_seat.price_cents,
        Some(auth_user.id),
        payload.reason.as_deref(),
        Some(metadata),
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to record seat change log: {e}") })),
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
        seat_booking_id = %payload.old_seat_booking_id,
        new_seat_id = %payload.new_seat_id,
        "Seat changed successfully"
    );

    // Fetch and return fresh order details
    let order_detail = OrderRepository::find_order_detail_by_id(&state.pool, order_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Order not found after update" })),
            )
        })?;

    let seats = OrderRepository::find_order_seats_by_order_id(&state.pool, order_id)
        .await
        .unwrap_or_default();

    Ok(Json(OrderDetail {
        id: order_detail.id,
        user_id: order_detail.user_id,
        user_email: order_detail.user_email,
        showtime_id: order_detail.showtime_id,
        event_id: order_detail.event_id,
        event_title: order_detail.event_title,
        theater_name: order_detail.theater_name,
        starts_at: order_detail.starts_at,
        total_amount_cents: order_detail.total_amount_cents,
        refunded_amount_cents: order_detail.refunded_amount_cents,
        status: order_detail.status,
        stripe_payment_intent_id: order_detail.stripe_payment_intent_id,
        stripe_client_secret: None,
        billing_name: order_detail.billing_name,
        tax_id: order_detail.tax_id,
        billing_address: order_detail.billing_address,
        organizer_tax_business_name: order_detail.organizer_tax_business_name,
        organizer_tax_id: order_detail.organizer_tax_id,
        organizer_tax_address: order_detail.organizer_tax_address,
        organizer_tax_email: order_detail.organizer_tax_email,
        organizer_tax_phone: order_detail.organizer_tax_phone,
        created_at: order_detail.created_at,
        seats,
    }))
}

pub async fn get_order_seat_changes(
    Path(id): Path<Uuid>,
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<TicketSeatChangeDetail>>, (StatusCode, Json<serde_json::Value>)> {
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

    let changes = SeatChangeRepository::find_by_order_id(&state.pool, id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(changes))
}
