#!/usr/bin/env python3
"""
Observability Metrics - Prometheus Exporter
===========================================
Exposes AI Agent metrics for Prometheus scraping.

Metrics Tracked:
- Decision latency (histogram)
- Decision throughput (counter)
- Approval rate (gauge)
- STF compliance rate (gauge)
- Score distribution (histogram)

Usage:
    from observability.metrics import metrics_registry, track_decision

    # Track a decision
    track_decision(
        agent_id="claude-code",
        score=0.95,
        approved=True,
        stf_compliant=True,
        latency_ms=45.2
    )
"""

import time
from prometheus_client import (
    Counter,
    Histogram,
    Gauge,
    Info,
    CollectorRegistry,
    generate_latest,
    CONTENT_TYPE_LATEST
)
from typing import Optional
import logging

logger = logging.getLogger(__name__)

# =============================================================================
# METRICS REGISTRY
# =============================================================================

# Create custom registry (allows multiple instances)
metrics_registry = CollectorRegistry()

# Info metric - System metadata
system_info = Info(
    'ai_agent_hub_info',
    'AI Agent Hub system information',
    registry=metrics_registry
)

system_info.info({
    'version': '1.0.0',
    'protocol': 'NEUTRON',
    'mode': 'STRICT_COMPLIANCE'
})

# =============================================================================
# DECISION METRICS
# =============================================================================

# Counter: Total decisions processed
decisions_total = Counter(
    'ai_agent_decisions_total',
    'Total number of agent decisions processed',
    ['agent_id', 'decision_type', 'approved'],
    registry=metrics_registry
)

# Histogram: Decision latency
decision_latency = Histogram(
    'ai_agent_decision_latency_seconds',
    'Decision processing latency in seconds',
    ['agent_id'],
    buckets=[0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0],  # Up to 5 seconds
    registry=metrics_registry
)

# Histogram: Score distribution
score_distribution = Histogram(
    'ai_agent_score_distribution',
    'Distribution of decision scores',
    ['agent_id'],
    buckets=[0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 0.95, 1.0],
    registry=metrics_registry
)

# Gauge: Current approval rate (rolling window)
approval_rate = Gauge(
    'ai_agent_approval_rate',
    'Current approval rate (0.0 - 1.0)',
    ['agent_id'],
    registry=metrics_registry
)

# Gauge: STF compliance rate
stf_compliance_rate = Gauge(
    'ai_agent_stf_compliance_rate',
    'STF compliance rate (0.0 - 1.0)',
    ['agent_id'],
    registry=metrics_registry
)

# Counter: STF violations
stf_violations_total = Counter(
    'ai_agent_stf_violations_total',
    'Total number of STF violations',
    ['agent_id', 'violation_type'],
    registry=metrics_registry
)

# Counter: Human reviews required
human_reviews_total = Counter(
    'ai_agent_human_reviews_total',
    'Total decisions requiring human review',
    ['agent_id', 'reason'],
    registry=metrics_registry
)

# =============================================================================
# TRACKING FUNCTIONS
# =============================================================================

# Rolling window for rate calculations
_approval_window = {}  # agent_id -> [bool, ...]
_stf_window = {}  # agent_id -> [bool, ...]
_window_size = 100  # Last 100 decisions


def track_decision(
    agent_id: str,
    score: float,
    approved: bool,
    stf_compliant: bool,
    latency_ms: float,
    decision_type: str = "unknown",
    violations: Optional[list] = None
):
    """
    Track a decision in Prometheus metrics.

    Args:
        agent_id: Agent identifier
        score: Decision score (0.0 - 1.0)
        approved: Whether decision was approved
        stf_compliant: Whether decision is STF compliant
        latency_ms: Processing latency in milliseconds
        decision_type: Type of decision
        violations: List of STF violations
    """
    try:
        # Convert latency to seconds
        latency_s = latency_ms / 1000.0

        # Increment decision counter
        decisions_total.labels(
            agent_id=agent_id,
            decision_type=decision_type,
            approved=str(approved)
        ).inc()

        # Track latency
        decision_latency.labels(agent_id=agent_id).observe(latency_s)

        # Track score distribution
        score_distribution.labels(agent_id=agent_id).observe(score)

        # Update rolling windows
        if agent_id not in _approval_window:
            _approval_window[agent_id] = []
            _stf_window[agent_id] = []

        _approval_window[agent_id].append(approved)
        _stf_window[agent_id].append(stf_compliant)

        # Trim windows
        if len(_approval_window[agent_id]) > _window_size:
            _approval_window[agent_id].pop(0)
        if len(_stf_window[agent_id]) > _window_size:
            _stf_window[agent_id].pop(0)

        # Update rates
        approval_rate.labels(agent_id=agent_id).set(
            sum(_approval_window[agent_id]) / len(_approval_window[agent_id])
        )
        stf_compliance_rate.labels(agent_id=agent_id).set(
            sum(_stf_window[agent_id]) / len(_stf_window[agent_id])
        )

        # Track violations
        if violations:
            for violation in violations:
                stf_violations_total.labels(
                    agent_id=agent_id,
                    violation_type=violation[:50]  # Truncate long violations
                ).inc()

        # Track human review
        if not approved:
            reason = "stf_violation" if not stf_compliant else "low_score"
            human_reviews_total.labels(agent_id=agent_id, reason=reason).inc()

        logger.debug(
            f"Metrics tracked: {agent_id} | "
            f"score={score:.3f} | "
            f"approved={approved} | "
            f"stf={stf_compliant} | "
            f"latency={latency_ms:.1f}ms"
        )

    except Exception as e:
        logger.error(f"Error tracking metrics: {e}")


def get_metrics() -> bytes:
    """
    Get current metrics in Prometheus exposition format.

    Returns:
        Metrics as bytes
    """
    return generate_latest(metrics_registry)


def get_metrics_content_type() -> str:
    """Get content type for metrics response"""
    return CONTENT_TYPE_LATEST


# =============================================================================
# HEALTH CHECK
# =============================================================================

def get_health_status() -> dict:
    """
    Get health status of observability system.

    Returns:
        Health status dictionary
    """
    try:
        # Check if metrics are collectible
        _ = generate_latest(metrics_registry)

        return {
            "status": "healthy",
            "metrics_registry": "operational",
            "tracked_agents": list(_approval_window.keys()),
            "window_size": _window_size
        }
    except Exception as e:
        return {
            "status": "unhealthy",
            "error": str(e)
        }


# =============================================================================
# EXAMPLE USAGE
# =============================================================================

if __name__ == '__main__':
    # Example tracking
    track_decision(
        agent_id="claude-code",
        score=0.95,
        approved=True,
        stf_compliant=True,
        latency_ms=42.5,
        decision_type="code_generation"
    )

    track_decision(
        agent_id="gemini-flash",
        score=0.65,
        approved=False,
        stf_compliant=False,
        latency_ms=120.3,
        decision_type="file_modification",
        violations=["Nix-First violation: pip install"]
    )

    # Get metrics
    print(get_metrics().decode('utf-8'))
