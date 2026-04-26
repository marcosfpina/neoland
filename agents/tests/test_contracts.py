"""
Contract tests — validam schemas Pydantic e IPC sem LLM nem DB.
Rodam em CI com: pytest tests/ -m contract -v
"""
from __future__ import annotations

import struct
import uuid
from pathlib import Path

import pytest
from pydantic import ValidationError

from neoland_agents.ipc.flags import AgentFlags, RiskLevel, _FMT, _SIZE
from neoland_agents.schemas.api import (
    ArchitectOutput,
    JuniorOutput,
    PipelineResult,
    SeniorOutput,
    TaskRequest,
    TechLeaderOutput,
)

# ── helpers ───────────────────────────────────────────────────────────────────


def _shm(tmp_path: Path) -> Path:
    """Create a zero-initialised 64-byte shm file in tmp_path."""
    p = tmp_path / "agent-flags.shm"
    p.write_bytes(b"\x00" * _SIZE)
    return p


# ── struct layout ─────────────────────────────────────────────────────────────


@pytest.mark.contract
def test_flags_struct_size():
    assert _SIZE == 64, f"SharedFlags layout mismatch: {_SIZE} != 64"


@pytest.mark.contract
def test_flags_fmt_unpacks_to_six_fields():
    dummy = b"\x00" * _SIZE
    fields = struct.unpack(_FMT, dummy)
    assert len(fields) == 6  # pa, eta, ar, conf, risk, sid_bytes


# ── AgentFlags IPC roundtrip ─────────────────────────────────────────────────


@pytest.mark.contract
def test_flags_pipeline_active(tmp_path: Path):
    with AgentFlags(_shm(tmp_path)) as f:
        f.set_pipeline_active(True)
        assert f.read()["pipeline_active"] is True
        f.set_pipeline_active(False)
        assert f.read()["pipeline_active"] is False


@pytest.mark.contract
def test_flags_escalate_to_architect(tmp_path: Path):
    with AgentFlags(_shm(tmp_path)) as f:
        f.set_escalate_to_architect(True)
        assert f.read()["escalate_to_architect"] is True


@pytest.mark.contract
def test_flags_junior_confidence(tmp_path: Path):
    with AgentFlags(_shm(tmp_path)) as f:
        f.set_junior_confidence(0.85)
        snap = f.read()
        assert abs(snap["junior_confidence"] - 0.85) < 1e-5


@pytest.mark.contract
def test_flags_confidence_zero_and_one(tmp_path: Path):
    with AgentFlags(_shm(tmp_path)) as f:
        for val in (0.0, 1.0):
            f.set_junior_confidence(val)
            assert abs(f.read()["junior_confidence"] - val) < 1e-6


@pytest.mark.contract
def test_flags_risk_level_all_values(tmp_path: Path):
    expected = {
        RiskLevel.LOW: "low",
        RiskLevel.MEDIUM: "medium",
        RiskLevel.HIGH: "high",
        RiskLevel.CRITICAL: "critical",
    }
    with AgentFlags(_shm(tmp_path)) as f:
        for level, name in expected.items():
            f.set_risk_level(level)
            assert f.read()["risk_level"] == name


@pytest.mark.contract
def test_flags_session_id_roundtrip(tmp_path: Path):
    sid = str(uuid.uuid4())
    with AgentFlags(_shm(tmp_path)) as f:
        f.set_session_id(sid)
        assert f.read()["session_id"] == sid


@pytest.mark.contract
def test_flags_session_id_truncated_to_36(tmp_path: Path):
    with AgentFlags(_shm(tmp_path)) as f:
        f.set_session_id("x" * 100)
        snap = f.read()
        assert len(snap["session_id"]) <= 36


@pytest.mark.contract
def test_flags_abort_fast_path(tmp_path: Path):
    with AgentFlags(_shm(tmp_path)) as f:
        assert f.abort_requested() is False


@pytest.mark.contract
def test_flags_not_open_raises(tmp_path: Path):
    f = AgentFlags(_shm(tmp_path))
    with pytest.raises(RuntimeError, match="not open"):
        f.read()


# ── JuniorOutput schema ───────────────────────────────────────────────────────


@pytest.mark.contract
def test_junior_valid():
    out = JuniorOutput(
        hypothesis="cache LRU reduz latência",
        confidence=0.72,
        risk_level="low",
        unknowns=["throughput sob carga"],
        innovation_vectors=["LRU com TTL"],
    )
    assert out.confidence == 0.72
    assert out.risk_level == "low"


@pytest.mark.contract
def test_junior_confidence_above_one_rejected():
    with pytest.raises(ValidationError):
        JuniorOutput(
            hypothesis="h", confidence=1.1, risk_level="low", unknowns=[], innovation_vectors=[]
        )


@pytest.mark.contract
def test_junior_confidence_below_zero_rejected():
    with pytest.raises(ValidationError):
        JuniorOutput(
            hypothesis="h", confidence=-0.1, risk_level="low", unknowns=[], innovation_vectors=[]
        )


@pytest.mark.contract
def test_junior_invalid_risk_level():
    with pytest.raises(ValidationError):
        JuniorOutput(
            hypothesis="h",
            confidence=0.5,
            risk_level="critical",  # not in Literal["low","medium","high"]
            unknowns=[],
            innovation_vectors=[],
        )


# ── SeniorOutput schema ───────────────────────────────────────────────────────


@pytest.mark.contract
def test_senior_valid():
    out = SeniorOutput(
        valid_parts=["cache"],
        rejected_parts=[],
        risk_assessment="medium",
        escalate_to_architect=False,
        refined_hypothesis="cache LRU com TTL de 30s",
    )
    assert out.escalate_to_architect is False


@pytest.mark.contract
def test_senior_escalate_to_architect_stores_bool():
    # Pydantic v2 coerces "yes"/"no" → True/False in lax mode; verify the field
    # is stored as a proper bool after construction.
    out = SeniorOutput(
        valid_parts=[],
        rejected_parts=[],
        risk_assessment="low",
        escalate_to_architect=False,
        refined_hypothesis="h",
    )
    assert isinstance(out.escalate_to_architect, bool)
    assert out.escalate_to_architect is False


# ── TechLeaderOutput schema ───────────────────────────────────────────────────


@pytest.mark.contract
def test_tech_leader_valid_decisions():
    for decision in ("approve", "reject", "defer", "escalate"):
        out = TechLeaderOutput(
            decision=decision,
            rationale="r",
            action_items=[],
            adr_title="ADR-001",
            session_summary="s",
        )
        assert out.decision == decision


@pytest.mark.contract
def test_tech_leader_invalid_decision():
    with pytest.raises(ValidationError):
        TechLeaderOutput(
            decision="accepted",  # not in Literal
            rationale="r",
            action_items=[],
            adr_title="ADR-001",
            session_summary="s",
        )


# ── ArchitectOutput schema ────────────────────────────────────────────────────


@pytest.mark.contract
def test_architect_composability_bounds():
    with pytest.raises(ValidationError):
        ArchitectOutput(
            structural_soundness=True,
            composability_score=1.5,  # > 1.0
            long_term_concerns=[],
            recommended_structure="monolith",
            blockers=[],
        )


# ── TaskRequest schema ────────────────────────────────────────────────────────


@pytest.mark.contract
def test_task_request_valid_start_from():
    for stage in ("junior", "senior", "architect", "tech_leader"):
        req = TaskRequest(
            task_id=uuid.uuid4(),
            session_id=uuid.uuid4(),
            task="implementar cache",
            requester_role="user",
            start_from=stage,
        )
        assert req.start_from == stage


@pytest.mark.contract
def test_task_request_invalid_start_from():
    with pytest.raises(ValidationError):
        TaskRequest(
            task_id=uuid.uuid4(),
            session_id=uuid.uuid4(),
            task="t",
            requester_role="user",
            start_from="intern",  # not valid
        )


@pytest.mark.contract
def test_task_request_rag_context_defaults_empty():
    req = TaskRequest(
        task_id=uuid.uuid4(),
        session_id=uuid.uuid4(),
        task="t",
        requester_role="user",
    )
    assert req.rag_context == ""


# ── PipelineResult schema ─────────────────────────────────────────────────────


@pytest.mark.contract
def test_pipeline_result_architect_optional():
    from datetime import datetime, timezone

    r = PipelineResult(
        task_id=uuid.uuid4(),
        session_id=uuid.uuid4(),
        timestamp=datetime.now(timezone.utc),
        junior=JuniorOutput(
            hypothesis="h", confidence=0.8, risk_level="low", unknowns=[], innovation_vectors=[]
        ),
        senior=SeniorOutput(
            valid_parts=[],
            rejected_parts=[],
            risk_assessment="low",
            escalate_to_architect=False,
            refined_hypothesis="h",
        ),
        architect=None,
        tech_leader=TechLeaderOutput(
            decision="approve",
            rationale="ok",
            action_items=[],
            adr_title="ADR-001",
            session_summary="done",
        ),
        checkpoint_path="/var/lib/neoland/checkpoints/adr/test.json",
    )
    assert r.architect is None
    assert r.tech_leader.decision == "approve"
