use sqlx::PgExecutor;

pub struct LockRepository;

impl LockRepository {
    pub async fn try_advisory_xact_lock<'e, E: PgExecutor<'e>>(
        executor: E,
        lock_key: i64,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar("SELECT pg_try_advisory_xact_lock($1)")
            .bind(lock_key)
            .fetch_one(executor)
            .await
    }
}
