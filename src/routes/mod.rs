pub mod auth;
pub mod booking;
pub mod order;
pub mod theater;
pub mod webhook;

use axum::{
    Router,
    http::{Method, header},
    routing::{get, post},
};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use crate::{config::AppConfig, stripe::StripeService};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: AppConfig,
    pub stripe: Arc<StripeService>,
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
        .route("/theaters", get(theater::list_theaters))
        .route("/events", get(theater::list_events).post(theater::create_event))
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
        .route("/theaters/{id}/seats", get(theater::get_theater_seats))
        // Booking Queue
        .route("/bookings", post(booking::enqueue_booking))
        .route("/bookings/{uuid}", get(booking::get_booking_status))
        .route("/bookings/release", post(booking::release_hold))
        // Orders
        .route("/orders", post(order::create_order).get(order::list_orders))
        .route("/orders/{id}", get(order::get_order))
        .route("/orders/{id}/confirm-mock", post(order::confirm_mock_order))
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
