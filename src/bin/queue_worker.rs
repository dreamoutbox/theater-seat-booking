use theater_seat_booking::{
    config::AppConfig,
    db::{create_pool, run_migrations},
    worker::{run_sweeper_loop, run_worker_loop},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "theater_seat_booking=debug,queue_worker=debug,info".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env();
    tracing::info!("🚀 Starting Dedicated Theater Booking Queue Worker...");
    tracing::info!(
        database_url = %config.database_url,
        poll_interval_ms = config.poll_interval_ms,
        hold_ttl_seconds = config.hold_ttl_seconds,
        "Worker configuration loaded"
    );

    let pool = create_pool(&config.database_url).await?;
    run_migrations(&pool).await?;
    tracing::info!("Database migrations verified.");

    let worker_pool = pool.clone();
    let worker_config = config.clone();
    let worker_handle = tokio::spawn(async move {
        run_worker_loop(worker_pool, worker_config).await;
    });

    let sweeper_pool = pool.clone();
    let sweeper_handle = tokio::spawn(async move {
        run_sweeper_loop(sweeper_pool, 30).await;
    });

    tracing::info!("Worker active and listening for partition claims. Press Ctrl+C to stop.");

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received shutdown signal. Stopping worker gracefully...");
        }
        _ = worker_handle => {
            tracing::warn!("Worker task exited unexpectedly.");
        }
        _ = sweeper_handle => {
            tracing::warn!("Sweeper task exited unexpectedly.");
        }
    }

    Ok(())
}
