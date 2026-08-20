use chrono::Utc;
use rand::Rng;
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::models::{Ticket, TicketDetail};

pub struct TicketRepository;

impl TicketRepository {
    pub fn generate_ticket_code() -> String {
        const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // Removed ambiguous chars 0, O, 1, I
        let mut rng = rand::thread_rng();
        (0..6)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    pub async fn save<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
        code: &str,
        order_id: Uuid,
        seat_booking_id: Uuid,
        user_id: Uuid,
        event_id: Uuid,
        showtime_id: Uuid,
        attendee_name: Option<&str>,
        attendee_phone: Option<&str>,
    ) -> Result<Ticket, sqlx::Error> {
        sqlx::query_as(
            "INSERT INTO tickets (id, code, order_id, seat_booking_id, user_id, event_id, showtime_id, attendee_name, attendee_phone, status)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'active')
             RETURNING id, code, order_id, seat_booking_id, user_id, event_id, showtime_id, attendee_name, attendee_phone, status, used_at, used_by, refunded_at, refunded_by, created_at",
        )
        .bind(id)
        .bind(code)
        .bind(order_id)
        .bind(seat_booking_id)
        .bind(user_id)
        .bind(event_id)
        .bind(showtime_id)
        .bind(attendee_name)
        .bind(attendee_phone)
        .fetch_one(executor)
        .await
    }

    pub async fn create_tickets_for_paid_order(
        conn: &mut sqlx::PgConnection,
        order_id: Uuid,
    ) -> Result<Vec<Ticket>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct SeatBookingForTicket {
            order_id: Uuid,
            user_id: Uuid,
            showtime_id: Uuid,
            event_id: Uuid,
            seat_booking_id: Uuid,
            attendee_name: Option<String>,
            attendee_phone: Option<String>,
        }

        // 1. Fetch seat bookings with event_id and attendee info
        let items: Vec<SeatBookingForTicket> = sqlx::query_as(
            "SELECT o.id AS order_id, o.user_id, o.showtime_id, st.event_id, sb.id AS seat_booking_id,
                    os.attendee_name, os.attendee_phone
             FROM orders o
             JOIN order_seats os ON os.order_id = o.id
             JOIN seat_bookings sb ON sb.id = os.seat_booking_id
             JOIN showtimes st ON st.id = o.showtime_id
             WHERE o.id = $1",
        )
        .bind(order_id)
        .fetch_all(&mut *conn)
        .await?;

        let mut tickets = Vec::new();

        for item in items {
            // Check if ticket already exists for this seat booking (idempotency)
            let existing: Option<Ticket> = sqlx::query_as(
                "SELECT id, code, order_id, seat_booking_id, user_id, event_id, showtime_id, attendee_name, attendee_phone, status, used_at, used_by, refunded_at, refunded_by, created_at
                 FROM tickets
                 WHERE seat_booking_id = $1",
            )
            .bind(item.seat_booking_id)
            .fetch_optional(&mut *conn)
            .await?;

            if let Some(t) = existing {
                tickets.push(t);
                continue;
            }

            let mut created = false;
            for _ in 0..5 {
                let code = Self::generate_ticket_code();
                let ticket_id = Uuid::new_v4();
                match Self::save(
                    &mut *conn,
                    ticket_id,
                    &code,
                    item.order_id,
                    item.seat_booking_id,
                    item.user_id,
                    item.event_id,
                    item.showtime_id,
                    item.attendee_name.as_deref(),
                    item.attendee_phone.as_deref(),
                )
                .await
                {
                    Ok(t) => {
                        tickets.push(t);
                        created = true;
                        break;
                    }
                    Err(e) => {
                        // Retry on unique constraint violation
                        tracing::warn!(code = %code, error = %e, "Collision on ticket code, retrying generation...");
                    }
                }
            }

            if !created {
                return Err(sqlx::Error::Protocol(
                    "Failed to generate unique ticket code after multiple retries".into(),
                ));
            }
        }

        Ok(tickets)
    }

    pub async fn find_by_order_id<'e, E: PgExecutor<'e>>(
        executor: E,
        order_id: Uuid,
    ) -> Result<Vec<TicketDetail>, sqlx::Error> {
        sqlx::query_as(
            "SELECT 
                t.id, t.code, t.order_id, t.seat_booking_id, t.user_id, t.event_id,
                e.title AS event_title, t.showtime_id, th.name AS theater_name,
                st.starts_at, 
                COALESCE(s.row_label, os.row_label, '—') AS row_label, 
                COALESCE(s.seat_number, os.seat_number, 0) AS seat_number, 
                COALESCE(z.name, os.zone_name, 'Standard') AS zone_name,
                COALESCE(os.price_cents, z.price_cents, 0) AS price_cents, 
                t.attendee_name, t.attendee_phone, t.status,
                t.used_at, t.used_by, t.refunded_at, t.refunded_by, t.created_at
            FROM tickets t
            JOIN events e ON e.id = t.event_id
            JOIN showtimes st ON st.id = t.showtime_id
            JOIN theaters th ON th.id = st.theater_id
            LEFT JOIN seat_bookings sb ON sb.id = t.seat_booking_id
            LEFT JOIN seats s ON s.id = sb.seat_id
            LEFT JOIN zones z ON z.id = s.zone_id
            LEFT JOIN order_seats os ON os.seat_booking_id = t.seat_booking_id
            WHERE t.order_id = $1
            ORDER BY COALESCE(s.row_label, os.row_label, '—') ASC, COALESCE(s.seat_number, os.seat_number, 0) ASC",
        )
        .bind(order_id)
        .fetch_all(executor)
        .await
    }

    pub async fn find_by_ids_and_order_id_for_update<'e, E: PgExecutor<'e>>(
        executor: E,
        ticket_ids: &[Uuid],
        order_id: Uuid,
    ) -> Result<Vec<TicketDetail>, sqlx::Error> {
        sqlx::query_as(
            "SELECT 
                t.id, t.code, t.order_id, t.seat_booking_id, t.user_id, t.event_id,
                e.title AS event_title, t.showtime_id, th.name AS theater_name,
                st.starts_at, 
                COALESCE(s.row_label, os.row_label, '—') AS row_label, 
                COALESCE(s.seat_number, os.seat_number, 0) AS seat_number, 
                COALESCE(z.name, os.zone_name, 'Standard') AS zone_name,
                COALESCE(os.price_cents, z.price_cents, 0) AS price_cents, 
                t.attendee_name, t.attendee_phone, t.status,
                t.used_at, t.used_by, t.refunded_at, t.refunded_by, t.created_at
            FROM tickets t
            JOIN events e ON e.id = t.event_id
            JOIN showtimes st ON st.id = t.showtime_id
            JOIN theaters th ON th.id = st.theater_id
            LEFT JOIN seat_bookings sb ON sb.id = t.seat_booking_id
            LEFT JOIN seats s ON s.id = sb.seat_id
            LEFT JOIN zones z ON z.id = s.zone_id
            LEFT JOIN order_seats os ON os.seat_booking_id = t.seat_booking_id
            WHERE t.order_id = $1 AND t.id = ANY($2)
            FOR UPDATE OF t",
        )
        .bind(order_id)
        .bind(ticket_ids)
        .fetch_all(executor)
        .await
    }

    pub async fn find_by_code<'e, E: PgExecutor<'e>>(
        executor: E,
        code: &str,
    ) -> Result<Option<TicketDetail>, sqlx::Error> {
        sqlx::query_as(
            "SELECT 
                t.id, t.code, t.order_id, t.seat_booking_id, t.user_id, t.event_id,
                e.title AS event_title, t.showtime_id, th.name AS theater_name,
                st.starts_at, 
                COALESCE(s.row_label, os.row_label, '—') AS row_label, 
                COALESCE(s.seat_number, os.seat_number, 0) AS seat_number, 
                COALESCE(z.name, os.zone_name, 'Standard') AS zone_name,
                COALESCE(os.price_cents, z.price_cents, 0) AS price_cents, 
                t.attendee_name, t.attendee_phone, t.status,
                t.used_at, t.used_by, t.refunded_at, t.refunded_by, t.created_at
            FROM tickets t
            JOIN events e ON e.id = t.event_id
            JOIN showtimes st ON st.id = t.showtime_id
            JOIN theaters th ON th.id = st.theater_id
            LEFT JOIN seat_bookings sb ON sb.id = t.seat_booking_id
            LEFT JOIN seats s ON s.id = sb.seat_id
            LEFT JOIN zones z ON z.id = s.zone_id
            LEFT JOIN order_seats os ON os.seat_booking_id = t.seat_booking_id
            WHERE t.code = $1",
        )
        .bind(code)
        .fetch_optional(executor)
        .await
    }

    pub async fn find_by_id<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
    ) -> Result<Option<Ticket>, sqlx::Error> {
        sqlx::query_as(
            "SELECT id, code, order_id, seat_booking_id, user_id, event_id, showtime_id, attendee_name, attendee_phone, status, used_at, used_by, refunded_at, refunded_by, created_at
             FROM tickets
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(executor)
        .await
    }

    pub async fn find_by_seat_booking_id<'e, E: PgExecutor<'e>>(
        executor: E,
        seat_booking_id: Uuid,
    ) -> Result<Option<Ticket>, sqlx::Error> {
        sqlx::query_as(
            "SELECT id, code, order_id, seat_booking_id, user_id, event_id, showtime_id, attendee_name, attendee_phone, status, used_at, used_by, refunded_at, refunded_by, created_at
             FROM tickets
             WHERE seat_booking_id = $1",
        )
        .bind(seat_booking_id)
        .fetch_optional(executor)
        .await
    }

    pub async fn mark_used<'e, E: PgExecutor<'e>>(
        executor: E,
        ticket_id: Uuid,
        used_by: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let now = Utc::now();
        let res = sqlx::query(
            "UPDATE tickets
             SET status = 'used', used_at = $1, used_by = $2
             WHERE id = $3 AND status = 'active'",
        )
        .bind(now)
        .bind(used_by)
        .bind(ticket_id)
        .execute(executor)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn mark_tickets_refunded<'e, E: PgExecutor<'e>>(
        executor: E,
        ticket_ids: &[Uuid],
        refunded_by: Uuid,
    ) -> Result<u64, sqlx::Error> {
        let now = Utc::now();
        let res = sqlx::query(
            "UPDATE tickets
             SET status = 'refunded', refunded_at = $1, refunded_by = $2
             WHERE id = ANY($3) AND status = 'active'",
        )
        .bind(now)
        .bind(refunded_by)
        .bind(ticket_ids)
        .execute(executor)
        .await?;
        Ok(res.rows_affected())
    }

    pub async fn count_active_tickets_by_order_id<'e, E: PgExecutor<'e>>(
        executor: E,
        order_id: Uuid,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM tickets WHERE order_id = $1 AND status = 'active'",
        )
        .bind(order_id)
        .fetch_one(executor)
        .await
    }

    pub async fn cancel_by_seat_booking_id<'e, E: PgExecutor<'e>>(
        executor: E,
        seat_booking_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let res = sqlx::query(
            "UPDATE tickets
             SET status = 'cancelled'
             WHERE seat_booking_id = $1",
        )
        .bind(seat_booking_id)
        .execute(executor)
        .await?;
        Ok(res.rows_affected() > 0)
    }
}
