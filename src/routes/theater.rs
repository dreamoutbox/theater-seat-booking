use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::{check_role, AuthenticatedUser, OptionalAuthenticatedUser},
    models::{Event, SeatWithStatus, ShowtimeDetail, Theater, UserRole},
    routes::AppState,
};

#[derive(Debug, Deserialize)]
pub struct SeatsQuery {
    pub showtime_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateEventPayload {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateShowtimePayload {
    pub event_id: Uuid,
    pub theater_id: Uuid,
    pub starts_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShowtimePayload {
    pub theater_id: Option<Uuid>,
    pub starts_at: DateTime<Utc>,
}

pub async fn list_theaters(
    State(state): State<AppState>,
) -> Result<Json<Vec<Theater>>, (StatusCode, Json<serde_json::Value>)> {
    let theaters: Vec<Theater> =
        sqlx::query_as("SELECT id, name, address FROM theaters ORDER BY name")
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

pub async fn create_event(
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateEventPayload>,
) -> Result<(StatusCode, Json<Event>), (StatusCode, Json<serde_json::Value>)> {
    check_role(
        &auth_user,
        &[
            UserRole::Admin,
            UserRole::Staff,
            UserRole::EventOrganizer,
        ],
    )
    .map_err(|_e| {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        )
    })?;

    let title = payload.title.trim();
    if title.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Event title is required" })),
        ));
    }

    let event: Event = sqlx::query_as(
        "INSERT INTO events (title) VALUES ($1) RETURNING id, title",
    )
    .bind(title)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    tracing::info!(event_id = %event.id, title = %event.title, user_id = %auth_user.id, "Created new event");

    Ok((StatusCode::CREATED, Json(event)))
}

pub async fn update_event(
    Path(id): Path<Uuid>,
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateEventPayload>,
) -> Result<Json<Event>, (StatusCode, Json<serde_json::Value>)> {
    check_role(
        &auth_user,
        &[
            UserRole::Admin,
            UserRole::Staff,
            UserRole::EventOrganizer,
        ],
    )
    .map_err(|_e| {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        )
    })?;

    let title = payload.title.trim();
    if title.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Event title is required" })),
        ));
    }

    let event: Option<Event> = sqlx::query_as(
        "UPDATE events SET title = $1 WHERE id = $2 RETURNING id, title",
    )
    .bind(title)
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    match event {
        Some(e) => {
            tracing::info!(event_id = %id, title = %e.title, "Updated event");
            Ok(Json(e))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Event not found" })),
        )),
    }
}

pub async fn delete_event(
    Path(id): Path<Uuid>,
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    check_role(
        &auth_user,
        &[
            UserRole::Admin,
            UserRole::Staff,
            UserRole::EventOrganizer,
        ],
    )
    .map_err(|_e| {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        )
    })?;

    let res = sqlx::query("DELETE FROM events WHERE id = $1")
        .bind(id)
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
            Json(serde_json::json!({ "error": "Event not found" })),
        ));
    }

    tracing::info!(event_id = %id, user_id = %auth_user.id, "Deleted event");

    Ok(Json(serde_json::json!({ "success": true, "message": "Event deleted successfully" })))
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

pub async fn create_showtime(
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateShowtimePayload>,
) -> Result<(StatusCode, Json<ShowtimeDetail>), (StatusCode, Json<serde_json::Value>)> {
    check_role(
        &auth_user,
        &[
            UserRole::Admin,
            UserRole::Staff,
            UserRole::EventOrganizer,
        ],
    )
    .map_err(|_e| {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        )
    })?;

    let showtime_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO showtimes (id, event_id, theater_id, starts_at)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(showtime_id)
    .bind(payload.event_id)
    .bind(payload.theater_id)
    .bind(payload.starts_at)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let showtime: ShowtimeDetail = sqlx::query_as(
        "SELECT s.id, s.event_id, e.title as event_title, s.theater_id, t.name as theater_name, s.starts_at
         FROM showtimes s
         JOIN events e ON e.id = s.event_id
         JOIN theaters t ON t.id = s.theater_id
         WHERE s.id = $1",
    )
    .bind(showtime_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    tracing::info!(showtime_id = %showtime_id, event_id = %payload.event_id, "Created new showtime");

    Ok((StatusCode::CREATED, Json(showtime)))
}

pub async fn update_showtime(
    Path(id): Path<Uuid>,
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<UpdateShowtimePayload>,
) -> Result<Json<ShowtimeDetail>, (StatusCode, Json<serde_json::Value>)> {
    check_role(
        &auth_user,
        &[
            UserRole::Admin,
            UserRole::Staff,
            UserRole::EventOrganizer,
        ],
    )
    .map_err(|_e| {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        )
    })?;

    if let Some(theater_id) = payload.theater_id {
        sqlx::query("UPDATE showtimes SET theater_id = $1, starts_at = $2 WHERE id = $3")
            .bind(theater_id)
            .bind(payload.starts_at)
            .bind(id)
            .execute(&state.pool)
            .await
    } else {
        sqlx::query("UPDATE showtimes SET starts_at = $1 WHERE id = $2")
            .bind(payload.starts_at)
            .bind(id)
            .execute(&state.pool)
            .await
    }
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let showtime_opt: Option<ShowtimeDetail> = sqlx::query_as(
        "SELECT s.id, s.event_id, e.title as event_title, s.theater_id, t.name as theater_name, s.starts_at
         FROM showtimes s
         JOIN events e ON e.id = s.event_id
         JOIN theaters t ON t.id = s.theater_id
         WHERE s.id = $1",
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

    match showtime_opt {
        Some(s) => Ok(Json(s)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Showtime not found" })),
        )),
    }
}

pub async fn delete_showtime(
    Path(id): Path<Uuid>,
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    check_role(
        &auth_user,
        &[
            UserRole::Admin,
            UserRole::Staff,
            UserRole::EventOrganizer,
        ],
    )
    .map_err(|_e| {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        )
    })?;

    let res = sqlx::query("DELETE FROM showtimes WHERE id = $1")
        .bind(id)
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
            Json(serde_json::json!({ "error": "Showtime not found" })),
        ));
    }

    tracing::info!(showtime_id = %id, user_id = %auth_user.id, "Deleted showtime");

    Ok(Json(serde_json::json!({ "success": true, "message": "Showtime deleted successfully" })))
}

pub async fn get_theater_seats(
    Path(theater_id): Path<Uuid>,
    Query(query): Query<SeatsQuery>,
    opt_user: OptionalAuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<SeatWithStatus>>, (StatusCode, Json<serde_json::Value>)> {
    let current_user_id = opt_user.0.map(|u| u.id);

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
