use sqlx::PgExecutor;
use uuid::Uuid;

use crate::models::{EventDashboardMetrics, ShowtimeDashboardRow};

pub struct AnalyticsRepository;

impl AnalyticsRepository {
    pub async fn get_event_dashboard_metrics<'e, E: PgExecutor<'e>>(
        executor: E,
        event_id: Option<Uuid>,
        owner_id: Option<Uuid>,
    ) -> Result<EventDashboardMetrics, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct MetricsRow {
            gross_sales_cents: Option<i64>,
            total_refunds_cents: Option<i64>,
            total_tickets_created: Option<i64>,
            active_tickets: Option<i64>,
            used_tickets: Option<i64>,
            refunded_tickets: Option<i64>,
        }

        let row: MetricsRow = sqlx::query_as(
            "SELECT 
                COALESCE(SUM(o.total_amount_cents), 0) AS gross_sales_cents,
                COALESCE(SUM(o.refunded_amount_cents), 0) AS total_refunds_cents,
                (
                    SELECT COUNT(*)
                    FROM tickets t
                    JOIN events e_sub ON e_sub.id = t.event_id
                    WHERE ($1::uuid IS NULL OR t.event_id = $1)
                      AND ($2::uuid IS NULL OR e_sub.owner_id = $2)
                ) AS total_tickets_created,
                (
                    SELECT COUNT(*)
                    FROM tickets t
                    JOIN events e_sub ON e_sub.id = t.event_id
                    WHERE ($1::uuid IS NULL OR t.event_id = $1)
                      AND ($2::uuid IS NULL OR e_sub.owner_id = $2)
                      AND t.status = 'active'
                ) AS active_tickets,
                (
                    SELECT COUNT(*)
                    FROM tickets t
                    JOIN events e_sub ON e_sub.id = t.event_id
                    WHERE ($1::uuid IS NULL OR t.event_id = $1)
                      AND ($2::uuid IS NULL OR e_sub.owner_id = $2)
                      AND t.status = 'used'
                ) AS used_tickets,
                (
                    SELECT COUNT(*)
                    FROM tickets t
                    JOIN events e_sub ON e_sub.id = t.event_id
                    WHERE ($1::uuid IS NULL OR t.event_id = $1)
                      AND ($2::uuid IS NULL OR e_sub.owner_id = $2)
                      AND t.status = 'refunded'
                ) AS refunded_tickets
             FROM orders o
             JOIN showtimes st ON st.id = o.showtime_id
             JOIN events e ON e.id = st.event_id
             WHERE o.status IN ('paid', 'partially_refunded', 'refunded')
               AND ($1::uuid IS NULL OR e.id = $1)
               AND ($2::uuid IS NULL OR e.owner_id = $2)",
        )
        .bind(event_id)
        .bind(owner_id)
        .fetch_one(executor)
        .await?;

        let gross = row.gross_sales_cents.unwrap_or(0);
        let refunds = row.total_refunds_cents.unwrap_or(0);
        let net = gross - refunds;
        let total_tix = row.total_tickets_created.unwrap_or(0);
        let active_tix = row.active_tickets.unwrap_or(0);
        let used_tix = row.used_tickets.unwrap_or(0);
        let refunded_tix = row.refunded_tickets.unwrap_or(0);

        let valid_tickets = active_tix + used_tix;
        let attendance_rate = if valid_tickets > 0 {
            (used_tix as f64 / valid_tickets as f64) * 100.0
        } else {
            0.0
        };

        Ok(EventDashboardMetrics {
            gross_sales_cents: gross,
            total_refunds_cents: refunds,
            net_profit_cents: net,
            total_tickets_created: total_tix,
            active_tickets: active_tix,
            used_tickets: used_tix,
            refunded_tickets: refunded_tix,
            attendance_rate_percent: (attendance_rate * 10.0).round() / 10.0,
        })
    }

    pub async fn get_showtime_dashboard_breakdown<'e, E: PgExecutor<'e>>(
        executor: E,
        event_id: Option<Uuid>,
        owner_id: Option<Uuid>,
    ) -> Result<Vec<ShowtimeDashboardRow>, sqlx::Error> {
        sqlx::query_as(
            "SELECT 
                st.id AS showtime_id,
                e.id AS event_id,
                e.title AS event_title,
                th.id AS theater_id,
                th.name AS theater_name,
                st.starts_at,
                (SELECT COUNT(*) FROM seats s WHERE s.theater_id = th.id) AS total_seats,
                COALESCE((
                    SELECT COUNT(*)
                    FROM tickets t
                    WHERE t.showtime_id = st.id AND t.status IN ('active', 'used')
                ), 0) AS sold_seats,
                COALESCE((
                    SELECT COUNT(*)
                    FROM tickets t
                    WHERE t.showtime_id = st.id AND t.status = 'used'
                ), 0) AS used_tickets,
                COALESCE((
                    SELECT COUNT(*)
                    FROM tickets t
                    WHERE t.showtime_id = st.id AND t.status = 'refunded'
                ), 0) AS refunded_tickets,
                COALESCE((
                    SELECT SUM(o.total_amount_cents)
                    FROM orders o
                    WHERE o.showtime_id = st.id AND o.status IN ('paid', 'partially_refunded', 'refunded')
                ), 0) AS gross_cents,
                COALESCE((
                    SELECT SUM(o.refunded_amount_cents)
                    FROM orders o
                    WHERE o.showtime_id = st.id AND o.status IN ('paid', 'partially_refunded', 'refunded')
                ), 0) AS refunded_cents,
                (
                    COALESCE((
                        SELECT SUM(o.total_amount_cents)
                        FROM orders o
                        WHERE o.showtime_id = st.id AND o.status IN ('paid', 'partially_refunded', 'refunded')
                    ), 0) -
                    COALESCE((
                        SELECT SUM(o.refunded_amount_cents)
                        FROM orders o
                        WHERE o.showtime_id = st.id AND o.status IN ('paid', 'partially_refunded', 'refunded')
                    ), 0)
                ) AS net_cents
             FROM showtimes st
             JOIN events e ON e.id = st.event_id
             JOIN theaters th ON th.id = st.theater_id
             WHERE ($1::uuid IS NULL OR e.id = $1)
               AND ($2::uuid IS NULL OR e.owner_id = $2)
             ORDER BY st.starts_at DESC",
        )
        .bind(event_id)
        .bind(owner_id)
        .fetch_all(executor)
        .await
    }
}
