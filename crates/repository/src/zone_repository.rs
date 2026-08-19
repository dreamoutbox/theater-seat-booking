use sqlx::PgExecutor;
use uuid::Uuid;

use crate::models::Zone;

pub struct ZoneRepository;

impl ZoneRepository {
    pub async fn find_by_theater_id<'e, E: PgExecutor<'e>>(
        executor: E,
        theater_id: Uuid,
    ) -> Result<Vec<Zone>, sqlx::Error> {
        sqlx::query_as(
            "SELECT id, theater_id, name, price_cents, color
             FROM zones
             WHERE theater_id = $1
             ORDER BY price_cents DESC",
        )
        .bind(theater_id)
        .fetch_all(executor)
        .await
    }

    pub async fn find_by_theater_id_and_name<'e, E: PgExecutor<'e>>(
        executor: E,
        theater_id: Uuid,
        name: &str,
    ) -> Result<Option<Zone>, sqlx::Error> {
        sqlx::query_as(
            "SELECT id, theater_id, name, price_cents, color
             FROM zones
             WHERE theater_id = $1 AND name = $2
             LIMIT 1",
        )
        .bind(theater_id)
        .bind(name)
        .fetch_optional(executor)
        .await
    }

    pub async fn exists_by_id_and_theater_id<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
        theater_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM zones WHERE id = $1 AND theater_id = $2)",
        )
        .bind(id)
        .bind(theater_id)
        .fetch_one(executor)
        .await
    }

    pub async fn save<'e, E: PgExecutor<'e>>(
        executor: E,
        theater_id: Uuid,
        name: &str,
        price_cents: i64,
        color: &str,
    ) -> Result<Uuid, sqlx::Error> {
        sqlx::query_scalar(
            "INSERT INTO zones (theater_id, name, price_cents, color)
             VALUES ($1, $2, $3, $4)
             RETURNING id",
        )
        .bind(theater_id)
        .bind(name)
        .bind(price_cents)
        .bind(color)
        .fetch_one(executor)
        .await
    }

    pub async fn save_with_id<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
        theater_id: Uuid,
        name: &str,
        price_cents: i64,
        color: &str,
    ) -> Result<Uuid, sqlx::Error> {
        sqlx::query_scalar(
            "INSERT INTO zones (id, theater_id, name, price_cents, color)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id",
        )
        .bind(id)
        .bind(theater_id)
        .bind(name)
        .bind(price_cents)
        .bind(color)
        .fetch_one(executor)
        .await
    }

    pub async fn update<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
        theater_id: Uuid,
        name: &str,
        price_cents: i64,
        color: &str,
    ) -> Result<bool, sqlx::Error> {
        let res = sqlx::query(
            "UPDATE zones
             SET name = $1, price_cents = $2, color = $3
             WHERE id = $4 AND theater_id = $5",
        )
        .bind(name)
        .bind(price_cents)
        .bind(color)
        .bind(id)
        .bind(theater_id)
        .execute(executor)
        .await?;
        Ok(res.rows_affected() > 0)
    }
}
