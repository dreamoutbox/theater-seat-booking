use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use repository::{
    Event, EventRepository, SaveLayoutPayload, SeatRepository, ShowtimeDetail, ShowtimeRepository,
    Theater, TheaterLayout, TheaterRepository, UserRole, ZoneRepository,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::{AuthenticatedUser, OptionalAuthenticatedUser, check_role},
    models::SeatWithStatus,
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
    pub owner_id: Option<Uuid>,
    pub tax_business_name: Option<String>,
    pub tax_id: Option<String>,
    pub tax_address: Option<String>,
    pub tax_email: Option<String>,
    pub tax_phone: Option<String>,
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
    let theaters = TheaterRepository::find_all(&state.pool)
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
    check_role(&auth_user, &[UserRole::Admin, UserRole::Staff]).map_err(|_e| {
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

    let theater = TheaterRepository::save(&state.pool, name, payload.address.as_deref())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    // Create default zone for the new theater
    let _ = ZoneRepository::save(&state.pool, theater.id, "Standard", 1200, "#14b8a6").await;

    tracing::info!(theater_id = %theater.id, name = %theater.name, user_id = %auth_user.id, "Created new theater");

    Ok((StatusCode::CREATED, Json(theater)))
}

pub async fn update_theater(
    Path(id): Path<Uuid>,
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateTheaterPayload>,
) -> Result<Json<Theater>, (StatusCode, Json<serde_json::Value>)> {
    check_role(&auth_user, &[UserRole::Admin, UserRole::Staff]).map_err(|_e| {
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

    let theater = TheaterRepository::update(&state.pool, id, name, payload.address.as_deref())
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
    check_role(&auth_user, &[UserRole::Admin, UserRole::Staff]).map_err(|_e| {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        )
    })?;

    let deleted = TheaterRepository::delete_by_id(&state.pool, id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    if !deleted {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Theater not found" })),
        ));
    }

    tracing::info!(theater_id = %id, user_id = %auth_user.id, "Deleted theater");

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Theater deleted successfully"
    })))
}

pub async fn list_events(
    opt_user: OptionalAuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Event>>, (StatusCode, Json<serde_json::Value>)> {
    let events = if let Some(ref user) = opt_user.0 {
        if user.role == UserRole::EventOrganizer {
            EventRepository::find_by_owner_id(&state.pool, user.id).await
        } else {
            EventRepository::find_all(&state.pool).await
        }
    } else {
        EventRepository::find_all(&state.pool).await
    }
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
        &[UserRole::Admin, UserRole::Staff, UserRole::EventOrganizer],
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
    let owner_id = if auth_user.role == UserRole::EventOrganizer {
        Some(auth_user.id)
    } else {
        payload.owner_id.or(Some(auth_user.id))
    };

    let event = EventRepository::save(
        &state.pool,
        title,
        max_booking,
        max_user,
        owner_id,
        payload.tax_business_name.as_deref(),
        payload.tax_id.as_deref(),
        payload.tax_address.as_deref(),
        payload.tax_email.as_deref(),
        payload.tax_phone.as_deref(),
    )
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
        &[UserRole::Admin, UserRole::Staff, UserRole::EventOrganizer],
    )
    .map_err(|_e| {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        )
    })?;

    let existing = EventRepository::find_by_id(&state.pool, id)
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
                Json(serde_json::json!({ "error": "Event not found" })),
            )
        })?;

    if auth_user.role == UserRole::EventOrganizer && existing.owner_id != Some(auth_user.id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "You do not own this event" })),
        ));
    }

    let title = payload.title.trim();
    if title.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Event title is required" })),
        ));
    }

    let max_booking = payload.max_seats_per_booking.unwrap_or(6).max(1);
    let max_user = payload.max_seats_per_user.unwrap_or(10).max(1);
    let owner_id = if auth_user.role == UserRole::EventOrganizer {
        existing.owner_id
    } else {
        payload.owner_id.or(existing.owner_id)
    };

    let event = EventRepository::update(
        &state.pool,
        id,
        title,
        max_booking,
        max_user,
        owner_id,
        payload.tax_business_name.as_deref().or(existing.tax_business_name.as_deref()),
        payload.tax_id.as_deref().or(existing.tax_id.as_deref()),
        payload.tax_address.as_deref().or(existing.tax_address.as_deref()),
        payload.tax_email.as_deref().or(existing.tax_email.as_deref()),
        payload.tax_phone.as_deref().or(existing.tax_phone.as_deref()),
    )
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
        &[UserRole::Admin, UserRole::Staff, UserRole::EventOrganizer],
    )
    .map_err(|_e| {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        )
    })?;

    let existing = EventRepository::find_by_id(&state.pool, id)
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
                Json(serde_json::json!({ "error": "Event not found" })),
            )
        })?;

    if auth_user.role == UserRole::EventOrganizer && existing.owner_id != Some(auth_user.id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "You do not own this event" })),
        ));
    }

    let deleted = EventRepository::delete_by_id(&state.pool, id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    if !deleted {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Event not found" })),
        ));
    }

    tracing::info!(event_id = %id, user_id = %auth_user.id, "Deleted event");

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Event deleted successfully"
    })))
}

pub async fn list_showtimes(
    State(state): State<AppState>,
) -> Result<Json<Vec<ShowtimeDetail>>, (StatusCode, Json<serde_json::Value>)> {
    let showtimes = ShowtimeRepository::find_all(&state.pool)
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
        &[UserRole::Admin, UserRole::Staff, UserRole::EventOrganizer],
    )
    .map_err(|_e| {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        )
    })?;

    let showtime = ShowtimeRepository::save(
        &state.pool,
        payload.event_id,
        payload.theater_id,
        payload.starts_at,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    tracing::info!(showtime_id = %showtime.id, event_id = %payload.event_id, "Created new showtime");

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
        &[UserRole::Admin, UserRole::Staff, UserRole::EventOrganizer],
    )
    .map_err(|_e| {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        )
    })?;

    let showtime =
        ShowtimeRepository::update(&state.pool, id, payload.theater_id, payload.starts_at)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
            })?;

    match showtime {
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
        &[UserRole::Admin, UserRole::Staff, UserRole::EventOrganizer],
    )
    .map_err(|_e| {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        )
    })?;

    let deleted = ShowtimeRepository::delete_by_id(&state.pool, id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    if !deleted {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Showtime not found" })),
        ));
    }

    tracing::info!(showtime_id = %id, user_id = %auth_user.id, "Deleted showtime");

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Showtime deleted successfully"
    })))
}

pub async fn get_theater_seats(
    Path(theater_id): Path<Uuid>,
    Query(query): Query<SeatsQuery>,
    opt_user: OptionalAuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<SeatWithStatus>>, (StatusCode, Json<serde_json::Value>)> {
    let current_user_id = opt_user.0.map(|u| u.id);

    let seats = SeatRepository::find_seats_with_status_by_theater_id(
        &state.pool,
        theater_id,
        Some(query.showtime_id),
        current_user_id,
    )
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

    let theater_id = ShowtimeRepository::find_theater_id_by_id(&state.pool, showtime_id)
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

    let seats = SeatRepository::find_seats_with_status_by_theater_id(
        &state.pool,
        theater_id,
        Some(showtime_id),
        current_user_id,
    )
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
    let theater = TheaterRepository::find_by_id(&state.pool, theater_id)
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

    let zones = ZoneRepository::find_by_theater_id(&state.pool, theater_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    let seats = SeatRepository::find_by_theater_id(&state.pool, theater_id)
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
        &[UserRole::Admin, UserRole::Staff, UserRole::EventOrganizer],
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
                let updated = ZoneRepository::update(
                    &mut *tx,
                    existing_id,
                    theater_id,
                    zone_name,
                    zone_item.price_cents,
                    zone_color,
                )
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": e.to_string() })),
                    )
                })?;

                if updated {
                    existing_id
                } else {
                    ZoneRepository::save_with_id(
                        &mut *tx,
                        existing_id,
                        theater_id,
                        zone_name,
                        zone_item.price_cents,
                        zone_color,
                    )
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
                ZoneRepository::save(
                    &mut *tx,
                    theater_id,
                    zone_name,
                    zone_item.price_cents,
                    zone_color,
                )
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
            let zone_exists =
                ZoneRepository::exists_by_id_and_theater_id(&mut *tx, seat_item.zone_id, theater_id)
                    .await
                    .unwrap_or(false);

            if !zone_exists {
                tx.rollback().await.ok();
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": format!(
                            "Invalid zone id {} for seat {}{}",
                            seat_item.zone_id, row_label, seat_item.seat_number
                        )
                    })),
                ));
            }
        }

        current_seat_keys.insert((row_label.clone(), seat_item.seat_number));

        // Upsert seat on (theater_id, row_label, seat_number)
        SeatRepository::upsert(
            &mut *tx,
            theater_id,
            seat_item.zone_id,
            &row_label,
            seat_item.seat_number,
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;
    }

    // 3. Remove seats that are no longer in the payload
    let existing_seats = SeatRepository::find_all_keys_by_theater_id(&mut *tx, theater_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    for (seat_id, r_label, s_num) in existing_seats {
        if !current_seat_keys.contains(&(r_label, s_num)) {
            SeatRepository::delete_by_id(&mut *tx, seat_id)
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
    let theater = TheaterRepository::find_by_id(&state.pool, theater_id)
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

    let zones = ZoneRepository::find_by_theater_id(&state.pool, theater_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    let seats = SeatRepository::find_by_theater_id(&state.pool, theater_id)
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
