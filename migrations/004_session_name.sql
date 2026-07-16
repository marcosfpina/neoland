-- Migration 004: Add session_name to agent session metadata
ALTER TABLE agent_session_metadata ADD COLUMN IF NOT EXISTS session_name TEXT NOT NULL DEFAULT 'Nova sessão';
