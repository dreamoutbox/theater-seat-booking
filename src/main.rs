use std::sync::Arc;
use theater_seat_booking::{
    config::AppConfig,
    db::{create_pool, run_migrations},
    routes::{create_router, AppState},
    stripe::StripeService,
    worker::{run_sweeper_loop, run_worker_loop},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "theater_seat_booking=debug,tower_http=debug,info".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env();
    tracing::info!("Initializing Theater Seat Booking backend API server...");

    let pool = create_pool(&config.database_url).await?;
    tracing::info!("Connected to PostgreSQL database at {}", config.database_url);

    run_migrations(&pool).await?;
    tracing::info!("Database schema migrations verified and up to date.");

    let stripe = Arc::new(StripeService::new(config.stripe_secret_key.clone()));

    let state = AppState {
        pool: pool.clone(),
        config: config.clone(),
        stripe,
    };

    // Optionally spawn queue worker & sweeper (enabled by default unless explicitly disabled)
    let spawn_embedded_worker = std::env::var("EMBEDDED_WORKER")
        .map(|v| v != "0" && v.to_lowercase() != "false")
        .unwrap_or(true);

    if spawn_embedded_worker {
        tracing::info!("Starting embedded queue worker and hold sweeper tasks...");
        let worker_pool = pool.clone();
        let worker_config = config.clone();
        tokio::spawn(async move {
            run_worker_loop(worker_pool, worker_config).await;
        });

        let sweeper_pool = pool.clone();
        tokio::spawn(async move {
            run_sweeper_loop(sweeper_pool, 30).await;
        });
    }

    // Start the Axum HTTP API server
    let app = create_router(state);
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("🚀 Backend API Server listening on http://0.0.0.0:{}", config.port);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
