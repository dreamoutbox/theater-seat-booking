use sqlx::PgExecutor;
use uuid::Uuid;

use crate::models::{Event, GatekeeperDetail};

pub struct GatekeeperRepository;

impl GatekeeperRepository {
    pub async fn assign<'e, E: PgExecutor<'e>>(
        executor: E,
        event_id: Uuid,
        user_id: Uuid,
        assigned_by: Option<Uuid>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO event_gatekeepers (event_id, user_id, assigned_by)
             VALUES ($1, $2, $3)
             ON CONFLICT (event_id, user_id) DO NOTHING",
        )
        .bind(event_id)
        .bind(user_id)
        .bind(assigned_by)
        .execute(executor)
        .await?;
        Ok(())
    }

    pub async fn remove<'e, E: PgExecutor<'e>>(
        executor: E,
        event_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let res = sqlx::query(
            "DELETE FROM event_gatekeepers
             WHERE event_id = $1 AND user_id = $2",
        )
        .bind(event_id)
        .bind(user_id)
        .execute(executor)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn find_by_event_id<'e, E: PgExecutor<'e>>(
        executor: E,
        event_id: Uuid,
    ) -> Result<Vec<GatekeeperDetail>, sqlx::Error> {
        sqlx::query_as(
            "SELECT u.id AS user_id, u.email, eg.assigned_at
             FROM event_gatekeepers eg
             JOIN users u ON u.id = eg.user_id
             WHERE eg.event_id = $1
             ORDER BY eg.assigned_at DESC",
        )
        .bind(event_id)
        .fetch_all(executor)
        .await
    }

    pub async fn is_assigned<'e, E: PgExecutor<'e>>(
        executor: E,
        user_id: Uuid,
        event_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM event_gatekeepers
                WHERE user_id = $1 AND event_id = $2
            )",
        )
        .bind(user_id)
        .bind(event_id)
        .fetch_one(executor)
        .await
    }

    pub async fn find_assigned_events_by_user_id<'e, E: PgExecutor<'e>>(
        executor: E,
        user_id: Uuid,
    ) -> Result<Vec<Event>, sqlx::Error> {
        sqlx::query_as(
            "SELECT e.id, e.title, e.max_seats_per_booking, e.max_seats_per_user, e.owner_id
             FROM event_gatekeepers eg
             JOIN events e ON e.id = eg.event_id
             WHERE eg.user_id = $1
             ORDER BY e.title",
        )
        .bind(user_id)
        .fetch_all(executor)
        .await
    }
}
