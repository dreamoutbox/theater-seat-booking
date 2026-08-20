use sqlx::PgExecutor;
use uuid::Uuid;

use crate::models::{Refund, RefundDetail, RefundTicketItem};

pub struct RefundRepository;

impl RefundRepository {
    pub async fn save<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
        order_id: Uuid,
        amount_cents: i64,
        stripe_refund_id: Option<&str>,
        refunded_by: Uuid,
        reason: Option<&str>,
    ) -> Result<Refund, sqlx::Error> {
        sqlx::query_as(
            "INSERT INTO refunds (id, order_id, amount_cents, stripe_refund_id, refunded_by, reason)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, order_id, amount_cents, stripe_refund_id, refunded_by, reason, created_at",
        )
        .bind(id)
        .bind(order_id)
        .bind(amount_cents)
        .bind(stripe_refund_id)
        .bind(refunded_by)
        .bind(reason)
        .fetch_one(executor)
        .await
    }

    pub async fn save_refund_ticket<'e, E: PgExecutor<'e>>(
        executor: E,
        refund_id: Uuid,
        ticket_id: Uuid,
        price_cents: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO refund_tickets (refund_id, ticket_id, price_cents)
             VALUES ($1, $2, $3)",
        )
        .bind(refund_id)
        .bind(ticket_id)
        .bind(price_cents)
        .execute(executor)
        .await?;
        Ok(())
    }

    pub async fn find_by_order_id<'e, E: PgExecutor<'e>>(
        executor: E,
        order_id: Uuid,
    ) -> Result<Vec<RefundDetail>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct RefundRow {
            id: Uuid,
            order_id: Uuid,
            amount_cents: i64,
            stripe_refund_id: Option<String>,
            refunded_by: Option<Uuid>,
            refunded_by_email: Option<String>,
            reason: Option<String>,
            created_at: chrono::DateTime<chrono::Utc>,
            items: sqlx::types::Json<Vec<RefundTicketItem>>,
        }

        let refunds: Vec<RefundRow> = sqlx::query_as(
            "SELECT 
                r.id, r.order_id, r.amount_cents, r.stripe_refund_id, r.refunded_by,
                u.email AS refunded_by_email, r.reason, r.created_at,
                COALESCE(
                    json_agg(
                        json_build_object(
                            'ticket_id', rt.ticket_id,
                            'ticket_code', t.code,
                            'price_cents', rt.price_cents,
                            'row_label', COALESCE(s.row_label, os.row_label, '—'),
                            'seat_number', COALESCE(s.seat_number, os.seat_number, 0),
                            'zone_name', COALESCE(z.name, os.zone_name, 'Standard')
                        ) ORDER BY COALESCE(s.row_label, os.row_label, '—') ASC, COALESCE(s.seat_number, os.seat_number, 0) ASC
                    ) FILTER (WHERE rt.ticket_id IS NOT NULL),
                    '[]'::json
                ) AS items
             FROM refunds r
             LEFT JOIN users u ON u.id = r.refunded_by
             LEFT JOIN refund_tickets rt ON rt.refund_id = r.id
             LEFT JOIN tickets t ON t.id = rt.ticket_id
             LEFT JOIN seat_bookings sb ON sb.id = t.seat_booking_id
             LEFT JOIN seats s ON s.id = sb.seat_id
             LEFT JOIN zones z ON z.id = s.zone_id
             LEFT JOIN order_seats os ON os.seat_booking_id = t.seat_booking_id
             WHERE r.order_id = $1
             GROUP BY r.id, r.order_id, r.amount_cents, r.stripe_refund_id, r.refunded_by, u.email, r.reason, r.created_at
             ORDER BY r.created_at DESC",
        )
        .bind(order_id)
        .fetch_all(executor)
        .await?;

        let mut details = Vec::with_capacity(refunds.len());
        for r in refunds {
            let items = r.items.0;
            let ticket_ids = items.iter().map(|it| it.ticket_id).collect();
            details.push(RefundDetail {
                id: r.id,
                order_id: r.order_id,
                amount_cents: r.amount_cents,
                stripe_refund_id: r.stripe_refund_id,
                refunded_by: r.refunded_by,
                refunded_by_email: r.refunded_by_email,
                reason: r.reason,
                created_at: r.created_at,
                ticket_ids,
                items,
            });
        }

        Ok(details)
    }
}
