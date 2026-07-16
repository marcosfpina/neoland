//! Session state management via PostgreSQL.
//! Uses query_as! with explicit types to avoid DATABASE_URL requirement at
//! compile time.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct SessionState {
    pub session_id: Uuid,
    pub session_name: String,
    pub task_count: i32,
    pub last_activity: DateTime<Utc>,
    pub last_decision: Option<serde_json::Value>,
    pub active: bool,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SessionMessage {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

pub struct SessionManager {
    pool: PgPool,
}

impl SessionManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self), fields(session_id = %session_id))]
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

        // Fetch current state — named mapping via FromRow; safe under column reorder.
        let state = sqlx::query_as::<_, SessionState>(
            r#"
                SELECT session_id, session_name, task_count, last_activity, last_decision, active
                FROM agent_session_metadata
                WHERE session_id = $1
                "#,
        )
        .bind(session_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch session")?;

        Ok(state)
    }

    #[instrument(skip(self, decision_json), fields(session_id = %session_id))]
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

    #[instrument(skip(self))]
    pub async fn list_recent(&self, limit: i64) -> Result<Vec<SessionState>> {
        sqlx::query_as::<_, SessionState>(
            r#"
                SELECT session_id, session_name, task_count, last_activity, last_decision, active
                FROM agent_session_metadata
                ORDER BY last_activity DESC
                LIMIT $1
                "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .context("Failed to list sessions")
    }

    #[instrument(skip(self), fields(session_id = %session_id))]
    pub async fn set_session_name(&self, session_id: Uuid, name: &str) -> Result<()> {
        sqlx::query("UPDATE agent_session_metadata SET session_name = $2 WHERE session_id = $1")
            .bind(session_id)
            .bind(name)
            .execute(&self.pool)
            .await
            .context("Failed to update session name")?;
        Ok(())
    }

    #[instrument(skip(self), fields(session_id = %session_id))]
    pub async fn get_session_messages(&self, session_id: Uuid) -> Result<Vec<SessionMessage>> {
        sqlx::query_as::<_, SessionMessage>(
            r#"
                SELECT task as content, 'user' as role, created_at as timestamp
                FROM agent_sessions
                WHERE session_id = $1
                ORDER BY created_at ASC
                "#,
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch session messages")
    }
}
