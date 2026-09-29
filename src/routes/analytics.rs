use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use repository::{
    AnalyticsRepository, EventDashboardMetrics, EventRepository, ShowtimeDashboardRow, UserRole,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{AuthenticatedUser, check_role},
    routes::AppState,
};

#[derive(Debug, Deserialize)]
pub struct DashboardQuery {
    pub event_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct DashboardResponse {
    pub metrics: EventDashboardMetrics,
    pub showtimes: Vec<ShowtimeDashboardRow>,
}

pub async fn get_dashboard_data(
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
    Query(query): Query<DashboardQuery>,
) -> Result<Json<DashboardResponse>, (StatusCode, Json<serde_json::Value>)> {
    check_role(
        &auth_user,
        &[UserRole::Admin, UserRole::Staff, UserRole::EventOrganizer],
    )
    .map_err(|_e| {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Unauthorized to access analytics dashboard" })),
        )
    })?;

    // Determine ownership filter based on role
    let owner_id = if auth_user.role == UserRole::EventOrganizer {
        // If organizer requested a specific event, verify they own it
        if let Some(eid) = query.event_id {
            let event_opt = EventRepository::find_by_id(&state.pool, eid)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": e.to_string() })),
                    )
                })?;

            if let Some(event) = event_opt {
                if event.owner_id != Some(auth_user.id) {
                    return Err((
                        StatusCode::FORBIDDEN,
                        Json(
                            serde_json::json!({ "error": "Access denied to requested event analytics" }),
                        ),
                    ));
                }
            } else {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": "Event not found" })),
                ));
            }
        }
        Some(auth_user.id)
    } else {
        None
    };

    let metrics =
        AnalyticsRepository::get_event_dashboard_metrics(&state.pool, query.event_id, owner_id)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Failed to compute metrics: {e}") })),
                )
            })?;

    let showtimes = AnalyticsRepository::get_showtime_dashboard_breakdown(
        &state.pool,
        query.event_id,
        owner_id,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                serde_json::json!({ "error": format!("Failed to fetch showtime breakdown: {e}") }),
            ),
        )
    })?;

    Ok(Json(DashboardResponse { metrics, showtimes }))
}
