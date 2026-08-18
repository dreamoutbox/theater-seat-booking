use chrono::{Duration, Utc};
use sqlx::PgPool;
use theater_seat_booking::{
    auth::hash_password,
    config::AppConfig,
    db::{create_pool, run_migrations},
    models::UserRole,
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from_env();
    println!("Connecting to database: {}", config.database_url);

    let pool = create_pool(&config.database_url).await?;
    println!("Running database migrations...");
    run_migrations(&pool).await?;
    println!("Migrations applied successfully.");

    println!("Seeding users...");
    seed_users(&pool).await?;

    println!("Seeding theater, zones, and seats...");
    seed_theater_and_shows(&pool).await?;

    println!("\n========================================================");
    println!("Seed completed successfully!");
    println!("Demo Accounts (Password for all: <role>123 or customer123):");
    println!("  - Admin:           admin@theater.local      / admin123");
    println!("  - Staff:           staff@theater.local      / staff123");
    println!("  - Event Organizer: organizer@theater.local  / organizer123");
    println!("  - Customer 1:      alice@customer.local     / customer123");
    println!("  - Customer 2:      bob@customer.local       / customer123");
    println!("========================================================\n");

    Ok(())
}

async fn seed_users(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let users = vec![
        ("admin@theater.local", "admin123", UserRole::Admin),
        ("staff@theater.local", "staff123", UserRole::Staff),
        (
            "organizer@theater.local",
            "organizer123",
            UserRole::EventOrganizer,
        ),
        ("alice@customer.local", "customer123", UserRole::Customer),
        ("bob@customer.local", "customer123", UserRole::Customer),
    ];

    for (email, password, role) in users {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
                .bind(email)
                .fetch_one(pool)
                .await?;

        if !exists {
            let password_hash = hash_password(password).map_err(|e| format!("{e}"))?;
            sqlx::query("INSERT INTO users (email, password_hash, role) VALUES ($1, $2, $3)")
                .bind(email)
                .bind(password_hash)
                .bind(role)
                .execute(pool)
                .await?;
            println!("  Created user: {} ({:?})", email, role);
        } else {
            println!("  User already exists: {}", email);
        }
    }

    Ok(())
}

async fn seed_theater_and_shows(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Theater
    let theater_id: Uuid = match sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM theaters WHERE name = 'Grand Horizon Cinema' LIMIT 1",
    )
    .fetch_optional(pool)
    .await?
    {
        Some(id) => id,
        None => {
            sqlx::query_scalar("INSERT INTO theaters (name, address) VALUES ($1, $2) RETURNING id")
                .bind("Grand Horizon Cinema")
                .bind("123 Celestial Boulevard, Metro City")
                .fetch_one(pool)
                .await?
        }
    };
    println!("  Theater ID: {}", theater_id);

    // 2. Zones
    let vip_zone_id: Uuid = get_or_create_zone(pool, theater_id, "VIP Lounge", 2500).await?;
    let premium_zone_id: Uuid =
        get_or_create_zone(pool, theater_id, "Premium Stalls", 1800).await?;
    let standard_zone_id: Uuid =
        get_or_create_zone(pool, theater_id, "Standard Circle", 1200).await?;

    println!("  Zones configured: VIP ($25.00), Premium ($18.00), Standard ($12.00)");

    // 3. Seats (5 rows: A=VIP, B & C=Premium, D & E=Standard; 10 seats each = 50 seats)
    let rows = vec![
        ("A", vip_zone_id),
        ("B", premium_zone_id),
        ("C", premium_zone_id),
        ("D", standard_zone_id),
        ("E", standard_zone_id),
    ];

    let mut seats_created = 0;
    for (row_label, zone_id) in rows {
        for seat_number in 1..=10 {
            let seat_exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM seats WHERE theater_id = $1 AND row_label = $2 AND seat_number = $3)",
            )
            .bind(theater_id)
            .bind(row_label)
            .bind(seat_number)
            .fetch_one(pool)
            .await?;

            if !seat_exists {
                sqlx::query(
                    "INSERT INTO seats (theater_id, zone_id, row_label, seat_number) VALUES ($1, $2, $3, $4)",
                )
                .bind(theater_id)
                .bind(zone_id)
                .bind(row_label)
                .bind(seat_number)
                .execute(pool)
                .await?;
                seats_created += 1;
            }
        }
    }
    println!(
        "  Seats seeded: {} new seats created (50 total in grid)",
        seats_created
    );

    // 4. Events
    let event1_id: Uuid =
        get_or_create_event(pool, "Interstellar Live Symphony & Screening").await?;
    let event2_id: Uuid =
        get_or_create_event(pool, "Cyberpunk 2099: Orchestral Experience").await?;

    // 5. Showtimes
    let now = Utc::now();
    let showtime_times = vec![
        (event1_id, now + Duration::hours(3)),
        (event1_id, now + Duration::days(1) + Duration::hours(19)),
        (event2_id, now + Duration::days(2) + Duration::hours(20)),
        (event2_id, now + Duration::days(3) + Duration::hours(18)),
    ];

    for (event_id, starts_at) in showtime_times {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM showtimes WHERE event_id = $1 AND theater_id = $2 AND starts_at = $3)",
        )
        .bind(event_id)
        .bind(theater_id)
        .bind(starts_at)
        .fetch_one(pool)
        .await?;

        if !exists {
            sqlx::query(
                "INSERT INTO showtimes (event_id, theater_id, starts_at) VALUES ($1, $2, $3)",
            )
            .bind(event_id)
            .bind(theater_id)
            .bind(starts_at)
            .execute(pool)
            .await?;
        }
    }
    println!("  Showtimes created.");

    Ok(())
}

async fn get_or_create_zone(
    pool: &PgPool,
    theater_id: Uuid,
    name: &str,
    price_cents: i64,
) -> Result<Uuid, sqlx::Error> {
    if let Some(id) = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM zones WHERE theater_id = $1 AND name = $2 LIMIT 1",
    )
    .bind(theater_id)
    .bind(name)
    .fetch_optional(pool)
    .await?
    {
        Ok(id)
    } else {
        sqlx::query_scalar(
            "INSERT INTO zones (theater_id, name, price_cents) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(theater_id)
        .bind(name)
        .bind(price_cents)
        .fetch_one(pool)
        .await
    }
}

async fn get_or_create_event(pool: &PgPool, title: &str) -> Result<Uuid, sqlx::Error> {
    if let Some(id) =
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM events WHERE title = $1 LIMIT 1")
            .bind(title)
            .fetch_optional(pool)
            .await?
    {
        Ok(id)
    } else {
        sqlx::query_scalar("INSERT INTO events (title) VALUES ($1) RETURNING id")
            .bind(title)
            .fetch_one(pool)
            .await
    }
}
