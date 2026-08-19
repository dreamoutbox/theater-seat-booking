use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::models::{BookingQueueItem, PartitionCandidate};

pub struct BookingQueueRepository;

impl BookingQueueRepository {
    pub async fn save<'e, E: PgExecutor<'e>>(
        executor: E,
        user_id: Uuid,
        showtime_id: Uuid,
        zone_id: Uuid,
        seat_id: Uuid,
    ) -> Result<Uuid, sqlx::Error> {
        sqlx::query_scalar(
            "INSERT INTO booking_queue (user_id, showtime_id, zone_id, seat_id, status)
             VALUES ($1, $2, $3, $4, 'pending')
             RETURNING id",
        )
        .bind(user_id)
        .bind(showtime_id)
        .bind(zone_id)
        .bind(seat_id)
        .fetch_one(executor)
        .await
    }

    pub async fn find_by_id<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
    ) -> Result<Option<BookingQueueItem>, sqlx::Error> {
        sqlx::query_as(
            "SELECT id, user_id, showtime_id, zone_id, seat_id, status, failure_reason, created_at, processed_at
             FROM booking_queue
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(executor)
        .await
    }

    pub async fn count_in_flight_by_user_id<'e, E: PgExecutor<'e>>(
        executor: E,
        user_id: Uuid,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM booking_queue
             WHERE user_id = $1 AND status IN ('pending', 'processing')",
        )
        .bind(user_id)
        .fetch_one(executor)
        .await
    }

    pub async fn find_candidate_partitions<'e, E: PgExecutor<'e>>(
        executor: E,
    ) -> Result<Vec<PartitionCandidate>, sqlx::Error> {
        sqlx::query_as(
            "SELECT showtime_id, zone_id
             FROM booking_queue
             WHERE status = 'pending'
             GROUP BY showtime_id, zone_id
             ORDER BY MIN(created_at) ASC",
        )
        .fetch_all(executor)
        .await
    }

    pub async fn find_oldest_pending_for_update_skip_locked<'e, E: PgExecutor<'e>>(
        executor: E,
        showtime_id: Uuid,
        zone_id: Uuid,
    ) -> Result<Option<BookingQueueItem>, sqlx::Error> {
        sqlx::query_as(
            "SELECT id, user_id, showtime_id, zone_id, seat_id, status, failure_reason, created_at, processed_at
             FROM booking_queue
             WHERE showtime_id = $1 AND zone_id = $2 AND status = 'pending'
             ORDER BY created_at ASC
             LIMIT 1
             FOR UPDATE SKIP LOCKED",
        )
        .bind(showtime_id)
        .bind(zone_id)
        .fetch_optional(executor)
        .await
    }

    pub async fn update_status<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
        status: &str,
        failure_reason: Option<&str>,
        processed_at: DateTime<Utc>,
    ) -> Result<bool, sqlx::Error> {
        let res = sqlx::query(
            "UPDATE booking_queue
             SET status = $1, failure_reason = $2, processed_at = $3
             WHERE id = $4",
        )
        .bind(status)
        .bind(failure_reason)
        .bind(processed_at)
        .bind(id)
        .execute(executor)
        .await?;
        Ok(res.rows_affected() > 0)
    }
}
