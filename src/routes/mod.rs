pub mod analytics;
pub mod auth;
pub mod booking;
pub mod gatekeeper;
pub mod order;
pub mod report;
pub mod theater;
pub mod ticket;
pub mod webhook;

use axum::{
    Router,
    http::{Method, header},
    routing::{delete, get, post},
};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use crate::{config::AppConfig, email::EmailService, s3::S3Service, stripe::StripeService};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: AppConfig,
    pub stripe: Arc<StripeService>,
    pub s3: Arc<S3Service>,
    pub email: Arc<EmailService>,
}

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::mirror_request())
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::ACCEPT,
            header::CONTENT_TYPE,
            header::COOKIE,
            header::SET_COOKIE,
        ])
        .allow_credentials(true);

    let api_routes = Router::new()
        // Auth
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        .route("/me", get(auth::me))
        // Catalog & Management
        .route(
            "/theaters",
            get(theater::list_theaters).post(theater::create_theater),
        )
        .route(
            "/theaters/{id}",
            axum::routing::put(theater::update_theater).delete(theater::delete_theater),
        )
        .route(
            "/events",
            get(theater::list_events).post(theater::create_event),
        )
        .route(
            "/events/{id}",
            axum::routing::put(theater::update_event).delete(theater::delete_event),
        )
        .route(
            "/showtimes",
            get(theater::list_showtimes).post(theater::create_showtime),
        )
        .route(
            "/showtimes/{id}",
            axum::routing::put(theater::update_showtime).delete(theater::delete_showtime),
        )
        .route("/showtimes/{id}/seats", get(theater::get_showtime_seats))
        .route("/theaters/{id}/seats", get(theater::get_theater_seats))
        .route(
            "/theaters/{id}/layout",
            get(theater::get_theater_layout).put(theater::save_theater_layout),
        )
        .route(
            "/theaters/{id}/booked-seats",
            get(theater::get_theater_booked_seats),
        )
        // Booking Queue
        .route("/bookings", post(booking::enqueue_booking))
        .route("/bookings/{uuid}", get(booking::get_booking_status))
        .route("/bookings/release", post(booking::release_hold))
        .route("/hold", post(booking::enqueue_booking))
        .route("/release-hold", post(booking::release_hold))
        .route("/booking/status/{uuid}", get(booking::get_booking_status))
        // Orders
        .route("/orders", post(order::create_order).get(order::list_orders))
        .route("/orders/{id}", get(order::get_order))
        .route(
            "/orders/{id}/checkout-session",
            post(order::create_checkout_session),
        )
        .route("/orders/{id}/refund", post(order::refund_order))
        .route("/orders/{id}/refunds", get(order::get_order_refunds))
        .route("/orders/{id}/change-seat", post(order::change_seat))
        .route(
            "/orders/{id}/seat-changes",
            get(order::get_order_seat_changes),
        )
        .route("/orders/{id}/tickets", get(ticket::list_order_tickets))
        .route("/orders/{id}/pdf", get(order::download_order_pdf))
        // Tickets & Scanning
        .route("/tickets/scan", post(ticket::scan_ticket))
        .route("/tickets/{code}", get(ticket::get_ticket_by_code))
        // Analytics
        .route("/analytics/dashboard", get(analytics::get_dashboard_data))
        // Reports
        .route("/events/{id}/reports", get(report::list_event_reports))
        .route(
            "/events/{id}/reports/generate",
            post(report::generate_event_report),
        )
        .route("/reports/{id}", get(report::get_report_by_id))
        .route("/reports/{id}/pdf", get(report::download_report_pdf))
        // Gatekeepers
        .route(
            "/events/{id}/gatekeepers",
            get(gatekeeper::list_event_gatekeepers).post(gatekeeper::create_and_assign_gatekeeper),
        )
        .route(
            "/events/{id}/gatekeepers/{user_id}",
            delete(gatekeeper::remove_gatekeeper_from_event),
        )
        .route(
            "/gatekeeper/events",
            get(gatekeeper::list_my_assigned_events),
        )
        // Webhooks
        .route("/webhooks/stripe", post(webhook::stripe_webhook));

    let frontend_dist = "frontend/dist";
    let serve_dir = ServeDir::new(frontend_dist)
        .not_found_service(ServeFile::new(format!("{frontend_dist}/index.html")));

    Router::new()
        .nest("/api", api_routes)
        .fallback_service(serve_dir)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
