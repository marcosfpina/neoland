#!/usr/bin/env python3
"""
Test Suite: Integration Tests (E2E)
====================================
End-to-end tests for the AI Agent Ranking System.

Coverage:
- /rank endpoint updates Prometheus metrics
- STF violation triggers human review
- Metrics endpoint returns correct format
- Health check endpoint
- STF context endpoint
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


# =============================================================================
# TEST: RANK → METRICS INTEGRATION
# =============================================================================

def test_rank_endpoint_updates_prometheus_metrics(client, fixtures):
    """
    Test that making a decision via /rank updates Prometheus metrics.

    Flow:
    1. Make decision via POST /rank
    2. Wait for background task
    3. Check GET /metrics for updated values
    """
    decision = fixtures["approved_decision"]

    # Get initial metrics
    metrics_before = client.get("/metrics").text

    # Make decision
    rank_response = client.post("/rank", json=decision)
    assert rank_response.status_code == 200

    # Wait for background task to complete
    time.sleep(0.2)

    # Get updated metrics
    metrics_after = client.get("/metrics").text

    # Verify metrics were updated
    assert len(metrics_after) > len(metrics_before), "Metrics should have increased"
    assert decision["agent_id"] in metrics_after, "Agent ID should appear in metrics"


def test_multiple_decisions_update_metrics(client, fixtures):
    """
    Test that multiple decisions accumulate in metrics.
    """
    decisions = [
        fixtures["approved_decision"],
        fixtures["high_score_approval"],
        fixtures["critical_decision_requires_review"]
    ]

    # Make multiple decisions
    for decision in decisions:
        response = client.post("/rank", json=decision)
        assert response.status_code == 200

    time.sleep(0.3)

    # Get metrics
    metrics = client.get("/metrics").text

    # Check that counter shows multiple decisions
    assert "ai_agent_decisions_total" in metrics


# =============================================================================
# TEST: STF VIOLATION → HUMAN REVIEW FLOW
# =============================================================================

def test_stf_violation_triggers_human_review(client, fixtures):
    """
    Test complete flow:
    1. POST decision with STF violation
    2. Verify rejection + requires_human_review
    3. Verify metrics show violation
    4. Verify human_reviews_total incremented
    """
    violation = fixtures["stf_violation_nix_first"]

    # Step 1: Make decision
    rank_response = client.post("/rank", json=violation)
    assert rank_response.status_code == 200

    rank_data = rank_response.json()

    # Step 2: Verify rejection
    assert rank_data["approved"] is False, "STF violation should block approval"
    assert rank_data["requires_human_review"] is True, "Should require human review"
    assert rank_data["stf_compliant"] is False, "Should be non-compliant"
    assert len(rank_data["stf_violations"]) > 0, "Should have violation details"

    time.sleep(0.2)

    # Step 3: Verify metrics
    metrics = client.get("/metrics").text

    assert "ai_agent_stf_violations_total" in metrics, "Should track violations"
    assert "ai_agent_human_reviews_total" in metrics, "Should track human reviews"
    assert 'reason="stf_violation"' in metrics, "Should label reason"


def test_low_score_triggers_human_review(client, fixtures):
    """
    Test that low score (< 0.90) triggers human review even without STF violation.
    """
    critical = fixtures["critical_decision_requires_review"]

    rank_response = client.post("/rank", json=critical)
    assert rank_response.status_code == 200

    data = rank_response.json()

    # Should require human review due to low score
    assert data["requires_human_review"] is True


# =============================================================================
# TEST: HEALTH CHECK INTEGRATION
# =============================================================================

def test_health_endpoint(client):
    """
    Test /health endpoint returns system status.
    """
    response = client.get("/health")

    assert response.status_code == 200
    data = response.json()

    # Check required fields
    assert "status" in data
    assert "mode" in data
    assert "stf_protocol" in data
    assert "ml_model" in data

    # Check values
    assert data["status"] == "ok"
    assert data["mode"] == "strict_compliance"


def test_health_shows_stf_status(client):
    """
    Test that health endpoint shows STF parser status.
    """
    response = client.get("/health")
    data = response.json()

    # STF should be loaded or not_loaded
    assert data["stf_protocol"] in ["loaded", "not_loaded"]

    if data["stf_protocol"] == "loaded":
        assert "stf_version" in data
        assert data["stf_version"] is not None


def test_health_shows_ml_model_status(client):
    """
    Test that health endpoint shows ML model status.
    """
    response = client.get("/health")
    data = response.json()

    # ML model should be loaded or not_loaded
    assert data["ml_model"] in ["loaded", "not_loaded"]

    if data["ml_model"] == "loaded":
        assert "ml_version" in data


# =============================================================================
# TEST: STF CONTEXT ENDPOINT
# =============================================================================

def test_stf_context_endpoint_markdown(client):
    """
    Test /stf/context returns STF directives in markdown format.
    """
    response = client.get("/stf/context?format=markdown")

    # May return 503 if STF not loaded, which is acceptable
    if response.status_code == 200:
        data = response.json()
        assert "format" in data
        assert "context" in data
        assert data["format"] == "markdown"
        assert isinstance(data["context"], str)


def test_stf_context_endpoint_json(client):
    """
    Test /stf/context returns STF directives in JSON format.
    """
    response = client.get("/stf/context?format=json")

    # May return 503 if STF not loaded
    if response.status_code == 200:
        data = response.json()
        assert data["format"] == "json"


def test_stf_context_invalid_format(client):
    """
    Test that invalid format returns 400 error.
    """
    response = client.get("/stf/context?format=invalid")

    # Should return 400 or 503
    assert response.status_code in [400, 503]


# =============================================================================
# TEST: METRICS ENDPOINT FORMAT
# =============================================================================

def test_metrics_endpoint_returns_prometheus_format(client):
    """
    Test that /metrics returns valid Prometheus exposition format.
    """
    response = client.get("/metrics")

    assert response.status_code == 200

    # Check content type
    content_type = response.headers.get("content-type", "")
    assert "text/plain" in content_type.lower() or "prometheus" in content_type.lower()

    metrics = response.text

    # Check Prometheus format elements
    assert "# HELP" in metrics, "Should have HELP comments"
    assert "# TYPE" in metrics, "Should have TYPE declarations"
    assert "ai_agent_" in metrics, "Should have ai_agent metrics"


def test_metrics_parseable(client, fixtures):
    """
    Test that metrics can be parsed (basic validation).
    """
    # Make a decision to generate metrics
    client.post("/rank", json=fixtures["approved_decision"])
    time.sleep(0.1)

    response = client.get("/metrics")
    metrics = response.text

    # Split into lines
    lines = metrics.split('\n')

    # Check structure
    help_lines = [l for l in lines if l.startswith('# HELP')]
    type_lines = [l for l in lines if l.startswith('# TYPE')]
    metric_lines = [l for l in lines if l and not l.startswith('#')]

    assert len(help_lines) > 0, "Should have HELP lines"
    assert len(type_lines) > 0, "Should have TYPE lines"
    assert len(metric_lines) > 0, "Should have metric values"


# =============================================================================
# TEST: ERROR HANDLING
# =============================================================================

def test_invalid_endpoint_returns_404(client):
    """
    Test that invalid endpoints return 404.
    """
    response = client.get("/invalid-endpoint")
    assert response.status_code == 404


def test_rank_with_malformed_json(client):
    """
    Test that malformed JSON returns 422.
    """
    response = client.post(
        "/rank",
        data="not json",
        headers={"Content-Type": "application/json"}
    )

    assert response.status_code == 422


# =============================================================================
# TEST: COMPLETE E2E SCENARIO
# =============================================================================

def test_complete_decision_flow(client, fixtures):
    """
    Test complete decision flow from request to metrics.

    Scenario:
    1. System is healthy
    2. Make approved decision
    3. Verify approval
    4. Check metrics updated
    5. Make rejected decision
    6. Verify rejection
    7. Check metrics reflect both decisions
    """
    # Step 1: Check health
    health = client.get("/health")
    assert health.status_code == 200

    # Step 2: Make approved decision
    approved = fixtures["high_score_approval"]
    approved_response = client.post("/rank", json=approved)
    assert approved_response.status_code == 200
    approved_data = approved_response.json()
    assert approved_data["approved"] is True

    # Step 3: Make rejected decision
    rejected = fixtures["stf_violation_nix_first"]
    rejected_response = client.post("/rank", json=rejected)
    assert rejected_response.status_code == 200
    rejected_data = rejected_response.json()
    assert rejected_data["approved"] is False

    time.sleep(0.3)

    # Step 4: Check metrics
    metrics = client.get("/metrics").text

    # Should show both approved and rejected
    assert "ai_agent_decisions_total" in metrics
    assert approved["agent_id"] in metrics

    # Should show STF violations
    assert "ai_agent_stf_violations_total" in metrics

    # Should show human reviews
    assert "ai_agent_human_reviews_total" in metrics


def test_high_throughput_scenario(client, fixtures):
    """
    Test system under high throughput (100 decisions).
    """
    decision = fixtures["approved_decision"]

    # Make 100 decisions
    for i in range(100):
        response = client.post("/rank", json=decision)
        assert response.status_code == 200

    time.sleep(0.5)

    # Check metrics
    metrics = client.get("/metrics").text
    assert "ai_agent_decisions_total" in metrics


# =============================================================================
# TEST: CONCURRENT REQUESTS
# =============================================================================

def test_concurrent_rank_and_metrics(client, fixtures):
    """
    Test that /rank and /metrics can be called concurrently.
    """
    import concurrent.futures

    decision = fixtures["approved_decision"]

    def rank_request():
        return client.post("/rank", json=decision).status_code == 200

    def metrics_request():
        return client.get("/metrics").status_code == 200

    # Make 5 rank + 5 metrics requests concurrently
    with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
        rank_futures = [executor.submit(rank_request) for _ in range(5)]
        metrics_futures = [executor.submit(metrics_request) for _ in range(5)]

        all_futures = rank_futures + metrics_futures
        results = [f.result() for f in concurrent.futures.as_completed(all_futures)]

    # All requests should succeed
    assert all(results), "All concurrent requests should succeed"


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
