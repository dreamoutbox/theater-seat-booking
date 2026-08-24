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
        billing_name: Option<&str>,
        tax_id: Option<&str>,
        billing_address: Option<&str>,
    ) -> Result<Uuid, sqlx::Error> {
        sqlx::query_scalar(
            "INSERT INTO orders (id, user_id, showtime_id, total_amount_cents, status, stripe_payment_intent_id, billing_name, tax_id, billing_address)
             VALUES ($1, $2, $3, $4, 'pending_payment', $5, $6, $7, $8)
             RETURNING id",
        )
        .bind(id)
        .bind(user_id)
        .bind(showtime_id)
        .bind(total_amount_cents)
        .bind(stripe_payment_intent_id)
        .bind(billing_name)
        .bind(tax_id)
        .bind(billing_address)
        .fetch_one(executor)
        .await
    }

    pub async fn save_order_seat<'e, E: PgExecutor<'e>>(
        executor: E,
        order_id: Uuid,
        seat_booking_id: Uuid,
        price_cents: i64,
        attendee_name: Option<&str>,
        attendee_phone: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO order_seats (order_id, seat_booking_id, price_cents, original_price_cents, attendee_name, attendee_phone, row_label, seat_number, zone_name)
             SELECT $1, $2, $3, $3, $4, $5, s.row_label, s.seat_number, z.name
             FROM seat_bookings sb
             LEFT JOIN seats s ON s.id = sb.seat_id
             LEFT JOIN zones z ON z.id = s.zone_id
             WHERE sb.id = $2",
        )
        .bind(order_id)
        .bind(seat_booking_id)
        .bind(price_cents)
        .bind(attendee_name)
        .bind(attendee_phone)
        .execute(executor)
        .await?;
        Ok(())
    }

    pub async fn find_all_order_details<'e, E: PgExecutor<'e>>(
        executor: E,
    ) -> Result<Vec<OrderDetailRow>, sqlx::Error> {
        sqlx::query_as(
            "SELECT o.id, o.user_id, u.email as user_email, o.showtime_id, e.id as event_id,
                    e.title as event_title, t.name as theater_name, s.starts_at,
                    o.total_amount_cents, o.refunded_amount_cents, o.status, o.stripe_payment_intent_id,
                    o.billing_name, o.tax_id, o.billing_address,
                    e.tax_business_name as organizer_tax_business_name, e.tax_id as organizer_tax_id,
                    e.tax_address as organizer_tax_address, e.tax_email as organizer_tax_email,
                    e.tax_phone as organizer_tax_phone, o.created_at
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
            "SELECT o.id, o.user_id, u.email as user_email, o.showtime_id, e.id as event_id,
                    e.title as event_title, t.name as theater_name, s.starts_at,
                    o.total_amount_cents, o.refunded_amount_cents, o.status, o.stripe_payment_intent_id,
                    o.billing_name, o.tax_id, o.billing_address,
                    e.tax_business_name as organizer_tax_business_name, e.tax_id as organizer_tax_id,
                    e.tax_address as organizer_tax_address, e.tax_email as organizer_tax_email,
                    e.tax_phone as organizer_tax_phone, o.created_at
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

    pub async fn find_order_details_by_owner_id<'e, E: PgExecutor<'e>>(
        executor: E,
        owner_id: Uuid,
    ) -> Result<Vec<OrderDetailRow>, sqlx::Error> {
        sqlx::query_as(
            "SELECT o.id, o.user_id, u.email as user_email, o.showtime_id, e.id as event_id,
                    e.title as event_title, t.name as theater_name, s.starts_at,
                    o.total_amount_cents, o.refunded_amount_cents, o.status, o.stripe_payment_intent_id,
                    o.billing_name, o.tax_id, o.billing_address,
                    e.tax_business_name as organizer_tax_business_name, e.tax_id as organizer_tax_id,
                    e.tax_address as organizer_tax_address, e.tax_email as organizer_tax_email,
                    e.tax_phone as organizer_tax_phone, o.created_at
             FROM orders o
             JOIN users u ON u.id = o.user_id
             JOIN showtimes s ON s.id = o.showtime_id
             JOIN events e ON e.id = s.event_id
             JOIN theaters t ON t.id = s.theater_id
             WHERE e.owner_id = $1
             ORDER BY o.created_at DESC",
        )
        .bind(owner_id)
        .fetch_all(executor)
        .await
    }

    pub async fn find_order_details_by_event_id<'e, E: PgExecutor<'e>>(
        executor: E,
        event_id: Uuid,
    ) -> Result<Vec<OrderDetailRow>, sqlx::Error> {
        sqlx::query_as(
            "SELECT o.id, o.user_id, u.email as user_email, o.showtime_id, e.id as event_id,
                    e.title as event_title, t.name as theater_name, s.starts_at,
                    o.total_amount_cents, o.refunded_amount_cents, o.status, o.stripe_payment_intent_id,
                    o.billing_name, o.tax_id, o.billing_address,
                    e.tax_business_name as organizer_tax_business_name, e.tax_id as organizer_tax_id,
                    e.tax_address as organizer_tax_address, e.tax_email as organizer_tax_email,
                    e.tax_phone as organizer_tax_phone, o.created_at
             FROM orders o
             JOIN users u ON u.id = o.user_id
             JOIN showtimes s ON s.id = o.showtime_id
             JOIN events e ON e.id = s.event_id
             JOIN theaters t ON t.id = s.theater_id
             WHERE e.id = $1
             ORDER BY o.created_at DESC",
        )
        .bind(event_id)
        .fetch_all(executor)
        .await
    }

    pub async fn find_order_detail_by_id<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
    ) -> Result<Option<OrderDetailRow>, sqlx::Error> {
        sqlx::query_as(
            "SELECT o.id, o.user_id, u.email as user_email, o.showtime_id, e.id as event_id,
                    e.title as event_title, t.name as theater_name, s.starts_at,
                    o.total_amount_cents, o.refunded_amount_cents, o.status, o.stripe_payment_intent_id,
                    o.billing_name, o.tax_id, o.billing_address,
                    e.tax_business_name as organizer_tax_business_name, e.tax_id as organizer_tax_id,
                    e.tax_address as organizer_tax_address, e.tax_email as organizer_tax_email,
                    e.tax_phone as organizer_tax_phone, o.created_at
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
            "SELECT os.seat_booking_id, 
                    COALESCE(s.id, sb.seat_id, os.seat_booking_id) as seat_id, 
                    COALESCE(s.row_label, os.row_label, '—') as row_label, 
                    COALESCE(s.seat_number, os.seat_number, 0) as seat_number, 
                    COALESCE(z.name, os.zone_name, 'Standard') as zone_name,
                    os.price_cents, 
                    COALESCE(os.original_price_cents, os.price_cents) as original_price_cents,
                    os.attendee_name, os.attendee_phone
             FROM order_seats os
             LEFT JOIN seat_bookings sb ON sb.id = os.seat_booking_id
             LEFT JOIN seats s ON s.id = sb.seat_id
             LEFT JOIN zones z ON z.id = s.zone_id
             WHERE os.order_id = $1
             ORDER BY COALESCE(s.row_label, os.row_label, '—') ASC, COALESCE(s.seat_number, os.seat_number, 0) ASC",
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
        sqlx::query_scalar("SELECT status FROM orders WHERE id = $1 AND user_id = $2 FOR UPDATE")
            .bind(id)
            .bind(user_id)
            .fetch_optional(executor)
            .await
    }

    pub async fn find_id_by_payment_intent_for_update<'e, E: PgExecutor<'e>>(
        executor: E,
        payment_intent_id: &str,
    ) -> Result<Option<Uuid>, sqlx::Error> {
        sqlx::query_scalar("SELECT id FROM orders WHERE stripe_payment_intent_id = $1 FOR UPDATE")
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
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM payment_events WHERE stripe_event_id = $1)")
            .bind(stripe_event_id)
            .fetch_one(executor)
            .await
    }

    pub async fn update_refund_status<'e, E: PgExecutor<'e>>(
        executor: E,
        order_id: Uuid,
        status: &str,
        new_total_refunded_cents: i64,
    ) -> Result<bool, sqlx::Error> {
        let res =
            sqlx::query("UPDATE orders SET status = $1, refunded_amount_cents = $2 WHERE id = $3")
                .bind(status)
                .bind(new_total_refunded_cents)
                .bind(order_id)
                .execute(executor)
                .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn find_order_with_event_owner_for_update<'e, E: PgExecutor<'e>>(
        executor: E,
        order_id: Uuid,
    ) -> Result<Option<(Uuid, String, Option<String>, Option<Uuid>, i64, i64)>, sqlx::Error> {
        sqlx::query_as(
            "SELECT o.id, o.status, o.stripe_payment_intent_id, e.owner_id, o.total_amount_cents, o.refunded_amount_cents
             FROM orders o
             JOIN showtimes s ON s.id = o.showtime_id
             JOIN events e ON e.id = s.event_id
             WHERE o.id = $1
             FOR UPDATE OF o",
        )
        .bind(order_id)
        .fetch_optional(executor)
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

    pub async fn update_order_seat_price<'e, E: PgExecutor<'e>>(
        executor: E,
        order_id: Uuid,
        seat_booking_id: Uuid,
        new_price_cents: i64,
    ) -> Result<bool, sqlx::Error> {
        let res = sqlx::query(
            "UPDATE order_seats
             SET price_cents = $1
             WHERE order_id = $2 AND seat_booking_id = $3",
        )
        .bind(new_price_cents)
        .bind(order_id)
        .bind(seat_booking_id)
        .execute(executor)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn update_order_seat_full<'e, E: PgExecutor<'e>>(
        executor: E,
        order_id: Uuid,
        seat_booking_id: Uuid,
        row_label: &str,
        seat_number: i32,
        zone_name: &str,
        price_cents: i64,
    ) -> Result<bool, sqlx::Error> {
        let res = sqlx::query(
            "UPDATE order_seats
             SET row_label = $1, seat_number = $2, zone_name = $3, price_cents = $4
             WHERE order_id = $5 AND seat_booking_id = $6",
        )
        .bind(row_label)
        .bind(seat_number)
        .bind(zone_name)
        .bind(price_cents)
        .bind(order_id)
        .bind(seat_booking_id)
        .execute(executor)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn update_total_amount_from_seats<'e, E: PgExecutor<'e>>(
        executor: E,
        order_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE orders
             SET total_amount_cents = (
                 SELECT COALESCE(SUM(price_cents), 0)
                 FROM order_seats
                 WHERE order_id = $1
             )
             WHERE id = $1",
        )
        .bind(order_id)
        .execute(executor)
        .await?;
        Ok(())
    }
}
