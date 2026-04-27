"""
Observability Module - Metrics, Logging, and Monitoring
=======================================================
Provides comprehensive observability for AI Agent Hub.

Components:
- metrics: Prometheus metrics exporter
- structured_logger: JSON logging for ELK/Loki
- timescale_schema: TimescaleDB schema for time-series data

Usage:
    from observability import track_decision, log_decision, get_metrics

    # Track a decision (metrics + logs)
    track_decision(
        agent_id="claude",
        score=0.95,
        approved=True,
        stf_compliant=True,
        latency_ms=42.5,
        decision_type="code_generation",
        context={"file": "main.py"}
    )

    # Get Prometheus metrics
    metrics_data = get_metrics()
"""

from .metrics import (
    track_decision as track_metrics,
    get_metrics,
    get_metrics_content_type,
    get_health_status as get_metrics_health,
    metrics_registry
)

from .structured_logger import (
    log_decision as log_decision_event,
    log_system_event,
    get_logger
)

__all__ = [
    # Metrics
    'track_metrics',
    'get_metrics',
    'get_metrics_content_type',
    'get_metrics_health',
    'metrics_registry',

    # Logging
    'log_decision_event',
    'log_system_event',
    'get_logger',

    # Combined
    'track_decision'
]


def track_decision(
    agent_id: str,
    score: float,
    approved: bool,
    stf_compliant: bool,
    latency_ms: float,
    decision_type: str = "unknown",
    stf_violations: list = None,
    context: dict = None,
    metadata: dict = None
):
    """
    Track a decision in both metrics and logs.

    This is the main entrypoint for observability - combines
    Prometheus metrics tracking and structured logging.

    Args:
        agent_id: Agent identifier
        score: Decision score (0.0 - 1.0)
        approved: Whether decision was approved
        stf_compliant: STF compliance status
        latency_ms: Processing latency in milliseconds
        decision_type: Type of decision
        stf_violations: List of STF violations
        context: Decision context
        metadata: Additional metadata
    """
    violations = stf_violations or []

    # Track in Prometheus
    track_metrics(
        agent_id=agent_id,
        score=score,
        approved=approved,
        stf_compliant=stf_compliant,
        latency_ms=latency_ms,
        decision_type=decision_type,
        violations=violations
    )

    # Log structured event
    log_decision_event(
        agent_id=agent_id,
        decision_type=decision_type,
        score=score,
        approved=approved,
        stf_compliant=stf_compliant,
        stf_violations=violations,
        latency_ms=latency_ms,
        context=context,
        metadata=metadata
    )


__version__ = '1.0.0'
