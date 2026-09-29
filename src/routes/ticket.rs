use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use repository::{
    EventRepository, GatekeeperRepository, OrderRepository, TicketDetail, TicketRepository,
    UserRole,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{AuthenticatedUser, check_role},
    routes::AppState,
};

#[derive(Debug, Deserialize)]
pub struct ScanTicketRequest {
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct ScanTicketResponse {
    pub success: bool,
    pub message: String,
    pub ticket: TicketDetail,
}

pub async fn list_order_tickets(
    Path(order_id): Path<Uuid>,
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<TicketDetail>>, (StatusCode, Json<serde_json::Value>)> {
    // 1. Verify access to order
    let order_opt = OrderRepository::find_order_detail_by_id(&state.pool, order_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    let order = match order_opt {
        Some(o) => o,
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

    if !is_elevated && order.user_id != auth_user.id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Access denied to this order's tickets" })),
        ));
    }

    let tickets = TicketRepository::find_by_order_id(&state.pool, order_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(tickets))
}

pub async fn get_ticket_by_code(
    Path(code): Path<String>,
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<TicketDetail>, (StatusCode, Json<serde_json::Value>)> {
    let clean_code = code.trim().to_uppercase();
    let ticket_opt = TicketRepository::find_by_code(&state.pool, &clean_code)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    let ticket = match ticket_opt {
        Some(t) => t,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Ticket not found" })),
            ));
        }
    };

    // If gatekeeper, verify assigned to event
    if auth_user.role == UserRole::Gatekeeper {
        let assigned =
            GatekeeperRepository::is_assigned(&state.pool, auth_user.id, ticket.event_id)
                .await
                .unwrap_or(false);
        if !assigned {
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({ "error": "You are not assigned to gatekeep this event" })),
            ));
        }
    } else if auth_user.role == UserRole::Customer && ticket.user_id != auth_user.id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Access denied" })),
        ));
    }

    Ok(Json(ticket))
}

pub async fn scan_ticket(
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<ScanTicketRequest>,
) -> Result<Json<ScanTicketResponse>, (StatusCode, Json<serde_json::Value>)> {
    check_role(
        &auth_user,
        &[
            UserRole::Admin,
            UserRole::Staff,
            UserRole::EventOrganizer,
            UserRole::Gatekeeper,
        ],
    )
    .map_err(|_e| {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Unauthorized to scan tickets" })),
        )
    })?;

    let clean_code = payload.code.trim().to_uppercase();
    if clean_code.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Ticket code is required" })),
        ));
    }

    let ticket_opt = TicketRepository::find_by_code(&state.pool, &clean_code)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    let ticket = match ticket_opt {
        Some(t) => t,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Invalid ticket code: Ticket not found" })),
            ));
        }
    };

    // Scoping authorization check
    if auth_user.role == UserRole::Gatekeeper {
        let assigned =
            GatekeeperRepository::is_assigned(&state.pool, auth_user.id, ticket.event_id)
                .await
                .unwrap_or(false);
        if !assigned {
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "error": format!("Access denied: You are not assigned to '{}'", ticket.event_title)
                })),
            ));
        }
    } else if auth_user.role == UserRole::EventOrganizer {
        let event_opt = EventRepository::find_by_id(&state.pool, ticket.event_id)
            .await
            .unwrap_or(None);
        if let Some(event) = event_opt {
            if event.owner_id != Some(auth_user.id) {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({ "error": "You do not own this event" })),
                ));
            }
        }
    }

    // Status validation
    if ticket.status == "used" {
        let used_str = ticket
            .used_at
            .map(|t| t.to_rfc3339())
            .unwrap_or_else(|| "earlier".to_string());
        return Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": format!("Ticket ALREADY USED at {}", used_str),
                "ticket": ticket
            })),
        ));
    }

    if ticket.status == "refunded" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Ticket has been REFUNDED and is invalid for admission",
                "ticket": ticket
            })),
        ));
    }

    if ticket.status != "active" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("Ticket cannot be used (status: {})", ticket.status),
                "ticket": ticket
            })),
        ));
    }

    // Mark as used
    let marked = TicketRepository::mark_used(&state.pool, ticket.id, auth_user.id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    if !marked {
        return Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": "Ticket was already marked as used" })),
        ));
    }

    // Fetch updated detail
    let updated_ticket = TicketRepository::find_by_code(&state.pool, &clean_code)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?
        .unwrap_or(ticket);

    tracing::info!(
        ticket_code = %clean_code,
        scanned_by = %auth_user.id,
        event_id = %updated_ticket.event_id,
        "Ticket verified and marked as used"
    );

    Ok(Json(ScanTicketResponse {
        success: true,
        message: format!(
            "Access Granted: Valid ticket for Seat {}{} ({})",
            updated_ticket.row_label, updated_ticket.seat_number, updated_ticket.zone_name
        ),
        ticket: updated_ticket,
    }))
}
