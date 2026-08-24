use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::Utc;
use repository::{
    AnalyticsRepository, EventFinancialReport, EventFinancialReportDetail, EventRepository,
    ReportRepository, UserRepository, UserRole,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    auth::{AuthenticatedUser, check_role},
    routes::AppState,
};

const ORGANIZER_COOLDOWN_SECONDS: i64 = 300; // 5 minutes (300s) for event organizers; 0s for Admin/Staff

#[derive(Debug, Serialize)]
pub struct EventReportsListResponse {
    pub reports: Vec<EventFinancialReportDetail>,
    pub can_generate: bool,
    pub cooldown_remaining_seconds: i64,
    pub latest_report_at: Option<chrono::DateTime<Utc>>,
}

/// GET /api/events/:id/reports
pub async fn list_event_reports(
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(event_id): Path<Uuid>,
) -> Result<Json<EventReportsListResponse>, (StatusCode, Json<serde_json::Value>)> {
    check_role(
        &auth_user,
        &[
            UserRole::Admin,
            UserRole::Staff,
            UserRole::EventOrganizer,
        ],
    )
    .map_err(|_| {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Unauthorized to access reports" })),
        )
    })?;

    // Check event existence and ownership if EventOrganizer
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

    if auth_user.role == UserRole::EventOrganizer && event.owner_id != Some(auth_user.id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Access denied to event reports" })),
        ));
    }

    let reports = ReportRepository::find_by_event_id(&state.pool, event_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    let is_admin_or_staff = auth_user.role == UserRole::Admin || auth_user.role == UserRole::Staff;
    let now = Utc::now();
    let (can_generate, cooldown_remaining_seconds, latest_report_at) = if is_admin_or_staff {
        (true, 0, reports.first().map(|r| r.generated_at))
    } else if let Some(latest) = reports.first() {
        let elapsed = (now - latest.generated_at).num_seconds();
        if elapsed < ORGANIZER_COOLDOWN_SECONDS {
            (false, ORGANIZER_COOLDOWN_SECONDS - elapsed, Some(latest.generated_at))
        } else {
            (true, 0, Some(latest.generated_at))
        }
    } else {
        (true, 0, None)
    };

    Ok(Json(EventReportsListResponse {
        reports,
        can_generate,
        cooldown_remaining_seconds,
        latest_report_at,
    }))
}

/// POST /api/events/:id/reports/generate
pub async fn generate_event_report(
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(event_id): Path<Uuid>,
) -> Result<Json<EventFinancialReportDetail>, (StatusCode, Json<serde_json::Value>)> {
    check_role(
        &auth_user,
        &[
            UserRole::Admin,
            UserRole::Staff,
            UserRole::EventOrganizer,
        ],
    )
    .map_err(|_| {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Unauthorized to generate reports" })),
        )
    })?;

    // Check event existence and ownership
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

    if auth_user.role == UserRole::EventOrganizer && event.owner_id != Some(auth_user.id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Access denied to generate event report" })),
        ));
    }

    let is_admin_or_staff = auth_user.role == UserRole::Admin || auth_user.role == UserRole::Staff;
    let now = Utc::now();

    // Check cooldown for non-admin/staff
    if !is_admin_or_staff {
        let latest_report = ReportRepository::find_latest_by_event_id(&state.pool, event_id)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
            })?;

        if let Some(ref latest) = latest_report {
            let elapsed = (now - latest.generated_at).num_seconds();
            if elapsed < ORGANIZER_COOLDOWN_SECONDS {
                let remaining = ORGANIZER_COOLDOWN_SECONDS - elapsed;
                return Err((
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(serde_json::json!({
                        "error": format!("Report generation is on cooldown. Please wait {remaining} seconds before generating a new report."),
                        "cooldown_remaining_seconds": remaining,
                        "can_generate": false
                    })),
                ));
            }
        }
    }

    // Compute live metrics & showtimes snapshot
    let metrics = AnalyticsRepository::get_event_dashboard_metrics(
        &state.pool,
        Some(event_id),
        None,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to compute metrics: {e}") })),
        )
    })?;

    let showtimes = AnalyticsRepository::get_showtime_dashboard_breakdown(
        &state.pool,
        Some(event_id),
        None,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to compute showtimes: {e}") })),
        )
    })?;

    // Organizer email lookup
    let organizer_email = if let Some(owner_id) = event.owner_id {
        UserRepository::find_by_id(&state.pool, owner_id)
            .await
            .ok()
            .flatten()
            .map(|u| u.email)
            .unwrap_or_else(|| "Unknown Organizer".to_string())
    } else {
        "Primary Cinema Administration".to_string()
    };

    // Calculate total report count for sequence number
    let count = ReportRepository::count_by_event_id(&state.pool, event_id)
        .await
        .unwrap_or(0);

    let year = now.format("%Y");
    let month = now.format("%m");
    let seq_num = count + 1;
    let statement_ref = format!("FIN-{year}-{month}-{seq_num:03}");

    // Tax calculation: standard 10% withholding
    let tax_withholding_cents = (metrics.net_profit_cents * 10) / 100;
    let net_settlement_cents = metrics.net_profit_cents - tax_withholding_cents;

    let snapshot_data = serde_json::json!({
        "statement_ref": statement_ref,
        "generated_at": now,
        "generated_by": auth_user.id,
        "generated_by_email": auth_user.email,
        "organizer_email": organizer_email,
        "event": {
            "id": event.id,
            "title": event.title,
            "owner_id": event.owner_id,
            "max_seats_per_booking": event.max_seats_per_booking,
            "max_seats_per_user": event.max_seats_per_user,
        },
        "metrics": {
            "gross_sales_cents": metrics.gross_sales_cents,
            "total_refunds_cents": metrics.total_refunds_cents,
            "net_profit_cents": metrics.net_profit_cents,
            "total_tickets_created": metrics.total_tickets_created,
            "active_tickets": metrics.active_tickets,
            "used_tickets": metrics.used_tickets,
            "refunded_tickets": metrics.refunded_tickets,
            "attendance_rate_percent": metrics.attendance_rate_percent,
        },
        "tax_summary": {
            "tax_rate_percent": 10,
            "tax_withholding_cents": tax_withholding_cents,
            "net_settlement_cents": net_settlement_cents,
        },
        "showtimes": showtimes,
    });

    let new_report = EventFinancialReport {
        id: Uuid::new_v4(),
        event_id,
        statement_ref: statement_ref.clone(),
        generated_by: auth_user.id,
        generated_at: now,
        gross_sales_cents: metrics.gross_sales_cents,
        total_refunds_cents: metrics.total_refunds_cents,
        net_profit_cents: metrics.net_profit_cents,
        tax_withholding_cents,
        net_settlement_cents,
        tickets_issued: metrics.total_tickets_created as i32,
        tickets_scanned: metrics.used_tickets as i32,
        tickets_refunded: metrics.refunded_tickets as i32,
        attendance_rate_percent: metrics.attendance_rate_percent.round() as i32,
        report_data: snapshot_data,
    };

    let saved = ReportRepository::save(&state.pool, &new_report)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to save report: {e}") })),
            )
        })?;

    let report_detail = EventFinancialReportDetail {
        id: saved.id,
        event_id: saved.event_id,
        statement_ref: saved.statement_ref.clone(),
        generated_by: saved.generated_by,
        generated_by_email: Some(auth_user.email),
        generated_at: saved.generated_at,
        gross_sales_cents: saved.gross_sales_cents,
        total_refunds_cents: saved.total_refunds_cents,
        net_profit_cents: saved.net_profit_cents,
        tax_withholding_cents: saved.tax_withholding_cents,
        net_settlement_cents: saved.net_settlement_cents,
        tickets_issued: saved.tickets_issued,
        tickets_scanned: saved.tickets_scanned,
        tickets_refunded: saved.tickets_refunded,
        attendance_rate_percent: saved.attendance_rate_percent,
        report_data: saved.report_data,
    };

    // Generate PDF & upload to S3
    if let Ok(pdf_bytes) = crate::pdf::PdfService::generate_financial_report_pdf(
        &report_detail,
        &event.title,
        &showtimes,
    ) {
        let s3_key = format!("reports/{}/{}.pdf", event_id, statement_ref);
        if let Err(e) = state.s3.upload_file(&s3_key, pdf_bytes, "application/pdf").await {
            tracing::warn!("Failed to upload report PDF to S3: {e}");
        }
    }

    Ok(Json(report_detail))
}

/// GET /api/reports/:id
pub async fn get_report_by_id(
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(report_id): Path<Uuid>,
) -> Result<Json<EventFinancialReportDetail>, (StatusCode, Json<serde_json::Value>)> {
    check_role(
        &auth_user,
        &[
            UserRole::Admin,
            UserRole::Staff,
            UserRole::EventOrganizer,
        ],
    )
    .map_err(|_| {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Unauthorized to view report" })),
        )
    })?;

    let report = ReportRepository::find_by_id(&state.pool, report_id)
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
                Json(serde_json::json!({ "error": "Report not found" })),
            )
        })?;

    // Check organizer access if EventOrganizer
    if auth_user.role == UserRole::EventOrganizer {
        let event = EventRepository::find_by_id(&state.pool, report.event_id)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
            })?;

        if let Some(ev) = event {
            if ev.owner_id != Some(auth_user.id) {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({ "error": "Access denied to view report" })),
                ));
            }
        }
    }

    Ok(Json(report))
}

/// GET /api/reports/:id/pdf
pub async fn download_report_pdf(
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(report_id): Path<Uuid>,
) -> Result<axum::response::Response, (StatusCode, Json<serde_json::Value>)> {
    check_role(
        &auth_user,
        &[
            UserRole::Admin,
            UserRole::Staff,
            UserRole::EventOrganizer,
        ],
    )
    .map_err(|_| {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Unauthorized to download report PDF" })),
        )
    })?;

    let report = ReportRepository::find_by_id(&state.pool, report_id)
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
                Json(serde_json::json!({ "error": "Report not found" })),
            )
        })?;

    if auth_user.role == UserRole::EventOrganizer {
        let event = EventRepository::find_by_id(&state.pool, report.event_id)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
            })?;

        if let Some(ev) = event {
            if ev.owner_id != Some(auth_user.id) {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({ "error": "Access denied to download report" })),
                ));
            }
        }
    }

    let s3_key = format!("reports/{}/{}.pdf", report.event_id, report.statement_ref);

    // Try fetching from S3 or generate on-the-fly and save
    let pdf_bytes = match state.s3.get_file(&s3_key).await {
        Ok(bytes) => bytes,
        Err(_) => {
            let event = EventRepository::find_by_id(&state.pool, report.event_id)
                .await
                .ok()
                .flatten();
            let event_title = event.map(|e| e.title).unwrap_or_else(|| "Event".to_string());
            let showtimes = AnalyticsRepository::get_showtime_dashboard_breakdown(
                &state.pool,
                Some(report.event_id),
                None,
            )
            .await
            .unwrap_or_default();

            let generated = crate::pdf::PdfService::generate_financial_report_pdf(
                &report,
                &event_title,
                &showtimes,
            )
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Failed to generate PDF: {e}") })),
                )
            })?;

            let _ = state.s3.upload_file(&s3_key, generated.clone(), "application/pdf").await;
            generated
        }
    };

    use axum::http::header;
    use axum::response::IntoResponse;

    let disposition = format!("inline; filename=\"{}.pdf\"", report.statement_ref);
    let response = (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/pdf"),
            (header::CONTENT_DISPOSITION, &disposition),
        ],
        pdf_bytes,
    )
        .into_response();

    Ok(response)
}
