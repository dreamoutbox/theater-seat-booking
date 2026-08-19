use sqlx::PgExecutor;
use uuid::Uuid;

use crate::models::{Seat, SeatWithStatus};

pub struct SeatRepository;

impl SeatRepository {
    pub async fn find_by_theater_id<'e, E: PgExecutor<'e>>(
        executor: E,
        theater_id: Uuid,
    ) -> Result<Vec<Seat>, sqlx::Error> {
        sqlx::query_as(
            "SELECT id, theater_id, zone_id, row_label, seat_number
             FROM seats
             WHERE theater_id = $1
             ORDER BY row_label ASC, seat_number ASC",
        )
        .bind(theater_id)
        .fetch_all(executor)
        .await
    }

    pub async fn find_seats_with_status_by_theater_id<'e, E: PgExecutor<'e>>(
        executor: E,
        theater_id: Uuid,
        showtime_id: Option<Uuid>,
        current_user_id: Option<Uuid>,
    ) -> Result<Vec<SeatWithStatus>, sqlx::Error> {
        let query_str = "
            SELECT
                s.id,
                s.theater_id,
                s.zone_id,
                z.name AS zone_name,
                z.color AS zone_color,
                z.price_cents,
                s.row_label,
                s.seat_number,
                CASE
                    WHEN sb.status = 'confirmed' THEN 'confirmed'
                    WHEN sb.status = 'held' AND sb.expires_at > now() THEN 'held'
                    ELSE 'available'
                END AS status,
                CASE
                    WHEN sb.id IS NOT NULL 
                         AND (sb.status = 'confirmed' OR (sb.status = 'held' AND sb.expires_at > now()))
                         AND sb.user_id = $3 THEN true
                    ELSE false
                END AS is_mine,
                CASE
                    WHEN sb.status = 'held' AND sb.expires_at > now() THEN sb.expires_at
                    ELSE NULL
                END AS held_expires_at,
                CASE
                    WHEN sb.id IS NOT NULL 
                         AND (sb.status = 'confirmed' OR (sb.status = 'held' AND sb.expires_at > now()))
                         AND sb.user_id = $3 THEN sb.id
                    ELSE NULL
                END AS seat_booking_id
            FROM seats s
            JOIN zones z ON z.id = s.zone_id
            LEFT JOIN seat_bookings sb 
                ON sb.seat_id = s.id 
                AND sb.showtime_id = $2
                AND sb.status IN ('held', 'confirmed')
                AND (sb.status = 'confirmed' OR sb.expires_at > now())
            WHERE s.theater_id = $1
            ORDER BY s.row_label ASC, s.seat_number ASC
        ";

        sqlx::query_as(query_str)
            .bind(theater_id)
            .bind(showtime_id)
            .bind(current_user_id)
            .fetch_all(executor)
            .await
    }

    pub async fn find_zone_id_by_id<'e, E: PgExecutor<'e>>(
        executor: E,
        seat_id: Uuid,
    ) -> Result<Option<Uuid>, sqlx::Error> {
        sqlx::query_scalar("SELECT zone_id FROM seats WHERE id = $1")
            .bind(seat_id)
            .fetch_optional(executor)
            .await
    }

    pub async fn exists_by_theater_id_and_row_label_and_seat_number<'e, E: PgExecutor<'e>>(
        executor: E,
        theater_id: Uuid,
        row_label: &str,
        seat_number: i32,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM seats
                WHERE theater_id = $1 AND row_label = $2 AND seat_number = $3
            )",
        )
        .bind(theater_id)
        .bind(row_label)
        .bind(seat_number)
        .fetch_one(executor)
        .await
    }

    pub async fn upsert<'e, E: PgExecutor<'e>>(
        executor: E,
        theater_id: Uuid,
        zone_id: Uuid,
        row_label: &str,
        seat_number: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO seats (theater_id, zone_id, row_label, seat_number)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (theater_id, row_label, seat_number)
             DO UPDATE SET zone_id = EXCLUDED.zone_id",
        )
        .bind(theater_id)
        .bind(zone_id)
        .bind(row_label)
        .bind(seat_number)
        .execute(executor)
        .await?;
        Ok(())
    }

    pub async fn find_all_keys_by_theater_id<'e, E: PgExecutor<'e>>(
        executor: E,
        theater_id: Uuid,
    ) -> Result<Vec<(Uuid, String, i32)>, sqlx::Error> {
        sqlx::query_as(
            "SELECT id, row_label, seat_number
             FROM seats
             WHERE theater_id = $1",
        )
        .bind(theater_id)
        .fetch_all(executor)
        .await
    }

    pub async fn delete_by_id<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let res = sqlx::query("DELETE FROM seats WHERE id = $1")
            .bind(id)
            .execute(executor)
            .await?;
        Ok(res.rows_affected() > 0)
    }
}
