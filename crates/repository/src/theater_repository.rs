use sqlx::PgExecutor;
use uuid::Uuid;

use crate::models::Theater;

pub struct TheaterRepository;

impl TheaterRepository {
    pub async fn find_all<'e, E: PgExecutor<'e>>(
        executor: E,
    ) -> Result<Vec<Theater>, sqlx::Error> {
        sqlx::query_as("SELECT id, name, address FROM theaters ORDER BY name")
            .fetch_all(executor)
            .await
    }

    pub async fn find_by_id<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
    ) -> Result<Option<Theater>, sqlx::Error> {
        sqlx::query_as("SELECT id, name, address FROM theaters WHERE id = $1")
            .bind(id)
            .fetch_optional(executor)
            .await
    }

    pub async fn find_by_name<'e, E: PgExecutor<'e>>(
        executor: E,
        name: &str,
    ) -> Result<Option<Theater>, sqlx::Error> {
        sqlx::query_as("SELECT id, name, address FROM theaters WHERE name = $1 LIMIT 1")
            .bind(name)
            .fetch_optional(executor)
            .await
    }

    pub async fn save<'e, E: PgExecutor<'e>>(
        executor: E,
        name: &str,
        address: Option<&str>,
    ) -> Result<Theater, sqlx::Error> {
        sqlx::query_as(
            "INSERT INTO theaters (name, address) VALUES ($1, $2) RETURNING id, name, address",
        )
        .bind(name)
        .bind(address)
        .fetch_one(executor)
        .await
    }

    pub async fn update<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
        name: &str,
        address: Option<&str>,
    ) -> Result<Option<Theater>, sqlx::Error> {
        sqlx::query_as(
            "UPDATE theaters SET name = $1, address = $2 WHERE id = $3 RETURNING id, name, address",
        )
        .bind(name)
        .bind(address)
        .bind(id)
        .fetch_optional(executor)
        .await
    }

    pub async fn delete_by_id<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let res = sqlx::query("DELETE FROM theaters WHERE id = $1")
            .bind(id)
            .execute(executor)
            .await?;
        Ok(res.rows_affected() > 0)
    }
}
