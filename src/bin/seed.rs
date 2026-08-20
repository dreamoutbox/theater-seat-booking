use chrono::{Duration, Utc};
use repository::{
    EventRepository, GatekeeperRepository, SeatRepository, ShowtimeRepository, TheaterRepository,
    UserRepository, UserRole, ZoneRepository,
};
use sqlx::PgPool;
use theater_seat_booking::{
    auth::hash_password,
    config::AppConfig,
    db::{create_pool, run_migrations},
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
    println!("  - Gatekeeper:      gatekeeper@theater.local / gatekeeper123");
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
        (
            "gatekeeper@theater.local",
            "gatekeeper123",
            UserRole::Gatekeeper,
        ),
        ("alice@customer.local", "customer123", UserRole::Customer),
        ("bob@customer.local", "customer123", UserRole::Customer),
    ];

    for (email, password, role) in users {
        let exists = UserRepository::exists_by_email(pool, email).await?;

        if !exists {
            let password_hash = hash_password(password).map_err(|e| format!("{e}"))?;
            UserRepository::save(pool, email, &password_hash, role).await?;
            println!("  Created user: {} ({:?})", email, role);
        } else {
            println!("  User already exists: {}", email);
        }
    }

    Ok(())
}

async fn seed_theater_and_shows(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Theater
    let theater_id = match TheaterRepository::find_by_name(pool, "Grand Horizon Cinema").await? {
        Some(t) => t.id,
        None => {
            let t = TheaterRepository::save(
                pool,
                "Grand Horizon Cinema",
                Some("123 Celestial Boulevard, Metro City"),
            )
            .await?;
            t.id
        }
    };
    println!("  Theater ID: {}", theater_id);

    // 2. Zones
    let vip_zone_id: Uuid = get_or_create_zone(pool, theater_id, "VIP", 3000, "#f59e0b").await?;
    let premium_zone_id: Uuid =
        get_or_create_zone(pool, theater_id, "Premium", 2000, "#6366f1").await?;
    let standard_zone_id: Uuid =
        get_or_create_zone(pool, theater_id, "Standard", 1000, "#14b8a6").await?;

    println!("  Zones configured: VIP ($30.00), Premium ($20.00), Standard ($10.00)");

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
            let seat_exists = SeatRepository::exists_by_theater_id_and_row_label_and_seat_number(
                pool,
                theater_id,
                row_label,
                seat_number,
            )
            .await?;

            if !seat_exists {
                SeatRepository::upsert(pool, theater_id, zone_id, row_label, seat_number).await?;
                seats_created += 1;
            }
        }
    }
    println!(
        "  Seats seeded: {} new seats created (50 total in grid)",
        seats_created
    );

    // 4. Events
    let organizer_opt = UserRepository::find_by_email(pool, "organizer@theater.local").await?;
    let organizer_id = organizer_opt.map(|u| u.id);

    let event1_id: Uuid =
        get_or_create_event(pool, "Interstellar Live Symphony & Screening", organizer_id).await?;
    let event2_id: Uuid =
        get_or_create_event(pool, "Cyberpunk 2099: Orchestral Experience", organizer_id).await?;

    // Assign gatekeeper to both events
    if let Some(gatekeeper) =
        UserRepository::find_by_email(pool, "gatekeeper@theater.local").await?
    {
        GatekeeperRepository::assign(pool, event1_id, gatekeeper.id, organizer_id).await?;
        GatekeeperRepository::assign(pool, event2_id, gatekeeper.id, organizer_id).await?;
    }

    // 5. Showtimes
    let now = Utc::now();
    let showtime_times = vec![
        (event1_id, now + Duration::hours(3)),
        (event1_id, now + Duration::days(1) + Duration::hours(19)),
        (event2_id, now + Duration::days(2) + Duration::hours(20)),
        (event2_id, now + Duration::days(3) + Duration::hours(18)),
    ];

    for (event_id, starts_at) in showtime_times {
        let exists =
            ShowtimeRepository::exists_by_theater_and_starts_at(pool, theater_id, starts_at)
                .await?;

        if !exists {
            ShowtimeRepository::save(pool, event_id, theater_id, starts_at).await?;
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
    color: &str,
) -> Result<Uuid, sqlx::Error> {
    if let Some(zone) = ZoneRepository::find_by_theater_id_and_name(pool, theater_id, name).await? {
        Ok(zone.id)
    } else {
        ZoneRepository::save(pool, theater_id, name, price_cents, color).await
    }
}

async fn get_or_create_event(
    pool: &PgPool,
    title: &str,
    owner_id: Option<Uuid>,
) -> Result<Uuid, sqlx::Error> {
    if let Some(event) = EventRepository::find_by_title(pool, title).await? {
        Ok(event.id)
    } else {
        let event = EventRepository::save(
            pool,
            title,
            6,
            10,
            owner_id,
            Some("DreamCinema Live Productions Ltd."),
            Some("TAX-8842-TSB-2026"),
            Some("100 Theater Way, Suite 400, Los Angeles, CA 90028"),
            Some("billing@dreamcinema.com"),
            Some("+1 (555) 019-2834"),
        )
        .await?;
        Ok(event.id)
    }
}
