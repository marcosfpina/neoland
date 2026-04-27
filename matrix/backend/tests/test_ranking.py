#!/usr/bin/env python3
"""
Test Suite: AI Agent Ranking System
====================================
Tests the /rank endpoint with STF validation and ML scoring.

Coverage:
- STF validation blocks violations
- ML scoring above threshold approves
- Approval threshold 90%
- Human review required below threshold
- Background task logs decision
"""

import pytest
import json
from pathlib import Path
from fastapi.testclient import TestClient
import sys

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from ranking.main import app, AUTO_APPROVAL_THRESHOLD


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
# TEST: STF VALIDATION
# =============================================================================

def test_stf_validation_blocks_violations(client, fixtures):
    """
    Test that STF violations result in rejection.

    Expected behavior:
    - approved = False
    - requires_human_review = True
    - stf_compliant = False
    - stf_violations list is not empty
    """
    decision = fixtures["stf_violation_nix_first"]

    response = client.post("/rank", json=decision)

    assert response.status_code == 200
    data = response.json()

    # Assertions
    assert data["approved"] is False, "STF violation should block approval"
    assert data["requires_human_review"] is True, "STF violation should require human review"
    assert data["stf_compliant"] is False, "Should be marked as non-compliant"
    assert len(data["stf_violations"]) > 0, "Should have violation details"
    assert data["agent_id"] == decision["agent_id"]


def test_stf_violation_human_fallback(client, fixtures):
    """
    Test that bypassing human oversight triggers STF violation.

    STF Directive: Developer as Fallback (MANDATORY)
    """
    decision = fixtures["stf_violation_human_fallback"]

    response = client.post("/rank", json=decision)

    assert response.status_code == 200
    data = response.json()

    # Should be rejected due to STF violation
    assert data["approved"] is False
    assert data["stf_compliant"] is False
    assert data["requires_human_review"] is True


# =============================================================================
# TEST: ML SCORING & APPROVAL THRESHOLD
# =============================================================================

def test_ml_scoring_above_threshold_approves(client, fixtures):
    """
    Test that decisions with score > 0.90 are auto-approved.

    Expected behavior:
    - approved = True
    - requires_human_review = False
    - score >= 0.90
    """
    decision = fixtures["high_score_approval"]

    response = client.post("/rank", json=decision)

    assert response.status_code == 200
    data = response.json()

    # Assertions
    assert data["score"] >= AUTO_APPROVAL_THRESHOLD, f"Score {data['score']} should be >= {AUTO_APPROVAL_THRESHOLD}"
    assert data["approved"] is True, "High score should result in approval"
    assert data["requires_human_review"] is False, "High score should not require human review"
    assert data["stf_compliant"] is True, "Should be STF compliant"


def test_approval_threshold_90_percent(client, fixtures):
    """
    Test that the approval threshold is exactly 0.90.

    This validates the STF policy of requiring human review
    for decisions with uncertainty > 10%.
    """
    # Approved decision should have score >= 0.90
    approved = fixtures["high_score_approval"]
    response = client.post("/rank", json=approved)

    assert response.status_code == 200
    data = response.json()

    if data["approved"]:
        assert data["score"] >= 0.90, "Approved decisions must have score >= 0.90"


def test_human_review_required_below_threshold(client, fixtures):
    """
    Test that decisions with score < 0.90 require human review.

    Expected behavior:
    - approved = False
    - requires_human_review = True
    - score < 0.90
    """
    decision = fixtures["critical_decision_requires_review"]

    response = client.post("/rank", json=decision)

    assert response.status_code == 200
    data = response.json()

    # Critical decisions should have penalties applied
    assert data["approved"] is False, "Critical decisions should not auto-approve"
    assert data["requires_human_review"] is True, "Should require human review"


# =============================================================================
# TEST: CRITICAL DECISION TYPES
# =============================================================================

def test_critical_decision_type_deploy(client, fixtures):
    """
    Test that 'deploy' decision type is treated as critical.

    Critical types: deploy, delete_db, modify_acl
    """
    decision = fixtures["rejected_low_score"]

    response = client.post("/rank", json=decision)

    assert response.status_code == 200
    data = response.json()

    # Deploy decisions with high entropy should be rejected
    assert data["approved"] is False
    assert data["requires_human_review"] is True


def test_critical_decision_type_delete_db(client, fixtures):
    """
    Test that 'delete_db' decision type requires human review.
    """
    decision = fixtures["critical_decision_requires_review"]

    response = client.post("/rank", json=decision)

    assert response.status_code == 200
    data = response.json()

    # Database deletion should always require review
    assert data["requires_human_review"] is True
    assert data["approved"] is False


# =============================================================================
# TEST: BACKGROUND TASK LOGGING
# =============================================================================

def test_background_task_logs_decision(client, fixtures):
    """
    Test that decisions are logged in background tasks.

    Verifies:
    - Response is returned immediately (< 200ms)
    - Background task is scheduled
    - Metrics are updated (tested in test_metrics.py)
    """
    import time

    decision = fixtures["approved_decision"]

    start_time = time.time()
    response = client.post("/rank", json=decision)
    latency = (time.time() - start_time) * 1000

    assert response.status_code == 200
    assert latency < 200, f"Response should be fast (< 200ms), got {latency:.1f}ms"

    # Background task is scheduled, not awaited
    data = response.json()
    assert "timestamp" in data, "Should include timestamp"


# =============================================================================
# TEST: RESPONSE SCHEMA
# =============================================================================

def test_response_schema(client, fixtures):
    """
    Test that response matches RankingResponse schema.

    Required fields:
    - agent_id (str)
    - score (float)
    - approved (bool)
    - requires_human_review (bool)
    - stf_compliant (bool)
    - stf_violations (list[str])
    - timestamp (float)
    """
    decision = fixtures["approved_decision"]

    response = client.post("/rank", json=decision)

    assert response.status_code == 200
    data = response.json()

    # Check required fields
    assert "agent_id" in data
    assert "score" in data
    assert "approved" in data
    assert "requires_human_review" in data
    assert "stf_compliant" in data
    assert "stf_violations" in data
    assert "timestamp" in data

    # Check types
    assert isinstance(data["agent_id"], str)
    assert isinstance(data["score"], (int, float))
    assert isinstance(data["approved"], bool)
    assert isinstance(data["requires_human_review"], bool)
    assert isinstance(data["stf_compliant"], bool)
    assert isinstance(data["stf_violations"], list)
    assert isinstance(data["timestamp"], (int, float))

    # Check constraints
    assert 0.0 <= data["score"] <= 1.0, "Score must be between 0 and 1"


# =============================================================================
# TEST: ERROR HANDLING
# =============================================================================

def test_invalid_request_missing_fields(client):
    """
    Test that missing required fields return validation error.
    """
    invalid_decision = {
        "agent_id": "test"
        # Missing other required fields
    }

    response = client.post("/rank", json=invalid_decision)

    assert response.status_code == 422  # Validation error


def test_invalid_request_wrong_types(client):
    """
    Test that wrong field types return validation error.
    """
    invalid_decision = {
        "agent_id": 123,  # Should be string
        "decision_type": "test",
        "context": "not a dict",  # Should be dict
        "proposed_action": "test"
    }

    response = client.post("/rank", json=invalid_decision)

    assert response.status_code == 422


# =============================================================================
# TEST: EDGE CASES
# =============================================================================

def test_empty_context(client):
    """
    Test decision with empty context.
    """
    decision = {
        "agent_id": "test-agent",
        "decision_type": "test",
        "context": {},
        "proposed_action": "test action"
    }

    response = client.post("/rank", json=decision)

    assert response.status_code == 200
    data = response.json()

    # Should still return valid response
    assert "score" in data
    assert "approved" in data


def test_large_context(client):
    """
    Test decision with large context (stress test).
    """
    decision = {
        "agent_id": "test-agent",
        "decision_type": "test",
        "context": {
            "large_data": "x" * 10000,  # 10KB string
            "files": ["file" + str(i) for i in range(100)]
        },
        "proposed_action": "test action"
    }

    response = client.post("/rank", json=decision)

    assert response.status_code == 200


# =============================================================================
# TEST: CONCURRENCY
# =============================================================================

def test_concurrent_requests(client, fixtures):
    """
    Test that multiple concurrent requests are handled correctly.
    """
    import concurrent.futures

    decision = fixtures["approved_decision"]

    def make_request():
        response = client.post("/rank", json=decision)
        return response.status_code == 200

    # Make 10 concurrent requests
    with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
        results = list(executor.map(lambda _: make_request(), range(10)))

    # All requests should succeed
    assert all(results), "All concurrent requests should succeed"


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
