-- TimescaleDB Schema for AI Agent Decision Tracking
-- ====================================================
-- Creates hypertable for storing agent decisions with time-series optimization.

-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Decisions table (hypertable)
CREATE TABLE IF NOT EXISTS decisions (
    time TIMESTAMPTZ NOT NULL,
    decision_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id TEXT NOT NULL,
    decision_type TEXT NOT NULL,
    score FLOAT NOT NULL CHECK (score >= 0.0 AND score <= 1.0),
    approved BOOLEAN NOT NULL,
    stf_compliant BOOLEAN NOT NULL,
    stf_violations TEXT[],
    context JSONB,
    proposed_action TEXT,
    model_version TEXT,
    latency_ms FLOAT,
    requires_human_review BOOLEAN NOT NULL
);

-- Convert to hypertable (time-series optimization)
SELECT create_hypertable('decisions', 'time', if_not_exists => TRUE);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_agent_id ON decisions (agent_id, time DESC);
CREATE INDEX IF NOT EXISTS idx_decision_type ON decisions (decision_type, time DESC);
CREATE INDEX IF NOT EXISTS idx_approved ON decisions (approved, time DESC);
CREATE INDEX IF NOT EXISTS idx_stf_compliant ON decisions (stf_compliant, time DESC);
CREATE INDEX IF NOT EXISTS idx_requires_review ON decisions (requires_human_review, time DESC);

-- GIN index for JSONB context queries
CREATE INDEX IF NOT EXISTS idx_context ON decisions USING GIN (context);

-- GIN index for STF violations array
CREATE INDEX IF NOT EXISTS idx_stf_violations ON decisions USING GIN (stf_violations);

-- Retention policy: Keep 90 days of data
SELECT add_retention_policy('decisions', INTERVAL '90 days', if_not_exists => TRUE);

-- Continuous aggregates for common queries
-- ===========================================

-- Hourly approval rates by agent
CREATE MATERIALIZED VIEW IF NOT EXISTS decisions_hourly
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 hour', time) AS bucket,
    agent_id,
    decision_type,
    COUNT(*) AS total_decisions,
    SUM(CASE WHEN approved THEN 1 ELSE 0 END) AS approved_count,
    AVG(CASE WHEN approved THEN 1.0 ELSE 0.0 END) AS approval_rate,
    SUM(CASE WHEN stf_compliant THEN 1 ELSE 0 END) AS stf_compliant_count,
    AVG(CASE WHEN stf_compliant THEN 1.0 ELSE 0.0 END) AS stf_compliance_rate,
    AVG(score) AS avg_score,
    PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY latency_ms) AS p95_latency,
    PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY latency_ms) AS p99_latency
FROM decisions
GROUP BY bucket, agent_id, decision_type
WITH NO DATA;

-- Refresh policy for hourly aggregates
SELECT add_continuous_aggregate_policy('decisions_hourly',
    start_offset => INTERVAL '3 hours',
    end_offset => INTERVAL '1 hour',
    schedule_interval => INTERVAL '1 hour',
    if_not_exists => TRUE);

-- Daily agent performance summary
CREATE MATERIALIZED VIEW IF NOT EXISTS decisions_daily
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 day', time) AS bucket,
    agent_id,
    COUNT(*) AS total_decisions,
    SUM(CASE WHEN approved THEN 1 ELSE 0 END) AS approved_count,
    SUM(CASE WHEN stf_compliant THEN 1 ELSE 0 END) AS stf_compliant_count,
    SUM(CASE WHEN requires_human_review THEN 1 ELSE 0 END) AS human_review_count,
    AVG(score) AS avg_score,
    MIN(score) AS min_score,
    MAX(score) AS max_score,
    AVG(latency_ms) AS avg_latency,
    COUNT(DISTINCT decision_type) AS unique_decision_types
FROM decisions
GROUP BY bucket, agent_id
WITH NO DATA;

-- Refresh policy for daily aggregates
SELECT add_continuous_aggregate_policy('decisions_daily',
    start_offset => INTERVAL '3 days',
    end_offset => INTERVAL '1 day',
    schedule_interval => INTERVAL '1 day',
    if_not_exists => TRUE);

-- STF violations tracking
CREATE MATERIALIZED VIEW IF NOT EXISTS stf_violations_summary
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 hour', time) AS bucket,
    agent_id,
    unnest(stf_violations) AS violation,
    COUNT(*) AS violation_count
FROM decisions
WHERE NOT stf_compliant
GROUP BY bucket, agent_id, violation
WITH NO DATA;

-- Refresh policy for violations
SELECT add_continuous_aggregate_policy('stf_violations_summary',
    start_offset => INTERVAL '3 hours',
    end_offset => INTERVAL '1 hour',
    schedule_interval => INTERVAL '1 hour',
    if_not_exists => TRUE);

-- Useful queries (comments)
-- =========================

-- Get recent decisions for an agent:
-- SELECT * FROM decisions WHERE agent_id = 'test-agent' ORDER BY time DESC LIMIT 10;

-- Get approval rate for last 24 hours:
-- SELECT agent_id, AVG(CASE WHEN approved THEN 1.0 ELSE 0.0 END) as approval_rate
-- FROM decisions
-- WHERE time > NOW() - INTERVAL '24 hours'
-- GROUP BY agent_id;

-- Get STF violations by type:
-- SELECT violation, COUNT(*) as count
-- FROM stf_violations_summary
-- WHERE bucket > NOW() - INTERVAL '7 days'
-- GROUP BY violation
-- ORDER BY count DESC;

-- Get agent performance trends:
-- SELECT bucket, agent_id, approval_rate, stf_compliance_rate, avg_score
-- FROM decisions_hourly
-- WHERE bucket > NOW() - INTERVAL '7 days'
-- ORDER BY bucket DESC;

-- Find decisions requiring human review:
-- SELECT decision_id, agent_id, decision_type, score, stf_violations
-- FROM decisions
-- WHERE requires_human_review = TRUE AND time > NOW() - INTERVAL '1 day'
-- ORDER BY time DESC;
