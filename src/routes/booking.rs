use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use repository::{
    BookingQueueRepository, EventRepository, SeatBookingRepository, SeatRepository,
    ShowtimeRepository,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::AuthenticatedUser, routes::AppState};

#[derive(Debug, Deserialize)]
pub struct EnqueueBookingRequest {
    pub showtime_id: Uuid,
    pub seat_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ReleaseHoldRequest {
    pub showtime_id: Uuid,
    pub seat_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct EnqueueBookingResponse {
    pub booking_uuid: Uuid,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct BookingStatusResponse {
    pub booking_uuid: Uuid,
    pub status: String,
    pub failure_reason: Option<String>,
    pub showtime_id: Uuid,
    pub seat_id: Uuid,
    pub zone_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub processed_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn enqueue_booking(
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<EnqueueBookingRequest>,
) -> Result<(StatusCode, Json<EnqueueBookingResponse>), (StatusCode, Json<serde_json::Value>)> {
    let mut tx = state.pool.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    // 1. Check active booking count for this user
    // Active = held seat_bookings (with expires_at > now()) + pending/processing in booking_queue
    let active_holds_count =
        SeatBookingRepository::count_active_holds_by_user_id(&mut *tx, auth_user.id)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
            })?;

    let in_flight_queue_count =
        BookingQueueRepository::count_in_flight_by_user_id(&mut *tx, auth_user.id)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
            })?;

    let total_active = active_holds_count + in_flight_queue_count;
    if total_active >= state.config.max_bookings_per_user {
        tx.rollback().await.ok();
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!(
                    "Maximum active booking limit reached ({} seats max). Please complete or release existing holds.",
                    state.config.max_bookings_per_user
                )
            })),
        ));
    }

    // Check event-level limits: max seats per booking session and max cumulative seats per person
    let event_limits = EventRepository::find_limits_by_showtime_id(&mut *tx, payload.showtime_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    if let Some((max_booking, max_user)) = event_limits {
        // A. Limit per booking session / showtime
        let showtime_active_holds =
            SeatBookingRepository::count_active_holds_by_showtime_and_user_id(
                &mut *tx,
                payload.showtime_id,
                auth_user.id,
            )
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
            })?;

        if showtime_active_holds + in_flight_queue_count >= max_booking as i64 {
            tx.rollback().await.ok();
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": format!(
                        "Booking limit reached: maximum {} seats per booking for this event.",
                        max_booking
                    )
                })),
            ));
        }

        // B. Cumulative limit per customer for this entire event
        let total_user_event_seats =
            SeatBookingRepository::count_confirmed_and_held_by_showtime_and_user_id(
                &mut *tx,
                payload.showtime_id,
                auth_user.id,
            )
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
            })?;

        if total_user_event_seats + in_flight_queue_count >= max_user as i64 {
            tx.rollback().await.ok();
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": format!(
                        "Personal event limit reached: maximum {} total seats allowed per customer for this event.",
                        max_user
                    )
                })),
            ));
        }
    }

    // 2. Fetch the seat and zone to ensure existence
    let zone_id = SeatRepository::find_zone_id_by_id(&mut *tx, payload.seat_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    let zone_id = match zone_id {
        Some(z) => z,
        None => {
            tx.rollback().await.ok();
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Seat not found" })),
            ));
        }
    };

    // Verify showtime exists
    let showtime_exists = ShowtimeRepository::exists_by_id(&mut *tx, payload.showtime_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    if !showtime_exists {
        tx.rollback().await.ok();
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Showtime not found" })),
        ));
    }

    // 3. Insert into booking_queue
    let booking_uuid = BookingQueueRepository::save(
        &mut *tx,
        auth_user.id,
        payload.showtime_id,
        zone_id,
        payload.seat_id,
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

    tracing::info!(
        booking_uuid = %booking_uuid,
        user_id = %auth_user.id,
        showtime_id = %payload.showtime_id,
        seat_id = %payload.seat_id,
        "Enqueued booking request"
    );

    Ok((
        StatusCode::ACCEPTED,
        Json(EnqueueBookingResponse {
            booking_uuid,
            status: "pending".to_string(),
        }),
    ))
}

pub async fn release_hold(
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<ReleaseHoldRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Check if this seat booking is already attached to a pending order.
    // If so, reject the release — the seat is locked until payment completes or is cancelled.
    let in_pending_order: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1
            FROM seat_bookings sb
            JOIN order_seats os ON os.seat_booking_id = sb.id
            JOIN orders o ON o.id = os.order_id
            WHERE sb.user_id = $1
              AND sb.showtime_id = $2
              AND sb.seat_id = $3
              AND sb.status = 'held'
              AND o.status = 'pending_payment'
        )",
    )
    .bind(auth_user.id)
    .bind(payload.showtime_id)
    .bind(payload.seat_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    if in_pending_order {
        return Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "Cannot release this seat — payment is already in progress. Complete or cancel your order first."
            })),
        ));
    }

    let released = SeatBookingRepository::release_hold_by_user_and_seat(
        &state.pool,
        auth_user.id,
        payload.showtime_id,
        payload.seat_id,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    if !released {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Active seat hold not found for this user" })),
        ));
    }

    tracing::info!(
        user_id = %auth_user.id,
        showtime_id = %payload.showtime_id,
        seat_id = %payload.seat_id,
        "Seat hold released by user"
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Seat hold released successfully"
    })))
}

pub async fn get_booking_status(
    Path(uuid): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<BookingStatusResponse>, (StatusCode, Json<serde_json::Value>)> {
    let item = BookingQueueRepository::find_by_id(&state.pool, uuid)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    match item {
        Some(row) => Ok(Json(BookingStatusResponse {
            booking_uuid: row.id,
            status: row.status,
            failure_reason: row.failure_reason,
            showtime_id: row.showtime_id,
            seat_id: row.seat_id,
            zone_id: row.zone_id,
            created_at: row.created_at,
            processed_at: row.processed_at,
        })),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Booking request not found" })),
        )),
    }
}
