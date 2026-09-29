use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use repository::{
    Event, EventRepository, GatekeeperDetail, GatekeeperRepository, UserRepository, UserRole,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::{AuthenticatedUser, check_role, hash_password},
    routes::AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateGatekeeperPayload {
    pub email: String,
    pub password: String,
}

pub async fn list_event_gatekeepers(
    Path(event_id): Path<Uuid>,
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<GatekeeperDetail>>, (StatusCode, Json<serde_json::Value>)> {
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

    // If EventOrganizer, verify ownership
    if auth_user.role == UserRole::EventOrganizer {
        let event = EventRepository::find_by_id(&state.pool, event_id)
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

        if event.owner_id != Some(auth_user.id) {
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({ "error": "You do not own this event" })),
            ));
        }
    }

    let gatekeepers = GatekeeperRepository::find_by_event_id(&state.pool, event_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(gatekeepers))
}

pub async fn create_and_assign_gatekeeper(
    Path(event_id): Path<Uuid>,
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateGatekeeperPayload>,
) -> Result<(StatusCode, Json<GatekeeperDetail>), (StatusCode, Json<serde_json::Value>)> {
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

    // If EventOrganizer, verify ownership
    if auth_user.role == UserRole::EventOrganizer {
        let event = EventRepository::find_by_id(&state.pool, event_id)
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

        if event.owner_id != Some(auth_user.id) {
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({ "error": "You do not own this event" })),
            ));
        }
    }

    let email = payload.email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid email address" })),
        ));
    }

    if payload.password.len() < 6 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Password must be at least 6 characters" })),
        ));
    }

    // Find or create user
    let user = match UserRepository::find_by_email(&state.pool, &email)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })? {
        Some(u) => u,
        None => {
            let password_hash = hash_password(&payload.password).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e })),
                )
            })?;
            UserRepository::save(&state.pool, &email, &password_hash, UserRole::Gatekeeper)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": e.to_string() })),
                    )
                })?
        }
    };

    GatekeeperRepository::assign(&state.pool, event_id, user.id, Some(auth_user.id))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    tracing::info!(
        event_id = %event_id,
        gatekeeper_id = %user.id,
        assigned_by = %auth_user.id,
        "Assigned gatekeeper to event"
    );

    Ok((
        StatusCode::CREATED,
        Json(GatekeeperDetail {
            user_id: user.id,
            email: user.email,
            assigned_at: chrono::Utc::now(),
        }),
    ))
}

pub async fn remove_gatekeeper_from_event(
    Path((event_id, user_id)): Path<(Uuid, Uuid)>,
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

    // If EventOrganizer, verify ownership
    if auth_user.role == UserRole::EventOrganizer {
        let event = EventRepository::find_by_id(&state.pool, event_id)
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

        if event.owner_id != Some(auth_user.id) {
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({ "error": "You do not own this event" })),
            ));
        }
    }

    let removed = GatekeeperRepository::remove(&state.pool, event_id, user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    if !removed {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Gatekeeper assignment not found" })),
        ));
    }

    tracing::info!(
        event_id = %event_id,
        gatekeeper_id = %user_id,
        "Removed gatekeeper from event"
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Gatekeeper removed from event successfully"
    })))
}

pub async fn list_my_assigned_events(
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Event>>, (StatusCode, Json<serde_json::Value>)> {
    if auth_user.role == UserRole::Admin || auth_user.role == UserRole::Staff {
        let events = EventRepository::find_all(&state.pool).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;
        return Ok(Json(events));
    }

    if auth_user.role == UserRole::EventOrganizer {
        let events = EventRepository::find_by_owner_id(&state.pool, auth_user.id)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
            })?;
        return Ok(Json(events));
    }

    let events = GatekeeperRepository::find_assigned_events_by_user_id(&state.pool, auth_user.id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(events))
}
