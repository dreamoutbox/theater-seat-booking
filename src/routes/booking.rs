use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::AuthenticatedUser, models::BookingQueueItem, routes::AppState};

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
    let active_holds_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM seat_bookings
         WHERE user_id = $1 AND status = 'held' AND expires_at > now()",
    )
    .bind(auth_user.id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let in_flight_queue_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM booking_queue
         WHERE user_id = $1 AND status IN ('pending', 'processing')",
    )
    .bind(auth_user.id)
    .fetch_one(&mut *tx)
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

    // 2. Fetch the seat and zone to ensure existence
    let zone_id: Option<Uuid> = sqlx::query_scalar("SELECT zone_id FROM seats WHERE id = $1")
        .bind(payload.seat_id)
        .fetch_optional(&mut *tx)
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
    let showtime_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM showtimes WHERE id = $1)")
            .bind(payload.showtime_id)
            .fetch_one(&mut *tx)
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
    let booking_uuid = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO booking_queue (id, user_id, showtime_id, zone_id, seat_id, status)
         VALUES ($1, $2, $3, $4, $5, 'pending')",
    )
    .bind(booking_uuid)
    .bind(auth_user.id)
    .bind(payload.showtime_id)
    .bind(zone_id)
    .bind(payload.seat_id)
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
    let res = sqlx::query(
        "UPDATE seat_bookings
         SET status = 'released'
         WHERE user_id = $1 AND showtime_id = $2 AND seat_id = $3 AND status = 'held'",
    )
    .bind(auth_user.id)
    .bind(payload.showtime_id)
    .bind(payload.seat_id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    if res.rows_affected() == 0 {
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
    let item: Option<BookingQueueItem> =
        sqlx::query_as("SELECT * FROM booking_queue WHERE id = $1")
            .bind(uuid)
            .fetch_optional(&state.pool)
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
