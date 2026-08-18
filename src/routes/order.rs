use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::AuthenticatedUser,
    models::{OrderDetail, OrderSeatDetail, UserRole},
    routes::AppState,
};

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
    // Must belong to auth_user, status = 'held', expires_at > now()
    #[derive(sqlx::FromRow)]
    struct HoldItem {
        id: Uuid,
        showtime_id: Uuid,
        price_cents: i64,
    }

    let holds: Vec<HoldItem> = sqlx::query_as(
        "SELECT sb.id, sb.showtime_id, z.price_cents
         FROM seat_bookings sb
         JOIN seats s ON s.id = sb.seat_id
         JOIN zones z ON z.id = s.zone_id
         WHERE (sb.id = ANY($1) OR sb.seat_id = ANY($1))
           AND sb.user_id = $2 
           AND sb.status = 'held' 
           AND sb.expires_at > now()
         FOR UPDATE OF sb",
    )
    .bind(&payload.seat_booking_ids)
    .bind(auth_user.id)
    .fetch_all(&mut *tx)
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
    sqlx::query(
        "INSERT INTO orders (id, user_id, showtime_id, total_amount_cents, status, stripe_payment_intent_id)
         VALUES ($1, $2, $3, $4, 'pending_payment', $5)",
    )
    .bind(order_id)
    .bind(auth_user.id)
    .bind(first_showtime_id)
    .bind(total_amount_cents)
    .bind(&stripe_res.id)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    // 6. Insert order_seats
    for hold in holds {
        sqlx::query(
            "INSERT INTO order_seats (order_id, seat_booking_id, price_cents)
             VALUES ($1, $2, $3)",
        )
        .bind(order_id)
        .bind(hold.id)
        .bind(hold.price_cents)
        .execute(&mut *tx)
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

    #[derive(sqlx::FromRow)]
    struct OrderRow {
        id: Uuid,
        user_id: Uuid,
        user_email: Option<String>,
        showtime_id: Uuid,
        event_title: String,
        theater_name: String,
        starts_at: chrono::DateTime<chrono::Utc>,
        total_amount_cents: i64,
        status: String,
        stripe_payment_intent_id: Option<String>,
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let orders_query = if is_elevated {
        "SELECT o.id, o.user_id, u.email as user_email, o.showtime_id,
                e.title as event_title, t.name as theater_name, s.starts_at,
                o.total_amount_cents, o.status, o.stripe_payment_intent_id, o.created_at
         FROM orders o
         JOIN users u ON u.id = o.user_id
         JOIN showtimes s ON s.id = o.showtime_id
         JOIN events e ON e.id = s.event_id
         JOIN theaters t ON t.id = s.theater_id
         ORDER BY o.created_at DESC"
    } else {
        "SELECT o.id, o.user_id, u.email as user_email, o.showtime_id,
                e.title as event_title, t.name as theater_name, s.starts_at,
                o.total_amount_cents, o.status, o.stripe_payment_intent_id, o.created_at
         FROM orders o
         JOIN users u ON u.id = o.user_id
         JOIN showtimes s ON s.id = o.showtime_id
         JOIN events e ON e.id = s.event_id
         JOIN theaters t ON t.id = s.theater_id
         WHERE o.user_id = $1
         ORDER BY o.created_at DESC"
    };

    let order_rows: Vec<OrderRow> = if is_elevated {
        sqlx::query_as(orders_query)
            .fetch_all(&state.pool)
            .await
    } else {
        sqlx::query_as(orders_query)
            .bind(auth_user.id)
            .fetch_all(&state.pool)
            .await
    }
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let mut result = Vec::with_capacity(order_rows.len());

    for o in order_rows {
        let seats: Vec<OrderSeatDetail> = sqlx::query_as(
            "SELECT os.seat_booking_id, s.id as seat_id, s.row_label, s.seat_number, z.name as zone_name, os.price_cents
             FROM order_seats os
             JOIN seat_bookings sb ON sb.id = os.seat_booking_id
             JOIN seats s ON s.id = sb.seat_id
             JOIN zones z ON z.id = s.zone_id
             WHERE os.order_id = $1
             ORDER BY s.row_label ASC, s.seat_number ASC",
        )
        .bind(o.id)
        .fetch_all(&state.pool)
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
    #[derive(sqlx::FromRow)]
    struct OrderRow {
        id: Uuid,
        user_id: Uuid,
        user_email: Option<String>,
        showtime_id: Uuid,
        event_title: String,
        theater_name: String,
        starts_at: chrono::DateTime<chrono::Utc>,
        total_amount_cents: i64,
        status: String,
        stripe_payment_intent_id: Option<String>,
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let order_opt: Option<OrderRow> = sqlx::query_as(
        "SELECT o.id, o.user_id, u.email as user_email, o.showtime_id,
                e.title as event_title, t.name as theater_name, s.starts_at,
                o.total_amount_cents, o.status, o.stripe_payment_intent_id, o.created_at
         FROM orders o
         JOIN users u ON u.id = o.user_id
         JOIN showtimes s ON s.id = o.showtime_id
         JOIN events e ON e.id = s.event_id
         JOIN theaters t ON t.id = s.theater_id
         WHERE o.id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
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

    let seats: Vec<OrderSeatDetail> = sqlx::query_as(
        "SELECT os.seat_booking_id, s.id as seat_id, s.row_label, s.seat_number, z.name as zone_name, os.price_cents
         FROM order_seats os
         JOIN seat_bookings sb ON sb.id = os.seat_booking_id
         JOIN seats s ON s.id = sb.seat_id
         JOIN zones z ON z.id = s.zone_id
         WHERE os.order_id = $1
         ORDER BY s.row_label ASC, s.seat_number ASC",
    )
    .bind(o.id)
    .fetch_all(&state.pool)
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
    let order_status: Option<String> =
        sqlx::query_scalar("SELECT status FROM orders WHERE id = $1 AND user_id = $2 FOR UPDATE")
            .bind(order_id)
            .bind(auth_user.id)
            .fetch_optional(&mut *tx)
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
            Json(serde_json::json!({ "error": format!("Order cannot be paid (status: {status})") })),
        ));
    }

    // 2. Mark order as paid
    sqlx::query("UPDATE orders SET status = 'paid' WHERE id = $1")
        .bind(order_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    // 3. Mark held seats as confirmed
    sqlx::query(
        "UPDATE seat_bookings
         SET status = 'confirmed'
         WHERE id IN (SELECT seat_booking_id FROM order_seats WHERE order_id = $1)",
    )
    .bind(order_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    // 4. Log mock payment event
    let mock_event_id = format!("evt_mock_{}", Uuid::new_v4());
    sqlx::query(
        "INSERT INTO payment_events (stripe_event_id, order_id, type)
         VALUES ($1, $2, 'payment_intent.succeeded')",
    )
    .bind(&mock_event_id)
    .bind(order_id)
    .execute(&mut *tx)
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
