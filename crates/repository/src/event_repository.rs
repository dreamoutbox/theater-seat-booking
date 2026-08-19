use sqlx::PgExecutor;
use uuid::Uuid;

use crate::models::Event;

pub struct EventRepository;

impl EventRepository {
    pub async fn find_all<'e, E: PgExecutor<'e>>(
        executor: E,
    ) -> Result<Vec<Event>, sqlx::Error> {
        sqlx::query_as(
            "SELECT id, title, max_seats_per_booking, max_seats_per_user, owner_id, event_number
             FROM events
             ORDER BY event_number, title",
        )
        .fetch_all(executor)
        .await
    }

    pub async fn find_by_owner_id<'e, E: PgExecutor<'e>>(
        executor: E,
        owner_id: Uuid,
    ) -> Result<Vec<Event>, sqlx::Error> {
        sqlx::query_as(
            "SELECT id, title, max_seats_per_booking, max_seats_per_user, owner_id, event_number
             FROM events
             WHERE owner_id = $1
             ORDER BY event_number, title",
        )
        .bind(owner_id)
        .fetch_all(executor)
        .await
    }

    pub async fn find_by_id<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
    ) -> Result<Option<Event>, sqlx::Error> {
        sqlx::query_as(
            "SELECT id, title, max_seats_per_booking, max_seats_per_user, owner_id, event_number
             FROM events
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(executor)
        .await
    }

    pub async fn find_by_title<'e, E: PgExecutor<'e>>(
        executor: E,
        title: &str,
    ) -> Result<Option<Event>, sqlx::Error> {
        sqlx::query_as(
            "SELECT id, title, max_seats_per_booking, max_seats_per_user, owner_id, event_number
             FROM events
             WHERE title = $1
             LIMIT 1",
        )
        .bind(title)
        .fetch_optional(executor)
        .await
    }

    pub async fn save<'e, E: PgExecutor<'e>>(
        executor: E,
        title: &str,
        max_seats_per_booking: i32,
        max_seats_per_user: i32,
        owner_id: Option<Uuid>,
    ) -> Result<Event, sqlx::Error> {
        sqlx::query_as(
            "INSERT INTO events (title, max_seats_per_booking, max_seats_per_user, owner_id)
             VALUES ($1, $2, $3, $4)
             RETURNING id, title, max_seats_per_booking, max_seats_per_user, owner_id, event_number",
        )
        .bind(title)
        .bind(max_seats_per_booking)
        .bind(max_seats_per_user)
        .bind(owner_id)
        .fetch_one(executor)
        .await
    }

    pub async fn update<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
        title: &str,
        max_seats_per_booking: i32,
        max_seats_per_user: i32,
        owner_id: Option<Uuid>,
    ) -> Result<Option<Event>, sqlx::Error> {
        sqlx::query_as(
            "UPDATE events
             SET title = $1,
                 max_seats_per_booking = $2,
                 max_seats_per_user = $3,
                 owner_id = COALESCE($4, owner_id)
             WHERE id = $5
             RETURNING id, title, max_seats_per_booking, max_seats_per_user, owner_id, event_number",
        )
        .bind(title)
        .bind(max_seats_per_booking)
        .bind(max_seats_per_user)
        .bind(owner_id)
        .bind(id)
        .fetch_optional(executor)
        .await
    }

    pub async fn delete_by_id<'e, E: PgExecutor<'e>>(
        executor: E,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let res = sqlx::query("DELETE FROM events WHERE id = $1")
            .bind(id)
            .execute(executor)
            .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn find_limits_by_showtime_id<'e, E: PgExecutor<'e>>(
        executor: E,
        showtime_id: Uuid,
    ) -> Result<Option<(i32, i32)>, sqlx::Error> {
        sqlx::query_as(
            "SELECT e.max_seats_per_booking, e.max_seats_per_user
             FROM showtimes s
             JOIN events e ON s.event_id = e.id
             WHERE s.id = $1",
        )
        .bind(showtime_id)
        .fetch_optional(executor)
        .await
    }
}
