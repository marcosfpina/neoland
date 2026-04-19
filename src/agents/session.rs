//! Session state management via PostgreSQL.
//! Uses query_as! with explicit types to avoid DATABASE_URL requirement at
//! compile time.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct SessionState {
    pub session_id: Uuid,
    pub task_count: i32,
    pub last_activity: DateTime<Utc>,
    pub last_decision: Option<serde_json::Value>,
    pub active: bool,
}

pub struct SessionManager {
    pool: PgPool,
}

impl SessionManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_or_create(&self, session_id: Uuid) -> Result<SessionState> {
        // Upsert session metadata
        sqlx::query(
            r#"
            INSERT INTO agent_session_metadata (session_id)
            VALUES ($1)
            ON CONFLICT (session_id) DO UPDATE SET last_activity = NOW()
            "#,
        )
        .bind(session_id)
        .execute(&self.pool)
        .await
        .context("Failed to upsert session")?;

        // Fetch current state
        let row: (Uuid, i32, DateTime<Utc>, Option<serde_json::Value>, bool) = sqlx::query_as(
            r#"
                SELECT session_id, task_count, last_activity, last_decision, active
                FROM agent_session_metadata
                WHERE session_id = $1
                "#,
        )
        .bind(session_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch session")?;

        Ok(SessionState {
            session_id: row.0,
            task_count: row.1,
            last_activity: row.2,
            last_decision: row.3,
            active: row.4,
        })
    }

    pub async fn update_after_pipeline(
        &self,
        session_id: Uuid,
        decision_json: serde_json::Value,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE agent_session_metadata
            SET task_count    = task_count + 1,
                last_activity = NOW(),
                last_decision = $2
            WHERE session_id = $1
            "#,
        )
        .bind(session_id)
        .bind(decision_json)
        .execute(&self.pool)
        .await
        .context("Failed to update session after pipeline")?;
        Ok(())
    }
}
