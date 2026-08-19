use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::models::ShowtimeDetail;

pub struct ShowtimeRepository;

impl ShowtimeRepository {
    pub async fn find_all<'e, E: PgExecutor<'e>>(
        executor: E,
    ) -> Result<Vec<ShowtimeDetail>, sqlx::Error> {
        sqlx::query_as(
            "SELECT s.id, s.event_id, e.title AS event_title, s.theater_id, t.name AS theater_name, s.starts_at
             FROM showtimes s
             JOIN events e ON e.id = s.event_id
             JOIN theaters t ON t.id = s.theater_id
             ORDER BY s.starts_at ASC",
        )
        .fetch_all(executor)
        .await
    }

    pub async fn find_by_id<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
    ) -> Result<Option<ShowtimeDetail>, sqlx::Error> {
        sqlx::query_as(
            "SELECT s.id, s.event_id, e.title AS event_title, s.theater_id, t.name AS theater_name, s.starts_at
             FROM showtimes s
             JOIN events e ON e.id = s.event_id
             JOIN theaters t ON t.id = s.theater_id
             WHERE s.id = $1",
        )
        .bind(id)
        .fetch_optional(executor)
        .await
    }

    pub async fn find_theater_id_by_id<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
    ) -> Result<Option<Uuid>, sqlx::Error> {
        sqlx::query_scalar("SELECT theater_id FROM showtimes WHERE id = $1")
            .bind(id)
            .fetch_optional(executor)
            .await
    }

    pub async fn exists_by_id<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM showtimes WHERE id = $1)")
            .bind(id)
            .fetch_one(executor)
            .await
    }

    pub async fn exists_by_theater_and_starts_at<'e, E: PgExecutor<'e>>(
        executor: E,
        theater_id: Uuid,
        starts_at: DateTime<Utc>,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM showtimes WHERE theater_id = $1 AND starts_at = $2)",
        )
        .bind(theater_id)
        .bind(starts_at)
        .fetch_one(executor)
        .await
    }

    pub async fn save<'e, E: PgExecutor<'e>>(
        executor: E,
        event_id: Uuid,
        theater_id: Uuid,
        starts_at: DateTime<Utc>,
    ) -> Result<ShowtimeDetail, sqlx::Error> {
        sqlx::query_as(
            "WITH inserted AS (
                INSERT INTO showtimes (event_id, theater_id, starts_at)
                VALUES ($1, $2, $3)
                RETURNING id, event_id, theater_id, starts_at
             )
             SELECT i.id, i.event_id, e.title AS event_title, i.theater_id, t.name AS theater_name, i.starts_at
             FROM inserted i
             JOIN events e ON e.id = i.event_id
             JOIN theaters t ON t.id = i.theater_id",
        )
        .bind(event_id)
        .bind(theater_id)
        .bind(starts_at)
        .fetch_one(executor)
        .await
    }

    pub async fn update<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
        theater_id: Option<Uuid>,
        starts_at: DateTime<Utc>,
    ) -> Result<Option<ShowtimeDetail>, sqlx::Error> {
        sqlx::query_as(
            "WITH updated AS (
                UPDATE showtimes
                SET theater_id = COALESCE($1, theater_id),
                    starts_at = $2
                WHERE id = $3
                RETURNING id, event_id, theater_id, starts_at
             )
             SELECT u.id, u.event_id, e.title AS event_title, u.theater_id, t.name AS theater_name, u.starts_at
             FROM updated u
             JOIN events e ON e.id = u.event_id
             JOIN theaters t ON t.id = u.theater_id",
        )
        .bind(theater_id)
        .bind(starts_at)
        .bind(id)
        .fetch_optional(executor)
        .await
    }

    pub async fn delete_by_id<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let res = sqlx::query("DELETE FROM showtimes WHERE id = $1")
            .bind(id)
            .execute(executor)
            .await?;
        Ok(res.rows_affected() > 0)
    }
}
