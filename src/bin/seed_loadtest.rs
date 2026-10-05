use chrono::{Duration, Utc};
use repository::{
    EventRepository, SeatRepository, ShowtimeRepository, TheaterRepository, UserRepository,
    UserRole, ZoneRepository,
};
use serde::Serialize;
use std::fs;
use std::path::Path;
use theater_seat_booking::{
    auth::hash_password,
    config::AppConfig,
    db::{create_pool, run_migrations},
};
use uuid::Uuid;

#[derive(Serialize)]
struct UserMeta {
    index: usize,
    id: Uuid,
    email: String,
}

#[derive(Serialize)]
struct TestDataMeta {
    event_id: Uuid,
    showtime_id: Uuid,
    theater_id: Uuid,
    total_seats: usize,
    user_count: usize,
    jwt_secret: String,
    users: Vec<UserMeta>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from_env();
    println!("Connecting to database: {}", config.database_url);

    let pool = create_pool(&config.database_url).await?;
    run_migrations(&pool).await?;

    println!("Seeding high-capacity load test environment...");

    // 1. Theater (physical venue)
    let theater_id = match TheaterRepository::find_by_name(&pool, "National Stadium Arena").await? {
        Some(t) => t.id,
        None => {
            let t = TheaterRepository::save(
                &pool,
                "National Stadium Arena",
                Some("123 Grand Olympic Way, Metro City"),
            )
            .await?;
            t.id
        }
    };
    println!("  Theater ID: {}", theater_id);

    // 2. Event
    let organizer_opt = UserRepository::find_by_email(&pool, "organizer@theater.local").await?;
    let organizer_id = organizer_opt.map(|u| u.id);

    let event_title = "Mega Arena Rock Festival 2026";
    let event = match EventRepository::find_by_title(&pool, event_title).await? {
        Some(ev) => ev,
        None => {
            EventRepository::save(
                &pool,
                event_title,
                4,  // max seats per booking
                10, // max seats per user
                organizer_id,
                Some("Mega Events Global"),
                Some("TAX-LOADTEST-999"),
                Some("888 Grand Olympic Way"),
                Some("billing@megaevents.local"),
                Some("+1 555-019-9999"),
            )
            .await?
        }
    };
    println!("  Event ID: {}", event.id);

    // 3. Zones & 1,200 Seats
    // 3 zones x 10 rows x 40 seats = 1,200 seats
    let zone_defs = vec![
        (
            "VIP Pit",
            5000,
            "#f59e0b",
            vec!["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"],
        ),
        (
            "Premium Circle",
            3000,
            "#6366f1",
            vec!["K", "L", "M", "N", "O", "P", "Q", "R", "S", "T"],
        ),
        (
            "General Grandstand",
            1500,
            "#14b8a6",
            vec!["U", "V", "W", "X", "Y", "Z", "AA", "BB", "CC", "DD"],
        ),
    ];

    let mut total_seats = 0;
    for (zone_name, price_cents, color, rows) in zone_defs {
        let zone_id = match ZoneRepository::find_by_event_id_and_name(&pool, event.id, zone_name)
            .await?
        {
            Some(z) => z.id,
            None => ZoneRepository::save(&pool, event.id, zone_name, price_cents, color).await?,
        };

        let mut tx = pool.begin().await?;
        for row_label in rows {
            for seat_number in 1..=40 {
                SeatRepository::upsert(&mut *tx, event.id, zone_id, row_label, seat_number).await?;
                total_seats += 1;
            }
        }
        tx.commit().await?;
    }
    println!("  Seeded {} seats across 3 zones for event.", total_seats);

    // 4. Showtime
    let starts_at = Utc::now() + Duration::days(5);
    let showtime = match ShowtimeRepository::find_all(&pool)
        .await?
        .into_iter()
        .find(|s| s.event_id == event.id && s.theater_id == theater_id)
    {
        Some(s) => s,
        None => ShowtimeRepository::save(&pool, event.id, theater_id, starts_at).await?,
    };
    println!("  Showtime ID: {}", showtime.id);

    // 5. Test Customer Accounts (1,200 users)
    let user_count = 1200;
    println!("  Seeding {} test customer accounts...", user_count);

    let default_password_hash = hash_password("customer123").map_err(|e| format!("{e}"))?;
    let mut users_meta = Vec::with_capacity(user_count);

    let mut tx = pool.begin().await?;
    for i in 1..=user_count {
        let email = format!("loadtest_user_{:04}@loadtest.local", i);
        let user_uuid = Uuid::from_u128(0x00000000_0000_0000_0002_000000000000 | (i as u128));

        sqlx::query(
            "INSERT INTO users (id, email, password_hash, role)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (email) DO UPDATE SET password_hash = EXCLUDED.password_hash",
        )
        .bind(user_uuid)
        .bind(&email)
        .bind(&default_password_hash)
        .bind(UserRole::Customer)
        .execute(&mut *tx)
        .await?;

        users_meta.push(UserMeta {
            index: i,
            id: user_uuid,
            email,
        });
    }
    tx.commit().await?;
    println!("  Created/updated {} load test users.", user_count);

    // 6. Write test metadata file to loadtest/test-data.json
    let loadtest_dir = Path::new("loadtest");
    if !loadtest_dir.exists() {
        fs::create_dir_all(loadtest_dir)?;
    }

    let meta = TestDataMeta {
        event_id: event.id,
        showtime_id: showtime.id,
        theater_id,
        total_seats,
        user_count,
        jwt_secret: config.jwt_secret.clone(),
        users: users_meta,
    };

    let meta_json = serde_json::to_string_pretty(&meta)?;
    let meta_path = loadtest_dir.join("test-data.json");
    fs::write(&meta_path, meta_json)?;
    println!("  Metadata saved to: {}", meta_path.display());

    println!("\nHigh-capacity load test environment ready!");
    Ok(())
}
