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
    models::{
        Event, SaveLayoutPayload, Seat, SeatWithStatus, ShowtimeDetail, Theater, TheaterLayout,
        UserRole, Zone,
    },
    routes::AppState,
};

#[derive(Debug, Deserialize)]
pub struct SeatsQuery {
    pub showtime_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateTheaterPayload {
    pub name: String,
    pub address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEventPayload {
    pub title: String,
    pub max_seats_per_booking: Option<i32>,
    pub max_seats_per_user: Option<i32>,
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

pub async fn create_theater(
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateTheaterPayload>,
) -> Result<(StatusCode, Json<Theater>), (StatusCode, Json<serde_json::Value>)> {
    check_role(
        &auth_user,
        &[UserRole::Admin, UserRole::Staff],
    )
    .map_err(|_e| {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        )
    })?;

    let name = payload.name.trim();
    if name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Theater name is required" })),
        ));
    }

    let theater_id = Uuid::new_v4();
    let theater: Theater = sqlx::query_as(
        "INSERT INTO theaters (id, name, address) VALUES ($1, $2, $3) RETURNING id, name, address",
    )
    .bind(theater_id)
    .bind(name)
    .bind(payload.address.as_deref())
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    // Create default zone for the new theater
    sqlx::query(
        "INSERT INTO zones (theater_id, name, price_cents) VALUES ($1, 'Standard', 1200)",
    )
    .bind(theater_id)
    .execute(&state.pool)
    .await
    .ok();

    tracing::info!(theater_id = %theater.id, name = %theater.name, user_id = %auth_user.id, "Created new theater");

    Ok((StatusCode::CREATED, Json(theater)))
}

pub async fn update_theater(
    Path(id): Path<Uuid>,
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateTheaterPayload>,
) -> Result<Json<Theater>, (StatusCode, Json<serde_json::Value>)> {
    check_role(
        &auth_user,
        &[UserRole::Admin, UserRole::Staff],
    )
    .map_err(|_e| {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        )
    })?;

    let name = payload.name.trim();
    if name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Theater name is required" })),
        ));
    }

    let theater: Option<Theater> = sqlx::query_as(
        "UPDATE theaters SET name = $1, address = $2 WHERE id = $3 RETURNING id, name, address",
    )
    .bind(name)
    .bind(payload.address.as_deref())
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    match theater {
        Some(t) => {
            tracing::info!(theater_id = %id, name = %t.name, "Updated theater");
            Ok(Json(t))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Theater not found" })),
        )),
    }
}

pub async fn delete_theater(
    Path(id): Path<Uuid>,
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    check_role(
        &auth_user,
        &[UserRole::Admin, UserRole::Staff],
    )
    .map_err(|_e| {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        )
    })?;

    let res = sqlx::query("DELETE FROM theaters WHERE id = $1")
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
            Json(serde_json::json!({ "error": "Theater not found" })),
        ));
    }

    tracing::info!(theater_id = %id, user_id = %auth_user.id, "Deleted theater");

    Ok(Json(serde_json::json!({ "success": true, "message": "Theater deleted successfully" })))
}

pub async fn list_events(
    State(state): State<AppState>,
) -> Result<Json<Vec<Event>>, (StatusCode, Json<serde_json::Value>)> {
    let events: Vec<Event> = sqlx::query_as(
        "SELECT id, title, max_seats_per_booking, max_seats_per_user FROM events ORDER BY title",
    )
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

    let max_booking = payload.max_seats_per_booking.unwrap_or(6).max(1);
    let max_user = payload.max_seats_per_user.unwrap_or(10).max(1);

    let event: Event = sqlx::query_as(
        "INSERT INTO events (title, max_seats_per_booking, max_seats_per_user)
         VALUES ($1, $2, $3)
         RETURNING id, title, max_seats_per_booking, max_seats_per_user",
    )
    .bind(title)
    .bind(max_booking)
    .bind(max_user)
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

    let max_booking = payload.max_seats_per_booking.unwrap_or(6).max(1);
    let max_user = payload.max_seats_per_user.unwrap_or(10).max(1);

    let event: Option<Event> = sqlx::query_as(
        "UPDATE events
         SET title = $1, max_seats_per_booking = $2, max_seats_per_user = $3
         WHERE id = $4
         RETURNING id, title, max_seats_per_booking, max_seats_per_user",
    )
    .bind(title)
    .bind(max_booking)
    .bind(max_user)
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
            tracing::info!(event_id = %id, title = %e.title, max_booking = %e.max_seats_per_booking, max_user = %e.max_seats_per_user, "Updated event");
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
            z.color AS zone_color,
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

pub async fn get_showtime_seats(
    Path(showtime_id): Path<Uuid>,
    opt_user: OptionalAuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<SeatWithStatus>>, (StatusCode, Json<serde_json::Value>)> {
    let current_user_id = opt_user.0.map(|u| u.id);

    let theater_id: Option<Uuid> = sqlx::query_scalar("SELECT theater_id FROM showtimes WHERE id = $1")
        .bind(showtime_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    let Some(theater_id) = theater_id else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Showtime not found" })),
        ));
    };

    let query_str = "
        SELECT
            s.id,
            s.theater_id,
            s.zone_id,
            z.name AS zone_name,
            z.color AS zone_color,
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
        .bind(showtime_id)
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

pub async fn get_theater_layout(
    Path(theater_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<TheaterLayout>, (StatusCode, Json<serde_json::Value>)> {
    let theater: Theater = sqlx::query_as("SELECT id, name, address FROM theaters WHERE id = $1")
        .bind(theater_id)
        .fetch_optional(&state.pool)
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
                Json(serde_json::json!({ "error": "Theater not found" })),
            )
        })?;

    let zones: Vec<Zone> = sqlx::query_as(
        "SELECT id, theater_id, name, price_cents, color FROM zones WHERE theater_id = $1 ORDER BY price_cents DESC",
    )
    .bind(theater_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let seats: Vec<Seat> = sqlx::query_as(
        "SELECT id, theater_id, zone_id, row_label, seat_number FROM seats WHERE theater_id = $1 ORDER BY row_label ASC, seat_number ASC",
    )
    .bind(theater_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    Ok(Json(TheaterLayout {
        theater,
        zones,
        seats,
    }))
}

pub async fn save_theater_layout(
    Path(theater_id): Path<Uuid>,
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<SaveLayoutPayload>,
) -> Result<Json<TheaterLayout>, (StatusCode, Json<serde_json::Value>)> {
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

    let mut tx = state.pool.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    // 1. Sync zones
    let mut valid_zone_ids = std::collections::HashSet::new();
    for zone_item in &payload.zones {
        let zone_name = zone_item.name.trim();
        if zone_name.is_empty() {
            continue;
        }

        let zone_color = zone_item.color.as_deref().unwrap_or("#14b8a6");

        let zone_id = match zone_item.id {
            Some(existing_id) => {
                let updated = sqlx::query(
                    "UPDATE zones SET name = $1, price_cents = $2, color = $3 WHERE id = $4 AND theater_id = $5",
                )
                .bind(zone_name)
                .bind(zone_item.price_cents)
                .bind(zone_color)
                .bind(existing_id)
                .bind(theater_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": e.to_string() })),
                    )
                })?;

                if updated.rows_affected() > 0 {
                    existing_id
                } else {
                    sqlx::query_scalar(
                        "INSERT INTO zones (id, theater_id, name, price_cents, color) VALUES ($1, $2, $3, $4, $5) RETURNING id",
                    )
                    .bind(existing_id)
                    .bind(theater_id)
                    .bind(zone_name)
                    .bind(zone_item.price_cents)
                    .bind(zone_color)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({ "error": e.to_string() })),
                        )
                    })?
                }
            }
            None => {
                sqlx::query_scalar(
                    "INSERT INTO zones (theater_id, name, price_cents, color) VALUES ($1, $2, $3, $4) RETURNING id",
                )
                .bind(theater_id)
                .bind(zone_name)
                .bind(zone_item.price_cents)
                .bind(zone_color)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": e.to_string() })),
                    )
                })?
            }
        };

        valid_zone_ids.insert(zone_id);
    }

    // 2. Sync seats
    let mut current_seat_keys = std::collections::HashSet::new();

    for seat_item in &payload.seats {
        let row_label = seat_item.row_label.trim().to_uppercase();
        if row_label.is_empty() || seat_item.seat_number <= 0 {
            continue;
        }

        // Validate zone_id is valid
        if !valid_zone_ids.contains(&seat_item.zone_id) {
            let zone_exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM zones WHERE id = $1 AND theater_id = $2)",
            )
            .bind(seat_item.zone_id)
            .bind(theater_id)
            .fetch_one(&mut *tx)
            .await
            .unwrap_or(false);

            if !zone_exists {
                tx.rollback().await.ok();
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": format!("Invalid zone id {} for seat {}{}", seat_item.zone_id, row_label, seat_item.seat_number) })),
                ));
            }
        }

        current_seat_keys.insert((row_label.clone(), seat_item.seat_number));

        // Upsert seat on (theater_id, row_label, seat_number)
        sqlx::query(
            "INSERT INTO seats (theater_id, zone_id, row_label, seat_number)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (theater_id, row_label, seat_number)
             DO UPDATE SET zone_id = EXCLUDED.zone_id",
        )
        .bind(theater_id)
        .bind(seat_item.zone_id)
        .bind(&row_label)
        .bind(seat_item.seat_number)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;
    }

    // 3. Remove seats that are no longer in the payload
    let existing_seats: Vec<(Uuid, String, i32)> = sqlx::query_as(
        "SELECT id, row_label, seat_number FROM seats WHERE theater_id = $1",
    )
    .bind(theater_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    for (seat_id, r_label, s_num) in existing_seats {
        if !current_seat_keys.contains(&(r_label, s_num)) {
            sqlx::query("DELETE FROM seats WHERE id = $1")
                .bind(seat_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": e.to_string() })),
                    )
                })?;
        }
    }

    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    // 4. Return updated layout
    let theater: Theater = sqlx::query_as("SELECT id, name, address FROM theaters WHERE id = $1")
        .bind(theater_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    let zones: Vec<Zone> = sqlx::query_as(
        "SELECT id, theater_id, name, price_cents, color FROM zones WHERE theater_id = $1 ORDER BY price_cents DESC",
    )
    .bind(theater_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let seats: Vec<Seat> = sqlx::query_as(
        "SELECT id, theater_id, zone_id, row_label, seat_number FROM seats WHERE theater_id = $1 ORDER BY row_label ASC, seat_number ASC",
    )
    .bind(theater_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    tracing::info!(theater_id = %theater_id, seats_count = seats.len(), zones_count = zones.len(), "Theater layout updated successfully");

    Ok(Json(TheaterLayout {
        theater,
        zones,
        seats,
    }))
}
