-- Migration 002: Agent session state for multi-agent pipeline

CREATE TABLE IF NOT EXISTS agent_sessions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id      UUID NOT NULL,
    task_id         UUID NOT NULL UNIQUE,
    task            TEXT NOT NULL,
    requester_role  TEXT NOT NULL,
    start_from      TEXT NOT NULL DEFAULT 'junior',
    junior_output   JSONB,
    senior_output   JSONB,
    architect_output JSONB,
    tech_leader_output JSONB,
    final_decision  TEXT CHECK (final_decision IN ('approve', 'reject', 'defer', 'escalate')),
    checkpoint_path TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at    TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_agent_sessions_session  ON agent_sessions(session_id);
CREATE INDEX IF NOT EXISTS idx_agent_sessions_decision ON agent_sessions(final_decision);
CREATE INDEX IF NOT EXISTS idx_agent_sessions_created  ON agent_sessions(created_at DESC);

CREATE TABLE IF NOT EXISTS agent_session_metadata (
    session_id    UUID PRIMARY KEY,
    task_count    INTEGER      NOT NULL DEFAULT 0,
    last_activity TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_decision JSONB,
    active        BOOLEAN      NOT NULL DEFAULT true,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
