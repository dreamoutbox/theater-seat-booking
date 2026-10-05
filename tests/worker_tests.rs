use chrono::{Duration, Utc};
use theater_seat_booking::{
    config::AppConfig,
    db::create_pool,
    models::{BookingQueueItem, SeatBooking},
    worker::process_partition,
};
use uuid::Uuid;

async fn setup_test_db() -> Result<sqlx::PgPool, Box<dyn std::error::Error>> {
    let config = AppConfig::from_env();
    let pool = create_pool(&config.database_url).await?;
    theater_seat_booking::db::run_migrations(&pool).await?;
    Ok(pool)
}

#[tokio::test]
async fn test_scenario_1_same_seat_same_showtime_conflict() -> Result<(), Box<dyn std::error::Error>>
{
    let pool = setup_test_db().await?;

    // Create unique test users
    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, email, password_hash, role) VALUES ($1, $2, 'hash', 'customer'), ($3, $4, 'hash', 'customer')")
        .bind(user1_id)
        .bind(format!("user1_{}@test.com", user1_id))
        .bind(user2_id)
        .bind(format!("user2_{}@test.com", user2_id))
        .execute(&pool)
        .await?;

    // Create theater, zone, seat, event, and showtime
    let theater_id = Uuid::new_v4();
    sqlx::query("INSERT INTO theaters (id, name) VALUES ($1, 'Test Theater 1')")
        .bind(theater_id)
        .execute(&pool)
        .await?;

    let event_id = Uuid::new_v4();
    sqlx::query("INSERT INTO events (id, title) VALUES ($1, 'Event 1')")
        .bind(event_id)
        .execute(&pool)
        .await?;

    let zone_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO zones (id, event_id, theater_id, name, price_cents) VALUES ($1, $2, $3, 'Zone A', 2000)",
    )
    .bind(zone_id)
    .bind(event_id)
    .bind(theater_id)
    .execute(&pool)
    .await?;

    let seat_id = Uuid::new_v4();
    sqlx::query("INSERT INTO seats (id, event_id, theater_id, zone_id, row_label, seat_number) VALUES ($1, $2, $3, $4, 'Z', 99)")
        .bind(seat_id)
        .bind(event_id)
        .bind(theater_id)
        .bind(zone_id)
        .execute(&pool)
        .await?;

    let showtime_id = Uuid::new_v4();
    let starts_at = Utc::now() + Duration::days(5);
    sqlx::query(
        "INSERT INTO showtimes (id, event_id, theater_id, starts_at) VALUES ($1, $2, $3, $4)",
    )
    .bind(showtime_id)
    .bind(event_id)
    .bind(theater_id)
    .bind(starts_at)
    .execute(&pool)
    .await?;

    // Enqueue two booking requests for the exact same seat and showtime
    let bq1_id = Uuid::new_v4();
    let bq2_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO booking_queue (id, user_id, showtime_id, zone_id, seat_id, status, created_at)
         VALUES ($1, $2, $3, $4, $5, 'pending', now())",
    )
    .bind(bq1_id)
    .bind(user1_id)
    .bind(showtime_id)
    .bind(zone_id)
    .bind(seat_id)
    .execute(&pool)
    .await?;

    sqlx::query(
        "INSERT INTO booking_queue (id, user_id, showtime_id, zone_id, seat_id, status, created_at)
         VALUES ($1, $2, $3, $4, $5, 'pending', now() + interval '10 milliseconds')",
    )
    .bind(bq2_id)
    .bind(user2_id)
    .bind(showtime_id)
    .bind(zone_id)
    .bind(seat_id)
    .execute(&pool)
    .await?;

    // Process the partition
    let processed = process_partition(&pool, showtime_id, zone_id, 600).await?;
    assert_eq!(processed, 2);

    // Verify first booking succeeded
    let bq1: BookingQueueItem = sqlx::query_as("SELECT * FROM booking_queue WHERE id = $1")
        .bind(bq1_id)
        .fetch_one(&pool)
        .await?;
    assert_eq!(bq1.status, "success");
    assert_eq!(bq1.failure_reason, None);

    // Verify second booking failed with 'seat_taken'
    let bq2: BookingQueueItem = sqlx::query_as("SELECT * FROM booking_queue WHERE id = $1")
        .bind(bq2_id)
        .fetch_one(&pool)
        .await?;
    assert_eq!(bq2.status, "failed");
    assert_eq!(bq2.failure_reason, Some("seat_taken".to_string()));

    // Verify only 1 active seat hold exists in seat_bookings
    let holds: Vec<SeatBooking> = sqlx::query_as(
        "SELECT * FROM seat_bookings WHERE showtime_id = $1 AND seat_id = $2 AND status = 'held'",
    )
    .bind(showtime_id)
    .bind(seat_id)
    .fetch_all(&pool)
    .await?;
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].user_id, user1_id);

    Ok(())
}

#[tokio::test]
async fn test_scenario_2_same_seat_different_showtimes_success()
-> Result<(), Box<dyn std::error::Error>> {
    let pool = setup_test_db().await?;

    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, email, password_hash, role) VALUES ($1, $2, 'hash', 'customer'), ($3, $4, 'hash', 'customer')")
        .bind(user1_id)
        .bind(format!("user1_{}@test.com", user1_id))
        .bind(user2_id)
        .bind(format!("user2_{}@test.com", user2_id))
        .execute(&pool)
        .await?;

    let theater_id = Uuid::new_v4();
    sqlx::query("INSERT INTO theaters (id, name) VALUES ($1, 'Test Theater 2')")
        .bind(theater_id)
        .execute(&pool)
        .await?;

    let event_id = Uuid::new_v4();
    sqlx::query("INSERT INTO events (id, title) VALUES ($1, 'Event 2')")
        .bind(event_id)
        .execute(&pool)
        .await?;

    let zone_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO zones (id, event_id, theater_id, name, price_cents) VALUES ($1, $2, $3, 'Zone B', 1500)",
    )
    .bind(zone_id)
    .bind(event_id)
    .bind(theater_id)
    .execute(&pool)
    .await?;

    let seat_id = Uuid::new_v4();
    sqlx::query("INSERT INTO seats (id, event_id, theater_id, zone_id, row_label, seat_number) VALUES ($1, $2, $3, $4, 'Y', 88)")
        .bind(seat_id)
        .bind(event_id)
        .bind(theater_id)
        .bind(zone_id)
        .execute(&pool)
        .await?;

    let showtime1_id = Uuid::new_v4();
    let showtime2_id = Uuid::new_v4();
    let now = Utc::now();
    sqlx::query("INSERT INTO showtimes (id, event_id, theater_id, starts_at) VALUES ($1, $2, $3, $4), ($5, $2, $3, $6)")
        .bind(showtime1_id)
        .bind(event_id)
        .bind(theater_id)
        .bind(now + Duration::days(1))
        .bind(showtime2_id)
        .bind(now + Duration::days(2))
        .execute(&pool)
        .await?;

    // Enqueue requests for the same seat across two different showtimes
    let bq1_id = Uuid::new_v4();
    let bq2_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO booking_queue (id, user_id, showtime_id, zone_id, seat_id, status, created_at)
         VALUES ($1, $2, $3, $4, $5, 'pending', now())",
    )
    .bind(bq1_id)
    .bind(user1_id)
    .bind(showtime1_id)
    .bind(zone_id)
    .bind(seat_id)
    .execute(&pool)
    .await?;

    sqlx::query(
        "INSERT INTO booking_queue (id, user_id, showtime_id, zone_id, seat_id, status, created_at)
         VALUES ($1, $2, $3, $4, $5, 'pending', now())",
    )
    .bind(bq2_id)
    .bind(user2_id)
    .bind(showtime2_id)
    .bind(zone_id)
    .bind(seat_id)
    .execute(&pool)
    .await?;

    // Process both partitions
    let processed1 = process_partition(&pool, showtime1_id, zone_id, 600).await?;
    let processed2 = process_partition(&pool, showtime2_id, zone_id, 600).await?;
    assert_eq!(processed1, 1);
    assert_eq!(processed2, 1);

    // Verify both bookings succeeded
    let bq1: BookingQueueItem = sqlx::query_as("SELECT * FROM booking_queue WHERE id = $1")
        .bind(bq1_id)
        .fetch_one(&pool)
        .await?;
    assert_eq!(bq1.status, "success");

    let bq2: BookingQueueItem = sqlx::query_as("SELECT * FROM booking_queue WHERE id = $1")
        .bind(bq2_id)
        .fetch_one(&pool)
        .await?;
    assert_eq!(bq2.status, "success");

    // Verify two holds exist (one per showtime)
    let holds1: Vec<SeatBooking> = sqlx::query_as(
        "SELECT * FROM seat_bookings WHERE showtime_id = $1 AND seat_id = $2 AND status = 'held'",
    )
    .bind(showtime1_id)
    .bind(seat_id)
    .fetch_all(&pool)
    .await?;
    assert_eq!(holds1.len(), 1);
    assert_eq!(holds1[0].user_id, user1_id);

    let holds2: Vec<SeatBooking> = sqlx::query_as(
        "SELECT * FROM seat_bookings WHERE showtime_id = $1 AND seat_id = $2 AND status = 'held'",
    )
    .bind(showtime2_id)
    .bind(seat_id)
    .fetch_all(&pool)
    .await?;
    assert_eq!(holds2.len(), 1);
    assert_eq!(holds2[0].user_id, user2_id);

    Ok(())
}
