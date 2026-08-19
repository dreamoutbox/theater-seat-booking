use sqlx::PgExecutor;
use uuid::Uuid;

use crate::models::{OrderDetailRow, OrderSeatDetail};

pub struct OrderRepository;

impl OrderRepository {
    pub async fn save<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
        user_id: Uuid,
        showtime_id: Uuid,
        total_amount_cents: i64,
        stripe_payment_intent_id: Option<&str>,
    ) -> Result<Uuid, sqlx::Error> {
        sqlx::query_scalar(
            "INSERT INTO orders (id, user_id, showtime_id, total_amount_cents, status, stripe_payment_intent_id)
             VALUES ($1, $2, $3, $4, 'pending_payment', $5)
             RETURNING id",
        )
        .bind(id)
        .bind(user_id)
        .bind(showtime_id)
        .bind(total_amount_cents)
        .bind(stripe_payment_intent_id)
        .fetch_one(executor)
        .await
    }

    pub async fn save_order_seat<'e, E: PgExecutor<'e>>(
        executor: E,
        order_id: Uuid,
        seat_booking_id: Uuid,
        price_cents: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO order_seats (order_id, seat_booking_id, price_cents)
             VALUES ($1, $2, $3)",
        )
        .bind(order_id)
        .bind(seat_booking_id)
        .bind(price_cents)
        .execute(executor)
        .await?;
        Ok(())
    }

    pub async fn find_all_order_details<'e, E: PgExecutor<'e>>(
        executor: E,
    ) -> Result<Vec<OrderDetailRow>, sqlx::Error> {
        sqlx::query_as(
            "SELECT o.id, o.user_id, u.email as user_email, o.showtime_id,
                    e.title as event_title, t.name as theater_name, s.starts_at,
                    o.total_amount_cents, o.status, o.stripe_payment_intent_id, o.created_at
             FROM orders o
             JOIN users u ON u.id = o.user_id
             JOIN showtimes s ON s.id = o.showtime_id
             JOIN events e ON e.id = s.event_id
             JOIN theaters t ON t.id = s.theater_id
             ORDER BY o.created_at DESC",
        )
        .fetch_all(executor)
        .await
    }

    pub async fn find_order_details_by_user_id<'e, E: PgExecutor<'e>>(
        executor: E,
        user_id: Uuid,
    ) -> Result<Vec<OrderDetailRow>, sqlx::Error> {
        sqlx::query_as(
            "SELECT o.id, o.user_id, u.email as user_email, o.showtime_id,
                    e.title as event_title, t.name as theater_name, s.starts_at,
                    o.total_amount_cents, o.status, o.stripe_payment_intent_id, o.created_at
             FROM orders o
             JOIN users u ON u.id = o.user_id
             JOIN showtimes s ON s.id = o.showtime_id
             JOIN events e ON e.id = s.event_id
             JOIN theaters t ON t.id = s.theater_id
             WHERE o.user_id = $1
             ORDER BY o.created_at DESC",
        )
        .bind(user_id)
        .fetch_all(executor)
        .await
    }

    pub async fn find_order_detail_by_id<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
    ) -> Result<Option<OrderDetailRow>, sqlx::Error> {
        sqlx::query_as(
            "SELECT o.id, o.user_id, u.email as user_email, o.showtime_id,
                    e.title as event_title, t.name as theater_name, s.starts_at,
                    o.total_amount_cents, o.status, o.stripe_payment_intent_id, o.created_at
             FROM orders o
             JOIN users u ON u.id = o.user_id
             JOIN showtimes s ON s.id = o.showtime_id
             JOIN events e ON e.id = s.event_id
             JOIN theaters t ON t.id = s.theater_id
             WHERE o.id = $1",
        )
        .bind(id)
        .fetch_optional(executor)
        .await
    }

    pub async fn find_order_seats_by_order_id<'e, E: PgExecutor<'e>>(
        executor: E,
        order_id: Uuid,
    ) -> Result<Vec<OrderSeatDetail>, sqlx::Error> {
        sqlx::query_as(
            "SELECT os.seat_booking_id, s.id as seat_id, s.row_label, s.seat_number, z.name as zone_name, os.price_cents
             FROM order_seats os
             JOIN seat_bookings sb ON sb.id = os.seat_booking_id
             JOIN seats s ON s.id = sb.seat_id
             JOIN zones z ON z.id = s.zone_id
             WHERE os.order_id = $1
             ORDER BY s.row_label ASC, s.seat_number ASC",
        )
        .bind(order_id)
        .fetch_all(executor)
        .await
    }

    pub async fn find_status_by_id_and_user_for_update<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<String>, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT status FROM orders WHERE id = $1 AND user_id = $2 FOR UPDATE",
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(executor)
        .await
    }

    pub async fn find_id_by_payment_intent_for_update<'e, E: PgExecutor<'e>>(
        executor: E,
        payment_intent_id: &str,
    ) -> Result<Option<Uuid>, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT id FROM orders WHERE stripe_payment_intent_id = $1 FOR UPDATE",
        )
        .bind(payment_intent_id)
        .fetch_optional(executor)
        .await
    }

    pub async fn update_status<'e, E: PgExecutor<'e>>(
        executor: E,
        order_id: Uuid,
        status: &str,
    ) -> Result<bool, sqlx::Error> {
        let res = sqlx::query("UPDATE orders SET status = $1 WHERE id = $2")
            .bind(status)
            .bind(order_id)
            .execute(executor)
            .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn expire_pending_orders_with_released_seats<'e, E: PgExecutor<'e>>(
        executor: E,
    ) -> Result<u64, sqlx::Error> {
        let res = sqlx::query(
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
        .execute(executor)
        .await?;
        Ok(res.rows_affected())
    }

    pub async fn exists_payment_event_by_stripe_event_id<'e, E: PgExecutor<'e>>(
        executor: E,
        stripe_event_id: &str,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM payment_events WHERE stripe_event_id = $1)",
        )
        .bind(stripe_event_id)
        .fetch_one(executor)
        .await
    }

    pub async fn save_payment_event<'e, E: PgExecutor<'e>>(
        executor: E,
        stripe_event_id: &str,
        order_id: Option<Uuid>,
        event_type: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO payment_events (stripe_event_id, order_id, type)
             VALUES ($1, $2, $3)",
        )
        .bind(stripe_event_id)
        .bind(order_id)
        .bind(event_type)
        .execute(executor)
        .await?;
        Ok(())
    }
}
