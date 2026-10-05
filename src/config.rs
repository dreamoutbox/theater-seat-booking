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
    pub stripe_currency: String,
    pub s3_endpoint: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub s3_bucket: String,
    pub s3_region: String,
    pub smtp_host: Option<String>,
    pub stripe_skip_payment: bool,
    pub smtp_port: u16,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_from_email: String,
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
            stripe_currency: parse_currency(env::var("STRIPE_CURRENCY").ok()),
            stripe_skip_payment: parse_bool_flag(env::var("STRIPE_SKIP_PAYMENT").ok()),
            s3_endpoint: env::var("S3_ENDPOINT")
                .unwrap_or_else(|_| "http://127.0.0.1:9000".to_string()),
            s3_access_key: env::var("S3_ACCESS_KEY").unwrap_or_else(|_| "miniominio".to_string()),
            s3_secret_key: env::var("S3_SECRET_KEY").unwrap_or_else(|_| "miniominio".to_string()),
            s3_bucket: env::var("S3_BUCKET").unwrap_or_else(|_| "theater-documents".to_string()),
            s3_region: env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            smtp_host: env::var("SMTP_HOST")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            smtp_port: env::var("SMTP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(587),
            smtp_username: env::var("SMTP_USERNAME")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            smtp_password: env::var("SMTP_PASSWORD")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            smtp_from_email: env::var("SMTP_FROM_EMAIL")
                .unwrap_or_else(|_| "Theater Booking <noreply@theaterbooking.local>".to_string()),
        }
    }
}

pub(crate) fn parse_currency(raw: Option<String>) -> String {
    raw.map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "thb".to_string())
}

pub(crate) fn parse_bool_flag(raw: Option<String>) -> bool {
    raw.map(|s| {
        let trimmed = s.trim();
        trimmed.eq_ignore_ascii_case("true") || trimmed == "1"
    })
    .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_currency_defaults_to_thb() {
        assert_eq!(parse_currency(None), "thb");
        assert_eq!(parse_currency(Some(String::new())), "thb");
        assert_eq!(parse_currency(Some("   ".to_string())), "thb");
    }

    #[test]
    fn test_parse_currency_normalizes_to_lowercase() {
        assert_eq!(parse_currency(Some("USD".to_string())), "usd");
        assert_eq!(parse_currency(Some("  EUR  ".to_string())), "eur");
        assert_eq!(parse_currency(Some("thb".to_string())), "thb");
        assert_eq!(parse_currency(Some("THB".to_string())), "thb");
    }

    #[test]
    fn test_parse_bool_flag() {
        assert!(parse_bool_flag(Some("true".to_string())));
        assert!(parse_bool_flag(Some("TRUE".to_string())));
        assert!(parse_bool_flag(Some("1".to_string())));
        assert!(!parse_bool_flag(Some("false".to_string())));
        assert!(!parse_bool_flag(Some("0".to_string())));
        assert!(!parse_bool_flag(Some("".to_string())));
        assert!(!parse_bool_flag(None));
    }
}
