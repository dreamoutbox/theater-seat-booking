use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{EventFinancialReport, EventFinancialReportDetail};

pub struct ReportRepository;

impl ReportRepository {
    pub async fn find_by_event_id(
        pool: &PgPool,
        event_id: Uuid,
    ) -> Result<Vec<EventFinancialReportDetail>, sqlx::Error> {
        sqlx::query_as::<_, EventFinancialReportDetail>(
            r#"
            SELECT 
                r.id,
                r.event_id,
                r.statement_ref,
                r.generated_by,
                u.email AS generated_by_email,
                r.generated_at,
                r.gross_sales_cents,
                r.total_refunds_cents,
                r.net_profit_cents,
                r.tax_withholding_cents,
                r.net_settlement_cents,
                r.tickets_issued,
                r.tickets_scanned,
                r.tickets_refunded,
                r.attendance_rate_percent,
                r.report_data
            FROM event_financial_reports r
            LEFT JOIN users u ON u.id = r.generated_by
            WHERE r.event_id = $1
            ORDER BY r.generated_at DESC
            "#,
        )
        .bind(event_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_latest_by_event_id(
        pool: &PgPool,
        event_id: Uuid,
    ) -> Result<Option<EventFinancialReportDetail>, sqlx::Error> {
        sqlx::query_as::<_, EventFinancialReportDetail>(
            r#"
            SELECT 
                r.id,
                r.event_id,
                r.statement_ref,
                r.generated_by,
                u.email AS generated_by_email,
                r.generated_at,
                r.gross_sales_cents,
                r.total_refunds_cents,
                r.net_profit_cents,
                r.tax_withholding_cents,
                r.net_settlement_cents,
                r.tickets_issued,
                r.tickets_scanned,
                r.tickets_refunded,
                r.attendance_rate_percent,
                r.report_data
            FROM event_financial_reports r
            LEFT JOIN users u ON u.id = r.generated_by
            WHERE r.event_id = $1
            ORDER BY r.generated_at DESC
            LIMIT 1
            "#,
        )
        .bind(event_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_id(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<Option<EventFinancialReportDetail>, sqlx::Error> {
        sqlx::query_as::<_, EventFinancialReportDetail>(
            r#"
            SELECT 
                r.id,
                r.event_id,
                r.statement_ref,
                r.generated_by,
                u.email AS generated_by_email,
                r.generated_at,
                r.gross_sales_cents,
                r.total_refunds_cents,
                r.net_profit_cents,
                r.tax_withholding_cents,
                r.net_settlement_cents,
                r.tickets_issued,
                r.tickets_scanned,
                r.tickets_refunded,
                r.attendance_rate_percent,
                r.report_data
            FROM event_financial_reports r
            LEFT JOIN users u ON u.id = r.generated_by
            WHERE r.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn count_by_event_id(
        pool: &PgPool,
        event_id: Uuid,
    ) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM event_financial_reports WHERE event_id = $1",
        )
        .bind(event_id)
        .fetch_one(pool)
        .await?;

        Ok(count.0)
    }

    pub async fn save(
        pool: &PgPool,
        report: &EventFinancialReport,
    ) -> Result<EventFinancialReport, sqlx::Error> {
        sqlx::query_as::<_, EventFinancialReport>(
            r#"
            INSERT INTO event_financial_reports (
                id,
                event_id,
                statement_ref,
                generated_by,
                generated_at,
                gross_sales_cents,
                total_refunds_cents,
                net_profit_cents,
                tax_withholding_cents,
                net_settlement_cents,
                tickets_issued,
                tickets_scanned,
                tickets_refunded,
                attendance_rate_percent,
                report_data
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING *
            "#,
        )
        .bind(report.id)
        .bind(report.event_id)
        .bind(&report.statement_ref)
        .bind(report.generated_by)
        .bind(report.generated_at)
        .bind(report.gross_sales_cents)
        .bind(report.total_refunds_cents)
        .bind(report.net_profit_cents)
        .bind(report.tax_withholding_cents)
        .bind(report.net_settlement_cents)
        .bind(report.tickets_issued)
        .bind(report.tickets_scanned)
        .bind(report.tickets_refunded)
        .bind(report.attendance_rate_percent)
        .bind(&report.report_data)
        .fetch_one(pool)
        .await
    }
}
