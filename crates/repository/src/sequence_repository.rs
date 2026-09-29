use sqlx::PgExecutor;
use uuid::Uuid;

pub struct SequenceRepository;

impl SequenceRepository {
    /// Atomically increments and returns the next sequential number for (scope_id, doc_type, period)
    /// without gaps. Uses PostgreSQL ON CONFLICT DO UPDATE on document_sequences.
    pub async fn next_sequence<'e, E: PgExecutor<'e>>(
        executor: E,
        scope_id: Uuid,
        doc_type: &str,
        period: &str,
    ) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as(
            r#"
            INSERT INTO document_sequences (scope_id, doc_type, period, last_number, updated_at)
            VALUES ($1, $2, $3, 1, now())
            ON CONFLICT (scope_id, doc_type, period)
            DO UPDATE SET last_number = document_sequences.last_number + 1, updated_at = now()
            RETURNING last_number
            "#,
        )
        .bind(scope_id)
        .bind(doc_type)
        .bind(period)
        .fetch_one(executor)
        .await?;

        Ok(row.0)
    }

    /// Helper to generate a formatted Thai Tax Running Number:
    /// e.g. "INV-202609-00001", "CN-202609-00001", "FIN-202609-00001"
    pub async fn next_running_number<'e, E: PgExecutor<'e>>(
        executor: E,
        scope_id: Uuid,
        doc_type: &str,
        prefix: &str,
        period: &str,
    ) -> Result<String, sqlx::Error> {
        let seq = Self::next_sequence(executor, scope_id, doc_type, period).await?;
        Ok(format!("{prefix}-{period}-{seq:05}"))
    }
}
