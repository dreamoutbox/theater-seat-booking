use sqlx::PgExecutor;
use uuid::Uuid;

use crate::models::{User, UserRole};

pub struct UserRepository;

impl UserRepository {
    pub async fn exists_by_email<'e, E: PgExecutor<'e>>(
        executor: E,
        email: &str,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
            .bind(email)
            .fetch_one(executor)
            .await
    }

    pub async fn save<'e, E: PgExecutor<'e>>(
        executor: E,
        email: &str,
        password_hash: &str,
        role: UserRole,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as(
            "INSERT INTO users (email, password_hash, role)
             VALUES ($1, $2, $3)
             RETURNING id, email, password_hash, role, created_at",
        )
        .bind(email)
        .bind(password_hash)
        .bind(role)
        .fetch_one(executor)
        .await
    }

    pub async fn find_by_email<'e, E: PgExecutor<'e>>(
        executor: E,
        email: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as(
            "SELECT id, email, password_hash, role, created_at
             FROM users
             WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(executor)
        .await
    }

    pub async fn find_by_id<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as(
            "SELECT id, email, password_hash, role, created_at
             FROM users
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(executor)
        .await
    }
}
