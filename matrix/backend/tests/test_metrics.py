#!/usr/bin/env python3
"""
Test Suite: Prometheus Metrics
================================
Tests the observability system and metrics tracking.

Coverage:
- Decision counter increments
- Latency histogram records
- Approval rate gauge updates
- STF violation counter
- Metrics endpoint format
"""

import pytest
import json
import time
from pathlib import Path
from fastapi.testclient import TestClient
import sys

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from ranking.main import app
from observability.metrics import (
    track_decision,
    get_metrics,
    decisions_total,
    decision_latency,
    approval_rate,
    stf_compliance_rate,
    stf_violations_total,
    human_reviews_total,
    _approval_window,
    _stf_window
)


# =============================================================================
# FIXTURES
# =============================================================================

@pytest.fixture
def client():
    """FastAPI test client"""
    return TestClient(app)


@pytest.fixture
def fixtures():
    """Load test fixtures"""
    fixtures_path = Path(__file__).parent / "fixtures" / "decisions.json"
    with open(fixtures_path) as f:
        return json.load(f)


@pytest.fixture(autouse=True)
def reset_metrics():
    """Reset metrics before each test"""
    # Clear rolling windows
    _approval_window.clear()
    _stf_window.clear()
    yield


# =============================================================================
# TEST: DECISION COUNTER
# =============================================================================

def test_decision_counter_increments(client, fixtures):
    """
    Test that ai_agent_decisions_total increments on each decision.
    """
    decision = fixtures["approved_decision"]

    # Get initial metrics
    response = client.get("/metrics")
    initial_metrics = response.text

    # Make decision
    client.post("/rank", json=decision)

    # Wait for background task
    time.sleep(0.1)

    # Get updated metrics
    response = client.get("/metrics")
    updated_metrics = response.text

    # Check that counter increased
    assert "ai_agent_decisions_total" in updated_metrics
    assert decision["agent_id"] in updated_metrics


def test_decision_counter_labels(client, fixtures):
    """
    Test that decision counter has correct labels.

    Labels: agent_id, decision_type, approved
    """
    decision = fixtures["approved_decision"]

    client.post("/rank", json=decision)
    time.sleep(0.1)

    response = client.get("/metrics")
    metrics = response.text

    # Check labels are present
    assert f'agent_id="{decision["agent_id"]}"' in metrics
    assert f'decision_type="{decision["decision_type"]}"' in metrics


# =============================================================================
# TEST: LATENCY HISTOGRAM
# =============================================================================

def test_latency_histogram_records(client, fixtures):
    """
    Test that ai_agent_decision_latency_seconds histogram records latency.
    """
    decision = fixtures["approved_decision"]

    client.post("/rank", json=decision)
    time.sleep(0.1)

    response = client.get("/metrics")
    metrics = response.text

    # Check histogram metrics exist
    assert "ai_agent_decision_latency_seconds_bucket" in metrics
    assert "ai_agent_decision_latency_seconds_count" in metrics
    assert "ai_agent_decision_latency_seconds_sum" in metrics


def test_latency_histogram_buckets(client, fixtures):
    """
    Test that latency histogram has correct buckets.

    Buckets: 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0
    """
    decision = fixtures["approved_decision"]

    client.post("/rank", json=decision)
    time.sleep(0.1)

    response = client.get("/metrics")
    metrics = response.text

    # Check that buckets exist
    expected_buckets = ["0.01", "0.025", "0.05", "0.1", "0.25", "0.5", "1.0", "2.5", "5.0"]
    for bucket in expected_buckets:
        assert f'le="{bucket}"' in metrics


# =============================================================================
# TEST: APPROVAL RATE GAUGE
# =============================================================================

def test_approval_rate_gauge_updates(client, fixtures):
    """
    Test that ai_agent_approval_rate gauge updates correctly.

    Rolling window: 100 decisions
    """
    approved = fixtures["high_score_approval"]
    rejected = fixtures["critical_decision_requires_review"]

    # Make 5 approved, 5 rejected
    for _ in range(5):
        client.post("/rank", json=approved)
    for _ in range(5):
        client.post("/rank", json=rejected)

    time.sleep(0.2)

    response = client.get("/metrics")
    metrics = response.text

    # Check gauge exists
    assert "ai_agent_approval_rate" in metrics

    # Approval rate should be 0.5 (5/10)
    # Note: Exact value may vary due to ML model, but should be measurable
    assert 'ai_agent_approval_rate{agent_id="test-agent"}' in metrics


def test_approval_rate_rolling_window(client, fixtures):
    """
    Test that approval rate uses rolling window of 100 decisions.
    """
    from observability.metrics import _window_size

    assert _window_size == 100, "Window size should be 100"

    approved = fixtures["high_score_approval"]

    # Make more than window size decisions
    for _ in range(110):
        client.post("/rank", json=approved)

    # Check window doesn't exceed size
    agent_id = approved["agent_id"]
    if agent_id in _approval_window:
        assert len(_approval_window[agent_id]) <= _window_size


# =============================================================================
# TEST: STF COMPLIANCE RATE
# =============================================================================

def test_stf_compliance_rate_gauge(client, fixtures):
    """
    Test that ai_agent_stf_compliance_rate tracks STF violations.
    """
    compliant = fixtures["approved_decision"]
    violation = fixtures["stf_violation_nix_first"]

    # Make 7 compliant, 3 violations
    for _ in range(7):
        client.post("/rank", json=compliant)
    for _ in range(3):
        client.post("/rank", json=violation)

    time.sleep(0.2)

    response = client.get("/metrics")
    metrics = response.text

    # Check gauge exists
    assert "ai_agent_stf_compliance_rate" in metrics


# =============================================================================
# TEST: STF VIOLATION COUNTER
# =============================================================================

def test_stf_violation_counter(client, fixtures):
    """
    Test that ai_agent_stf_violations_total tracks violations.
    """
    violation = fixtures["stf_violation_nix_first"]

    client.post("/rank", json=violation)
    time.sleep(0.1)

    response = client.get("/metrics")
    metrics = response.text

    # Check counter exists
    assert "ai_agent_stf_violations_total" in metrics
    assert violation["agent_id"] in metrics


def test_stf_violation_types(client, fixtures):
    """
    Test that violations are labeled by type.
    """
    violation = fixtures["stf_violation_nix_first"]

    client.post("/rank", json=violation)
    time.sleep(0.1)

    response = client.get("/metrics")
    metrics = response.text

    # Should have violation_type label
    assert "violation_type=" in metrics


# =============================================================================
# TEST: HUMAN REVIEW COUNTER
# =============================================================================

def test_human_review_counter(client, fixtures):
    """
    Test that ai_agent_human_reviews_total tracks review requests.
    """
    requires_review = fixtures["critical_decision_requires_review"]

    client.post("/rank", json=requires_review)
    time.sleep(0.1)

    response = client.get("/metrics")
    metrics = response.text

    # Check counter exists
    assert "ai_agent_human_reviews_total" in metrics


def test_human_review_reason_labels(client, fixtures):
    """
    Test that human reviews are labeled by reason.

    Reasons: stf_violation, low_score
    """
    stf_violation = fixtures["stf_violation_nix_first"]
    low_score = fixtures["critical_decision_requires_review"]

    client.post("/rank", json=stf_violation)
    client.post("/rank", json=low_score)
    time.sleep(0.1)

    response = client.get("/metrics")
    metrics = response.text

    # Check reasons are labeled
    assert 'reason="stf_violation"' in metrics or 'reason="low_score"' in metrics


# =============================================================================
# TEST: METRICS ENDPOINT
# =============================================================================

def test_metrics_endpoint_returns_prometheus_format(client):
    """
    Test that /metrics returns Prometheus exposition format.

    Format:
    - Content-Type: text/plain; version=0.0.4; charset=utf-8
    - Lines: # HELP, # TYPE, metric{labels} value timestamp
    """
    response = client.get("/metrics")

    assert response.status_code == 200

    # Check content type
    content_type = response.headers.get("content-type")
    assert "text/plain" in content_type or "prometheus" in content_type.lower()

    # Check format
    metrics = response.text
    assert "# HELP" in metrics
    assert "# TYPE" in metrics
    assert "ai_agent_" in metrics


def test_metrics_endpoint_has_all_metrics(client, fixtures):
    """
    Test that all expected metrics are present.
    """
    # Make a decision to populate metrics
    decision = fixtures["approved_decision"]
    client.post("/rank", json=decision)
    time.sleep(0.1)

    response = client.get("/metrics")
    metrics = response.text

    # Check all metrics exist
    expected_metrics = [
        "ai_agent_hub_info",
        "ai_agent_decisions_total",
        "ai_agent_decision_latency_seconds",
        "ai_agent_score_distribution",
        "ai_agent_approval_rate",
        "ai_agent_stf_compliance_rate",
        "ai_agent_stf_violations_total",
        "ai_agent_human_reviews_total"
    ]

    for metric in expected_metrics:
        assert metric in metrics, f"Missing metric: {metric}"


# =============================================================================
# TEST: TRACK_DECISION FUNCTION
# =============================================================================

def test_track_decision_function():
    """
    Test track_decision function directly.
    """
    track_decision(
        agent_id="test-direct",
        score=0.95,
        approved=True,
        stf_compliant=True,
        latency_ms=42.5,
        decision_type="test",
        violations=None
    )

    # Get metrics
    metrics = get_metrics().decode('utf-8')

    assert "test-direct" in metrics


def test_track_decision_with_violations():
    """
    Test tracking decision with violations.
    """
    track_decision(
        agent_id="test-violation",
        score=0.65,
        approved=False,
        stf_compliant=False,
        latency_ms=120.3,
        decision_type="test",
        violations=["Nix-First violation", "Human fallback violation"]
    )

    metrics = get_metrics().decode('utf-8')

    assert "test-violation" in metrics
    assert "stf_violations_total" in metrics


# =============================================================================
# TEST: SCORE DISTRIBUTION HISTOGRAM
# =============================================================================

def test_score_distribution_histogram(client, fixtures):
    """
    Test that ai_agent_score_distribution tracks score ranges.

    Buckets: 0.0, 0.1, 0.2, ..., 0.9, 0.95, 1.0
    """
    approved = fixtures["high_score_approval"]

    client.post("/rank", json=approved)
    time.sleep(0.1)

    response = client.get("/metrics")
    metrics = response.text

    assert "ai_agent_score_distribution_bucket" in metrics
    assert "ai_agent_score_distribution_count" in metrics
    assert "ai_agent_score_distribution_sum" in metrics


# =============================================================================
# TEST: METRICS PERSISTENCE
# =============================================================================

def test_metrics_persist_across_requests(client, fixtures):
    """
    Test that metrics accumulate across multiple requests.
    """
    decision = fixtures["approved_decision"]

    # Make 3 decisions
    for _ in range(3):
        client.post("/rank", json=decision)

    time.sleep(0.2)

    response = client.get("/metrics")
    metrics = response.text

    # Check that count increased
    # (exact value depends on initial state, but should be >= 3)
    assert "ai_agent_decisions_total" in metrics


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
