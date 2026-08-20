use sqlx::PgExecutor;
use uuid::Uuid;

use crate::models::{TicketSeatChange, TicketSeatChangeDetail};

pub struct SeatChangeRepository;

impl SeatChangeRepository {
    #[allow(clippy::too_many_arguments)]
    pub async fn save<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
        order_id: Uuid,
        ticket_id: Option<Uuid>,
        seat_booking_id: Option<Uuid>,
        showtime_id: Uuid,
        old_seat_id: Option<Uuid>,
        new_seat_id: Option<Uuid>,
        old_row_label: &str,
        old_seat_number: i32,
        old_zone_name: &str,
        old_price_cents: i64,
        new_row_label: &str,
        new_seat_number: i32,
        new_zone_name: &str,
        new_price_cents: i64,
        changed_by: Option<Uuid>,
        reason: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<TicketSeatChange, sqlx::Error> {
        sqlx::query_as(
            "INSERT INTO ticket_seat_changes (
                id, order_id, ticket_id, seat_booking_id, showtime_id,
                old_seat_id, new_seat_id, old_row_label, old_seat_number, old_zone_name, old_price_cents,
                new_row_label, new_seat_number, new_zone_name, new_price_cents,
                changed_by, reason, metadata
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9, $10, $11,
                $12, $13, $14, $15,
                $16, $17, $18
            )
            RETURNING id, order_id, ticket_id, seat_booking_id, showtime_id,
                      old_seat_id, new_seat_id, old_row_label, old_seat_number, old_zone_name, old_price_cents,
                      new_row_label, new_seat_number, new_zone_name, new_price_cents,
                      changed_by, reason, metadata, created_at",
        )
        .bind(id)
        .bind(order_id)
        .bind(ticket_id)
        .bind(seat_booking_id)
        .bind(showtime_id)
        .bind(old_seat_id)
        .bind(new_seat_id)
        .bind(old_row_label)
        .bind(old_seat_number)
        .bind(old_zone_name)
        .bind(old_price_cents)
        .bind(new_row_label)
        .bind(new_seat_number)
        .bind(new_zone_name)
        .bind(new_price_cents)
        .bind(changed_by)
        .bind(reason)
        .bind(metadata)
        .fetch_one(executor)
        .await
    }

    pub async fn find_by_order_id<'e, E: PgExecutor<'e>>(
        executor: E,
        order_id: Uuid,
    ) -> Result<Vec<TicketSeatChangeDetail>, sqlx::Error> {
        sqlx::query_as(
            "SELECT 
                sc.id, sc.order_id, sc.ticket_id, t.code AS ticket_code, sc.seat_booking_id, sc.showtime_id,
                sc.old_seat_id, sc.new_seat_id, sc.old_row_label, sc.old_seat_number, sc.old_zone_name, sc.old_price_cents,
                sc.new_row_label, sc.new_seat_number, sc.new_zone_name, sc.new_price_cents,
                sc.changed_by, u.email AS changed_by_email, sc.reason, sc.metadata, sc.created_at
             FROM ticket_seat_changes sc
             LEFT JOIN tickets t ON t.id = sc.ticket_id
             LEFT JOIN users u ON u.id = sc.changed_by
             WHERE sc.order_id = $1
             ORDER BY sc.created_at DESC",
        )
        .bind(order_id)
        .fetch_all(executor)
        .await
    }

    pub async fn find_by_ticket_id<'e, E: PgExecutor<'e>>(
        executor: E,
        ticket_id: Uuid,
    ) -> Result<Vec<TicketSeatChangeDetail>, sqlx::Error> {
        sqlx::query_as(
            "SELECT 
                sc.id, sc.order_id, sc.ticket_id, t.code AS ticket_code, sc.seat_booking_id, sc.showtime_id,
                sc.old_seat_id, sc.new_seat_id, sc.old_row_label, sc.old_seat_number, sc.old_zone_name, sc.old_price_cents,
                sc.new_row_label, sc.new_seat_number, sc.new_zone_name, sc.new_price_cents,
                sc.changed_by, u.email AS changed_by_email, sc.reason, sc.metadata, sc.created_at
             FROM ticket_seat_changes sc
             LEFT JOIN tickets t ON t.id = sc.ticket_id
             LEFT JOIN users u ON u.id = sc.changed_by
             WHERE sc.ticket_id = $1
             ORDER BY sc.created_at DESC",
        )
        .bind(ticket_id)
        .fetch_all(executor)
        .await
    }
}
