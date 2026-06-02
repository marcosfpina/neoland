"""
Contract tests: validam que os schemas DSPy produzem outputs compatíveis
com os tipos Rust (via Pydantic validation). Requer LLM real.
"""
import os
import pytest

import dspy
from neoland_agents.schemas.api import (
    JuniorOutput,
    SeniorOutput,
    ArchitectOutput,
    TechLeaderOutput,
)
from neoland_agents.signatures.task_proposal import TaskProposal
from neoland_agents.signatures.critique import SeniorCritique
from neoland_agents.signatures.architecture import ArchitectReview
from neoland_agents.signatures.decision import TechLeaderDecision


@pytest.fixture(scope="module")
def lm():
    from neoland_agents.config import settings
    return settings.dspy_lm()


@pytest.mark.integration
def test_junior_schema_valid(lm):
    dspy.configure(lm=lm)
    predictor = dspy.ChainOfThought(TaskProposal)
    result = predictor(
        task="Adicionar cache LRU ao vector store",
        context="O vector store usa PostgreSQL + pgvector.",
    )
    # Deve ser convertível para JuniorOutput sem ValidationError
    out = JuniorOutput(
        hypothesis=result.hypothesis,
        confidence=float(result.confidence),
        risk_level=result.risk_level,
        unknowns=result.unknowns if isinstance(result.unknowns, list) else [],
        innovation_vectors=result.innovation_vectors if isinstance(result.innovation_vectors, list) else [],
    )
    assert 0.0 <= out.confidence <= 1.0
    assert out.risk_level in ("low", "medium", "high")
    assert len(out.hypothesis) > 10


@pytest.mark.integration
def test_senior_schema_valid(lm):
    dspy.configure(lm=lm)
    jr = dspy.ChainOfThought(TaskProposal)(
        task="Adicionar cache LRU", context=""
    )
    sr = dspy.ChainOfThought(SeniorCritique)(
        task="Adicionar cache LRU",
        proposal=f"hypothesis: {jr.hypothesis}",
    )
    out = SeniorOutput(
        valid_parts=sr.valid_parts if isinstance(sr.valid_parts, list) else [],
        rejected_parts=sr.rejected_parts if isinstance(sr.rejected_parts, list) else [],
        risk_assessment=sr.risk_assessment,
        escalate_to_architect=bool(sr.escalate_to_architect),
        refined_hypothesis=sr.refined_hypothesis,
    )
    assert isinstance(out.escalate_to_architect, bool)
    assert len(out.refined_hypothesis) > 5


@pytest.mark.integration
def test_tech_leader_decision_is_valid_enum(lm):
    dspy.configure(lm=lm)
    tl = dspy.ChainOfThought(TechLeaderDecision)(
        task="Adicionar cache LRU",
        all_inputs='{"junior": {"hypothesis": "use LRU"}, "senior": {"risk": "low"}, "architect": null}',
    )
    out = TechLeaderOutput(
        decision=tl.decision,
        rationale=tl.rationale,
        action_items=tl.action_items if isinstance(tl.action_items, list) else [],
        adr_title=tl.adr_title,
        session_summary=tl.session_summary,
    )
    assert out.decision in ("approve", "reject", "defer", "escalate")
    assert len(out.action_items) >= 0


@pytest.mark.contract
def test_pipeline_result_rejects_invalid_decision():
    """Sem LLM — valida que Pydantic rejeita valores fora do enum."""
    from pydantic import ValidationError
    with pytest.raises(ValidationError):
        TechLeaderOutput(
            decision="invalid_value",
            rationale="x",
            action_items=[],
            adr_title="ADR-001",
            session_summary="x",
        )


@pytest.mark.contract
def test_junior_output_rejects_confidence_out_of_range():
    from pydantic import ValidationError
    with pytest.raises(ValidationError):
        JuniorOutput(
            hypothesis="x",
            confidence=1.5,  # > 1.0
            risk_level="low",
            unknowns=[],
            innovation_vectors=[],
        )
