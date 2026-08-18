use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::OptionalAuthenticatedUser,
    models::{Event, SeatWithStatus, ShowtimeDetail, Theater},
    routes::AppState,
};

#[derive(Debug, Deserialize)]
pub struct SeatsQuery {
    pub showtime_id: Uuid,
}

pub async fn list_theaters(
    State(state): State<AppState>,
) -> Result<Json<Vec<Theater>>, (StatusCode, Json<serde_json::Value>)> {
    let theaters: Vec<Theater> = sqlx::query_as("SELECT id, name, address FROM theaters ORDER BY name")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(theaters))
}

pub async fn list_events(
    State(state): State<AppState>,
) -> Result<Json<Vec<Event>>, (StatusCode, Json<serde_json::Value>)> {
    let events: Vec<Event> = sqlx::query_as("SELECT id, title FROM events ORDER BY title")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(events))
}

pub async fn list_showtimes(
    State(state): State<AppState>,
) -> Result<Json<Vec<ShowtimeDetail>>, (StatusCode, Json<serde_json::Value>)> {
    let showtimes: Vec<ShowtimeDetail> = sqlx::query_as(
        "SELECT s.id, s.event_id, e.title as event_title, s.theater_id, t.name as theater_name, s.starts_at
         FROM showtimes s
         JOIN events e ON e.id = s.event_id
         JOIN theaters t ON t.id = s.theater_id
         ORDER BY s.starts_at ASC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    Ok(Json(showtimes))
}

pub async fn get_theater_seats(
    Path(theater_id): Path<Uuid>,
    Query(query): Query<SeatsQuery>,
    opt_user: OptionalAuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<SeatWithStatus>>, (StatusCode, Json<serde_json::Value>)> {
    let current_user_id = opt_user.0.map(|u| u.id);

    // Fetch all seats in this theater joined with zone and active hold/confirmation
    // A hold is active if status = 'held' AND expires_at > now()
    // A confirmed booking is active if status = 'confirmed'
    let query_str = "
        SELECT
            s.id,
            s.theater_id,
            s.zone_id,
            z.name AS zone_name,
            z.price_cents,
            s.row_label,
            s.seat_number,
            CASE
                WHEN sb.status = 'confirmed' THEN 'confirmed'
                WHEN sb.status = 'held' AND sb.expires_at > now() THEN 'held'
                ELSE 'available'
            END AS status,
            CASE
                WHEN sb.id IS NOT NULL 
                     AND (sb.status = 'confirmed' OR (sb.status = 'held' AND sb.expires_at > now()))
                     AND sb.user_id = $3 THEN true
                ELSE false
            END AS is_mine,
            CASE
                WHEN sb.status = 'held' AND sb.expires_at > now() THEN sb.expires_at
                ELSE NULL
            END AS held_expires_at,
            CASE
                WHEN sb.id IS NOT NULL 
                     AND (sb.status = 'confirmed' OR (sb.status = 'held' AND sb.expires_at > now()))
                     AND sb.user_id = $3 THEN sb.id
                ELSE NULL
            END AS seat_booking_id
        FROM seats s
        JOIN zones z ON z.id = s.zone_id
        LEFT JOIN seat_bookings sb 
            ON sb.seat_id = s.id 
            AND sb.showtime_id = $2
            AND sb.status IN ('held', 'confirmed')
            AND (sb.status = 'confirmed' OR sb.expires_at > now())
        WHERE s.theater_id = $1
        ORDER BY s.row_label ASC, s.seat_number ASC
    ";

    let seats: Vec<SeatWithStatus> = sqlx::query_as(query_str)
        .bind(theater_id)
        .bind(query.showtime_id)
        .bind(current_user_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(seats))
}
