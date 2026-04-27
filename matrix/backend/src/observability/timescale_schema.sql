-- =============================================================================
-- TimescaleDB Schema for AI Agent Hub
-- =============================================================================
-- Creates hypertables for time-series storage of agent decisions and metrics.
--
-- Usage:
--     psql -U postgres -d ai_agent_hub -f timescale_schema.sql
--
-- Requirements:
--     - PostgreSQL 12+
--     - TimescaleDB extension
-- =============================================================================

-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- =============================================================================
-- DECISIONS TABLE
-- =============================================================================

CREATE TABLE IF NOT EXISTS decisions (
    time TIMESTAMPTZ NOT NULL,
    agent_id TEXT NOT NULL,
    decision_id UUID NOT NULL DEFAULT gen_random_uuid(),
    decision_type TEXT NOT NULL,
    score DOUBLE PRECISION NOT NULL CHECK (score >= 0.0 AND score <= 1.0),
    approved BOOLEAN NOT NULL,
    requires_human_review BOOLEAN NOT NULL,
    stf_compliant BOOLEAN NOT NULL,
    stf_violations JSONB,
    latency_ms DOUBLE PRECISION,
    context JSONB,
    metadata JSONB,
    PRIMARY KEY (time, decision_id)
);

-- Create hypertable for time-series optimization
SELECT create_hypertable('decisions', 'time', if_not_exists => TRUE);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_decisions_agent_id ON decisions (agent_id, time DESC);
CREATE INDEX IF NOT EXISTS idx_decisions_approved ON decisions (approved, time DESC);
CREATE INDEX IF NOT EXISTS idx_decisions_stf_compliant ON decisions (stf_compliant, time DESC);
CREATE INDEX IF NOT EXISTS idx_decisions_type ON decisions (decision_type, time DESC);

-- GIN index for JSON fields
CREATE INDEX IF NOT EXISTS idx_decisions_violations ON decisions USING GIN (stf_violations);
CREATE INDEX IF NOT EXISTS idx_decisions_context ON decisions USING GIN (context);

-- =============================================================================
-- METRICS TABLE
-- =============================================================================

CREATE TABLE IF NOT EXISTS metrics (
    time TIMESTAMPTZ NOT NULL,
    metric_name TEXT NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    labels JSONB,
    PRIMARY KEY (time, metric_name, labels)
);

-- Create hypertable
SELECT create_hypertable('metrics', 'time', if_not_exists => TRUE);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_metrics_name ON metrics (metric_name, time DESC);
CREATE INDEX IF NOT EXISTS idx_metrics_labels ON metrics USING GIN (labels);

-- =============================================================================
-- STF VIOLATIONS TABLE
-- =============================================================================

CREATE TABLE IF NOT EXISTS stf_violations (
    time TIMESTAMPTZ NOT NULL,
    agent_id TEXT NOT NULL,
    decision_id UUID NOT NULL,
    violation_type TEXT NOT NULL,
    violation_detail TEXT,
    severity TEXT CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    PRIMARY KEY (time, decision_id, violation_type)
);

-- Create hypertable
SELECT create_hypertable('stf_violations', 'time', if_not_exists => TRUE);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_violations_agent ON stf_violations (agent_id, time DESC);
CREATE INDEX IF NOT EXISTS idx_violations_type ON stf_violations (violation_type, time DESC);
CREATE INDEX IF NOT EXISTS idx_violations_severity ON stf_violations (severity, time DESC);

-- =============================================================================
-- FEEDBACK TABLE
-- =============================================================================

CREATE TABLE IF NOT EXISTS feedback (
    time TIMESTAMPTZ NOT NULL,
    feedback_id UUID NOT NULL DEFAULT gen_random_uuid(),
    decision_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    feedback_type TEXT NOT NULL CHECK (feedback_type IN ('approved', 'rejected', 'modified', 'reverted')),
    outcome_score DOUBLE PRECISION NOT NULL CHECK (outcome_score >= 0.0 AND outcome_score <= 1.0),
    user_id TEXT,
    comments TEXT,
    metadata JSONB,
    PRIMARY KEY (time, feedback_id)
);

-- Create hypertable
SELECT create_hypertable('feedback', 'time', if_not_exists => TRUE);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_feedback_decision ON feedback (decision_id, time DESC);
CREATE INDEX IF NOT EXISTS idx_feedback_agent ON feedback (agent_id, time DESC);
CREATE INDEX IF NOT EXISTS idx_feedback_type ON feedback (feedback_type, time DESC);
CREATE INDEX IF NOT EXISTS idx_feedback_user ON feedback (user_id, time DESC);

-- =============================================================================
-- AGENT PERFORMANCE TABLE
-- =============================================================================

CREATE TABLE IF NOT EXISTS agent_performance (
    time TIMESTAMPTZ NOT NULL,
    agent_id TEXT NOT NULL,
    period_minutes INTEGER NOT NULL, -- Aggregation period (5, 15, 60, etc.)
    decisions_count INTEGER NOT NULL,
    approval_rate DOUBLE PRECISION,
    stf_compliance_rate DOUBLE PRECISION,
    avg_score DOUBLE PRECISION,
    avg_latency_ms DOUBLE PRECISION,
    p50_latency_ms DOUBLE PRECISION,
    p95_latency_ms DOUBLE PRECISION,
    p99_latency_ms DOUBLE PRECISION,
    PRIMARY KEY (time, agent_id, period_minutes)
);

-- Create hypertable
SELECT create_hypertable('agent_performance', 'time', if_not_exists => TRUE);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_perf_agent ON agent_performance (agent_id, time DESC);

-- =============================================================================
-- CONTINUOUS AGGREGATES (TimescaleDB)
-- =============================================================================

-- 5-minute aggregation
CREATE MATERIALIZED VIEW IF NOT EXISTS agent_performance_5m
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('5 minutes', time) AS bucket,
    agent_id,
    COUNT(*) AS decisions_count,
    AVG(score) AS avg_score,
    AVG(CASE WHEN approved THEN 1.0 ELSE 0.0 END) AS approval_rate,
    AVG(CASE WHEN stf_compliant THEN 1.0 ELSE 0.0 END) AS stf_compliance_rate,
    AVG(latency_ms) AS avg_latency_ms,
    PERCENTILE_CONT(0.50) WITHIN GROUP (ORDER BY latency_ms) AS p50_latency_ms,
    PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY latency_ms) AS p95_latency_ms,
    PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY latency_ms) AS p99_latency_ms
FROM decisions
GROUP BY bucket, agent_id
WITH NO DATA;

-- Refresh policy (refresh every 1 minute, looking back 5 minutes)
SELECT add_continuous_aggregate_policy('agent_performance_5m',
    start_offset => INTERVAL '10 minutes',
    end_offset => INTERVAL '1 minute',
    schedule_interval => INTERVAL '1 minute',
    if_not_exists => TRUE
);

-- 1-hour aggregation
CREATE MATERIALIZED VIEW IF NOT EXISTS agent_performance_1h
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 hour', time) AS bucket,
    agent_id,
    COUNT(*) AS decisions_count,
    AVG(score) AS avg_score,
    AVG(CASE WHEN approved THEN 1.0 ELSE 0.0 END) AS approval_rate,
    AVG(CASE WHEN stf_compliant THEN 1.0 ELSE 0.0 END) AS stf_compliance_rate,
    AVG(latency_ms) AS avg_latency_ms,
    PERCENTILE_CONT(0.50) WITHIN GROUP (ORDER BY latency_ms) AS p50_latency_ms,
    PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY latency_ms) AS p95_latency_ms,
    PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY latency_ms) AS p99_latency_ms
FROM decisions
GROUP BY bucket, agent_id
WITH NO DATA;

-- Refresh policy
SELECT add_continuous_aggregate_policy('agent_performance_1h',
    start_offset => INTERVAL '3 hours',
    end_offset => INTERVAL '5 minutes',
    schedule_interval => INTERVAL '5 minutes',
    if_not_exists => TRUE
);

-- =============================================================================
-- DATA RETENTION POLICIES
-- =============================================================================

-- Retain raw decisions for 30 days
SELECT add_retention_policy('decisions', INTERVAL '30 days', if_not_exists => TRUE);

-- Retain raw metrics for 15 days
SELECT add_retention_policy('metrics', INTERVAL '15 days', if_not_exists => TRUE);

-- Retain violations for 90 days (compliance tracking)
SELECT add_retention_policy('stf_violations', INTERVAL '90 days', if_not_exists => TRUE);

-- =============================================================================
-- HELPER FUNCTIONS
-- =============================================================================

-- Function: Get agent performance summary
CREATE OR REPLACE FUNCTION get_agent_summary(
    p_agent_id TEXT,
    p_hours INTEGER DEFAULT 24
)
RETURNS TABLE (
    total_decisions BIGINT,
    approval_rate DOUBLE PRECISION,
    stf_compliance_rate DOUBLE PRECISION,
    avg_score DOUBLE PRECISION,
    avg_latency_ms DOUBLE PRECISION
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        COUNT(*)::BIGINT,
        AVG(CASE WHEN approved THEN 1.0 ELSE 0.0 END),
        AVG(CASE WHEN stf_compliant THEN 1.0 ELSE 0.0 END),
        AVG(score),
        AVG(latency_ms)
    FROM decisions
    WHERE agent_id = p_agent_id
      AND time > NOW() - (p_hours || ' hours')::INTERVAL;
END;
$$ LANGUAGE plpgsql;

-- Function: Get top STF violations
CREATE OR REPLACE FUNCTION get_top_violations(
    p_limit INTEGER DEFAULT 10,
    p_hours INTEGER DEFAULT 24
)
RETURNS TABLE (
    violation_type TEXT,
    violation_count BIGINT,
    affected_agents TEXT[]
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        v.violation_type,
        COUNT(*)::BIGINT,
        ARRAY_AGG(DISTINCT v.agent_id)
    FROM stf_violations v
    WHERE v.time > NOW() - (p_hours || ' hours')::INTERVAL
    GROUP BY v.violation_type
    ORDER BY COUNT(*) DESC
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql;

-- =============================================================================
-- GRANTS (Adjust as needed)
-- =============================================================================

-- Grant access to ai_agent_hub role (create role first if needed)
-- CREATE ROLE ai_agent_hub WITH LOGIN PASSWORD 'secure_password';
-- GRANT SELECT, INSERT ON ALL TABLES IN SCHEMA public TO ai_agent_hub;
-- GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO ai_agent_hub;

-- =============================================================================
-- VERIFICATION
-- =============================================================================

-- Verify hypertables
SELECT * FROM timescaledb_information.hypertables
WHERE hypertable_schema = 'public';

-- Verify continuous aggregates
SELECT * FROM timescaledb_information.continuous_aggregates;

-- Verify retention policies
SELECT * FROM timescaledb_information.jobs
WHERE proc_name IN ('policy_retention', 'policy_refresh_continuous_aggregate');

\echo 'TimescaleDB schema created successfully!'
\echo 'Hypertables: decisions, metrics, stf_violations, agent_performance'
\echo 'Continuous aggregates: agent_performance_5m, agent_performance_1h'
\echo 'Retention policies configured'
