use theater_seat_booking::{config::AppConfig, db::{create_pool, run_migrations}};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "theater_seat_booking=debug,tower_http=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env();
    tracing::info!("Starting Theater Seat Booking Service on port {}", config.port);

    let pool = create_pool(&config.database_url).await?;
    run_migrations(&pool).await?;
    tracing::info!("Database migrations applied.");

    Ok(())
}
