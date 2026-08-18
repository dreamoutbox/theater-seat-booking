use chrono::{Duration, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration as StdDuration;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::models::BookingQueueItem;

pub fn partition_lock_key(showtime_id: &Uuid, zone_id: &Uuid) -> i64 {
    let mut hasher = DefaultHasher::new();
    showtime_id.hash(&mut hasher);
    zone_id.hash(&mut hasher);
    hasher.finish() as i64
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PartitionCandidate {
    pub showtime_id: Uuid,
    pub zone_id: Uuid,
}

pub async fn fetch_candidate_partitions(
    pool: &PgPool,
) -> Result<Vec<PartitionCandidate>, sqlx::Error> {
    sqlx::query_as::<_, PartitionCandidate>(
        "SELECT showtime_id, zone_id
         FROM booking_queue
         WHERE status = 'pending'
         GROUP BY showtime_id, zone_id
         ORDER BY MIN(created_at) ASC",
    )
    .fetch_all(pool)
    .await
}

pub async fn process_partition(
    pool: &PgPool,
    showtime_id: Uuid,
    zone_id: Uuid,
    hold_ttl_seconds: i64,
) -> Result<usize, sqlx::Error> {
    let lock_key = partition_lock_key(&showtime_id, &zone_id);
    let mut processed_count = 0;

    loop {
        let mut tx: Transaction<'_, Postgres> = pool.begin().await?;

        // 1. Transaction-scoped advisory lock for the partition
        let acquired: bool = sqlx::query_scalar("SELECT pg_try_advisory_xact_lock($1)")
            .bind(lock_key)
            .fetch_one(&mut *tx)
            .await?;

        if !acquired {
            tracing::debug!(
                showtime_id = %showtime_id,
                zone_id = %zone_id,
                "Advisory lock held by another worker, skipping partition"
            );
            tx.rollback().await?;
            break;
        }

        // 2. Fetch the oldest pending booking in this partition
        let row_opt = sqlx::query_as::<_, BookingQueueItem>(
            "SELECT * FROM booking_queue
             WHERE showtime_id = $1 AND zone_id = $2 AND status = 'pending'
             ORDER BY created_at ASC
             LIMIT 1
             FOR UPDATE SKIP LOCKED",
        )
        .bind(showtime_id)
        .bind(zone_id)
        .fetch_optional(&mut *tx)
        .await?;

        let Some(row) = row_opt else {
            // No more pending items in this partition
            tx.commit().await?;
            break;
        };

        let booking_uuid = row.id;
        tracing::info!(
            booking_uuid = %booking_uuid,
            seat_id = %row.seat_id,
            showtime_id = %row.showtime_id,
            user_id = %row.user_id,
            "Worker picked up booking request"
        );

        // 3. Check if the seat is already held or confirmed
        // Defense-in-depth: status in ('held', 'confirmed') AND (confirmed OR unexpired hold)
        let taken: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM seat_bookings
                WHERE showtime_id = $1 AND seat_id = $2
                  AND status IN ('held','confirmed')
                  AND (status = 'confirmed' OR expires_at > now())
                FOR UPDATE
            )",
        )
        .bind(row.showtime_id)
        .bind(row.seat_id)
        .fetch_one(&mut *tx)
        .await?;

        let now = Utc::now();
        if taken {
            tracing::warn!(
                booking_uuid = %booking_uuid,
                seat_id = %row.seat_id,
                "Seat already taken or actively held. Marking booking as failed."
            );

            sqlx::query(
                "UPDATE booking_queue
                 SET status = 'failed', failure_reason = 'seat_taken', processed_at = $1
                 WHERE id = $2",
            )
            .bind(now)
            .bind(booking_uuid)
            .execute(&mut *tx)
            .await?;
        } else {
            let expires_at = now + Duration::seconds(hold_ttl_seconds);

            // Insert hold into seat_bookings (backed by partial unique index)
            let insert_res = sqlx::query(
                "INSERT INTO seat_bookings (showtime_id, seat_id, user_id, booking_queue_id, status, held_at, expires_at)
                 VALUES ($1, $2, $3, $4, 'held', $5, $6)",
            )
            .bind(row.showtime_id)
            .bind(row.seat_id)
            .bind(row.user_id)
            .bind(booking_uuid)
            .bind(now)
            .bind(expires_at)
            .execute(&mut *tx)
            .await;

            match insert_res {
                Ok(_) => {
                    sqlx::query(
                        "UPDATE booking_queue
                         SET status = 'success', failure_reason = NULL, processed_at = $1
                         WHERE id = $2",
                    )
                    .bind(now)
                    .bind(booking_uuid)
                    .execute(&mut *tx)
                    .await?;

                    tracing::info!(
                        booking_uuid = %booking_uuid,
                        seat_id = %row.seat_id,
                        expires_at = %expires_at,
                        "Seat hold acquired successfully"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        booking_uuid = %booking_uuid,
                        error = %e,
                        "Seat unique constraint collision or insert error"
                    );

                    sqlx::query(
                        "UPDATE booking_queue
                         SET status = 'failed', failure_reason = 'seat_taken', processed_at = $1
                         WHERE id = $2",
                    )
                    .bind(now)
                    .bind(booking_uuid)
                    .execute(&mut *tx)
                    .await?;
                }
            }
        }

        // Commit transaction atomically: releases advisory lock & persists changes
        tx.commit().await?;
        processed_count += 1;
    }

    Ok(processed_count)
}

pub async fn run_worker_loop(pool: PgPool, config: AppConfig) {
    let poll_interval = StdDuration::from_millis(config.poll_interval_ms);
    tracing::info!(
        poll_interval_ms = config.poll_interval_ms,
        hold_ttl_seconds = config.hold_ttl_seconds,
        "Starting Booking Queue Worker loop"
    );

    loop {
        match fetch_candidate_partitions(&pool).await {
            Ok(partitions) => {
                if !partitions.is_empty() {
                    tracing::debug!(
                        candidate_partition_count = partitions.len(),
                        "Found pending partitions"
                    );
                }
                for partition in partitions {
                    if let Err(err) = process_partition(
                        &pool,
                        partition.showtime_id,
                        partition.zone_id,
                        config.hold_ttl_seconds,
                    )
                    .await
                    {
                        tracing::error!(
                            showtime_id = %partition.showtime_id,
                            zone_id = %partition.zone_id,
                            error = %err,
                            "Error processing booking partition"
                        );
                    }
                }
            }
            Err(err) => {
                tracing::error!(error = %err, "Error fetching candidate partitions");
            }
        }

        tokio::time::sleep(poll_interval).await;
    }
}

pub async fn sweep_expired_holds(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let mut tx = pool.begin().await?;

    // 1. Release the expired holds
    let released_res = sqlx::query(
        "UPDATE seat_bookings
         SET status = 'released'
         WHERE status = 'held' AND expires_at <= now()",
    )
    .execute(&mut *tx)
    .await?;

    let released_count = released_res.rows_affected();

    // 2. Expire any order still waiting on payment whose seats just got released
    let expired_orders_res = sqlx::query(
        "UPDATE orders
         SET status = 'expired'
         WHERE status = 'pending_payment'
           AND id IN (
               SELECT os.order_id
               FROM order_seats os
               JOIN seat_bookings sb ON sb.id = os.seat_booking_id
               WHERE sb.status = 'released'
           )",
    )
    .execute(&mut *tx)
    .await?;

    let expired_orders_count = expired_orders_res.rows_affected();

    tx.commit().await?;

    if released_count > 0 || expired_orders_count > 0 {
        tracing::info!(
            released_seats = released_count,
            expired_orders = expired_orders_count,
            "Swept expired seat holds and pending orders"
        );
    }

    Ok(released_count)
}

pub async fn run_sweeper_loop(pool: PgPool, interval_seconds: u64) {
    let interval_duration = StdDuration::from_secs(interval_seconds);
    tracing::info!(
        interval_seconds = interval_seconds,
        "Starting Expired Holds Sweeper loop"
    );

    let mut interval = tokio::time::interval(interval_duration);
    loop {
        interval.tick().await;
        if let Err(err) = sweep_expired_holds(&pool).await {
            tracing::error!(error = %err, "Error sweeping expired seat holds");
        }
    }
}
