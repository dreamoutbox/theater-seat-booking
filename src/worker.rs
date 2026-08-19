use chrono::{Duration, Utc};
use repository::{
    BookingQueueRepository, LockRepository, OrderRepository, PartitionCandidate,
    SeatBookingRepository,
};
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration as StdDuration;
use uuid::Uuid;

use crate::config::AppConfig;

pub fn partition_lock_key(showtime_id: &Uuid, zone_id: &Uuid) -> i64 {
    let mut hasher = DefaultHasher::new();
    showtime_id.hash(&mut hasher);
    zone_id.hash(&mut hasher);
    hasher.finish() as i64
}

pub async fn fetch_candidate_partitions(
    pool: &PgPool,
) -> Result<Vec<PartitionCandidate>, sqlx::Error> {
    BookingQueueRepository::find_candidate_partitions(pool).await
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
        let acquired = LockRepository::try_advisory_xact_lock(&mut *tx, lock_key).await?;

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
        let row_opt = BookingQueueRepository::find_oldest_pending_for_update_skip_locked(
            &mut *tx,
            showtime_id,
            zone_id,
        )
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
        let taken = SeatBookingRepository::check_seat_is_taken_for_update(
            &mut *tx,
            row.showtime_id,
            row.seat_id,
        )
        .await?;

        let now = Utc::now();
        if taken {
            tracing::warn!(
                booking_uuid = %booking_uuid,
                seat_id = %row.seat_id,
                "Seat already taken or actively held. Marking booking as failed."
            );

            BookingQueueRepository::update_status(
                &mut *tx,
                booking_uuid,
                "failed",
                Some("seat_taken"),
                now,
            )
            .await?;
        } else {
            let expires_at = now + Duration::seconds(hold_ttl_seconds);

            // Insert hold into seat_bookings (backed by partial unique index)
            let insert_res = SeatBookingRepository::save_hold(
                &mut *tx,
                row.showtime_id,
                row.seat_id,
                row.user_id,
                booking_uuid,
                now,
                expires_at,
            )
            .await;

            match insert_res {
                Ok(_) => {
                    BookingQueueRepository::update_status(
                        &mut *tx,
                        booking_uuid,
                        "success",
                        None,
                        now,
                    )
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

                    BookingQueueRepository::update_status(
                        &mut *tx,
                        booking_uuid,
                        "failed",
                        Some("seat_taken"),
                        now,
                    )
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
    let released_count = SeatBookingRepository::release_expired_holds(&mut *tx).await?;

    // 2. Expire any order still waiting on payment whose seats just got released
    let expired_orders_count =
        OrderRepository::expire_pending_orders_with_released_seats(&mut *tx).await?;

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
