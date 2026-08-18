use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub port: u16,
    pub hold_ttl_seconds: i64,
    pub poll_interval_ms: u64,
    pub max_bookings_per_user: i64,
    pub stripe_secret_key: String,
    pub stripe_webhook_secret: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let _ = dotenvy::dotenv();

        Self {
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:example@localhost:5432/postgres".to_string()
            }),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "theater_booking_poc_jwt_secret_key_default_12345".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            hold_ttl_seconds: env::var("HOLD_TTL_SECONDS")
                .ok()
                .and_then(|t| t.parse().ok())
                .unwrap_or(600), // 10 minutes default
            poll_interval_ms: env::var("POLL_INTERVAL_MS")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(1000), // 1s default
            max_bookings_per_user: env::var("MAX_BOOKINGS_PER_USER")
                .ok()
                .and_then(|m| m.parse().ok())
                .unwrap_or(4), // 4 default
            stripe_secret_key: env::var("STRIPE_SECRET_KEY")
                .unwrap_or_else(|_| "sk_test_mock_key_for_poc".to_string()),
            stripe_webhook_secret: env::var("STRIPE_WEBHOOK_SECRET")
                .unwrap_or_else(|_| "whsec_mock_key_for_poc".to_string()),
        }
    }
}
