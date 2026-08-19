use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::models::HoldItem;

pub struct SeatBookingRepository;

impl SeatBookingRepository {
    pub async fn count_active_holds_by_user_id<'e, E: PgExecutor<'e>>(
        executor: E,
        user_id: Uuid,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM seat_bookings
             WHERE user_id = $1 AND status = 'held' AND expires_at > now()",
        )
        .bind(user_id)
        .fetch_one(executor)
        .await
    }

    pub async fn count_active_holds_by_showtime_and_user_id<'e, E: PgExecutor<'e>>(
        executor: E,
        showtime_id: Uuid,
        user_id: Uuid,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM seat_bookings sb
             WHERE sb.showtime_id = $1 AND sb.user_id = $2 AND sb.status = 'held' AND sb.expires_at > now()",
        )
        .bind(showtime_id)
        .bind(user_id)
        .fetch_one(executor)
        .await
    }

    pub async fn count_confirmed_and_held_by_showtime_and_user_id<'e, E: PgExecutor<'e>>(
        executor: E,
        showtime_id: Uuid,
        user_id: Uuid,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM seat_bookings sb
             JOIN showtimes s ON sb.showtime_id = s.id
             WHERE s.event_id = (SELECT event_id FROM showtimes WHERE id = $1)
               AND sb.user_id = $2
               AND (sb.status = 'confirmed' OR (sb.status = 'held' AND sb.expires_at > now()))",
        )
        .bind(showtime_id)
        .bind(user_id)
        .fetch_one(executor)
        .await
    }

    pub async fn find_active_holds_by_user_and_seat_ids<'e, E: PgExecutor<'e>>(
        executor: E,
        seat_booking_or_seat_ids: &[Uuid],
        user_id: Uuid,
    ) -> Result<Vec<HoldItem>, sqlx::Error> {
        sqlx::query_as(
            "SELECT sb.id, sb.showtime_id, z.price_cents
             FROM seat_bookings sb
             JOIN seats s ON s.id = sb.seat_id
             JOIN zones z ON z.id = s.zone_id
             WHERE (sb.id = ANY($1) OR sb.seat_id = ANY($1))
               AND sb.user_id = $2 
               AND sb.status = 'held' 
               AND sb.expires_at > now()
             FOR UPDATE OF sb",
        )
        .bind(seat_booking_or_seat_ids)
        .bind(user_id)
        .fetch_all(executor)
        .await
    }

    pub async fn check_seat_is_taken_for_update<'e, E: PgExecutor<'e>>(
        executor: E,
        showtime_id: Uuid,
        seat_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM seat_bookings
                WHERE showtime_id = $1 AND seat_id = $2
                  AND status IN ('held', 'confirmed')
                  AND (status = 'confirmed' OR expires_at > now())
                FOR UPDATE
            )",
        )
        .bind(showtime_id)
        .bind(seat_id)
        .fetch_one(executor)
        .await
    }

    pub async fn save_hold<'e, E: PgExecutor<'e>>(
        executor: E,
        showtime_id: Uuid,
        seat_id: Uuid,
        user_id: Uuid,
        booking_queue_id: Uuid,
        held_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO seat_bookings (showtime_id, seat_id, user_id, booking_queue_id, status, held_at, expires_at)
             VALUES ($1, $2, $3, $4, 'held', $5, $6)",
        )
        .bind(showtime_id)
        .bind(seat_id)
        .bind(user_id)
        .bind(booking_queue_id)
        .bind(held_at)
        .bind(expires_at)
        .execute(executor)
        .await?;
        Ok(())
    }

    pub async fn release_hold_by_user_and_seat<'e, E: PgExecutor<'e>>(
        executor: E,
        user_id: Uuid,
        showtime_id: Uuid,
        seat_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let res = sqlx::query(
            "UPDATE seat_bookings
             SET status = 'released'
             WHERE user_id = $1 AND showtime_id = $2 AND seat_id = $3 AND status = 'held'",
        )
        .bind(user_id)
        .bind(showtime_id)
        .bind(seat_id)
        .execute(executor)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn release_expired_holds<'e, E: PgExecutor<'e>>(
        executor: E,
    ) -> Result<u64, sqlx::Error> {
        let res = sqlx::query(
            "UPDATE seat_bookings
             SET status = 'released'
             WHERE status = 'held' AND expires_at <= now()",
        )
        .execute(executor)
        .await?;
        Ok(res.rows_affected())
    }

    pub async fn update_status_by_order_id<'e, E: PgExecutor<'e>>(
        executor: E,
        order_id: Uuid,
        status: &str,
    ) -> Result<u64, sqlx::Error> {
        let res = sqlx::query(
            "UPDATE seat_bookings
             SET status = $1
             WHERE id IN (SELECT seat_booking_id FROM order_seats WHERE order_id = $2)",
        )
        .bind(status)
        .bind(order_id)
        .execute(executor)
        .await?;
        Ok(res.rows_affected())
    }
}
