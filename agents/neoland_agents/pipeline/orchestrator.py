"""Orquestra os 4 agentes em sequência. Junior → Senior → Architect? → TechLeader."""
from __future__ import annotations

import json
from datetime import datetime, timezone

import dspy

from ..modules.architect import ArchitectAgent
from ..modules.junior import JuniorAgent
from ..modules.senior import SeniorAgent
from ..modules.tech_leader import TechLeaderAgent
from ..pipeline.checkpoint import CheckpointManager
from ..schemas.api import (
    ArchitectOutput,
    JuniorOutput,
    PipelineResult,
    SeniorOutput,
    TaskRequest,
    TechLeaderOutput,
)


class AgentOrchestrator:
    def __init__(self, lm: dspy.LM, checkpoint_dir: str, db_url: str) -> None:
        dspy.configure(lm=lm)
        self.junior = JuniorAgent()
        self.senior = SeniorAgent()
        self.architect = ArchitectAgent()
        self.tech_leader = TechLeaderAgent()
        self.checkpoint = CheckpointManager(checkpoint_dir=checkpoint_dir, db_url=db_url)

    async def run(self, request: TaskRequest) -> PipelineResult:
        # 1. Junior — sempre executa
        jr = self.junior(task=request.task, context=request.rag_context)
        junior_out = JuniorOutput(
            hypothesis=jr.hypothesis,
            confidence=float(jr.confidence),
            risk_level=jr.risk_level,
            unknowns=jr.unknowns if isinstance(jr.unknowns, list) else [],
            innovation_vectors=jr.innovation_vectors if isinstance(jr.innovation_vectors, list) else [],
        )

        # 2. Senior — sempre executa
        sr = self.senior(task=request.task, proposal=junior_out.model_dump_json())
        senior_out = SeniorOutput(
            valid_parts=sr.valid_parts if isinstance(sr.valid_parts, list) else [],
            rejected_parts=sr.rejected_parts if isinstance(sr.rejected_parts, list) else [],
            risk_assessment=sr.risk_assessment,
            escalate_to_architect=bool(sr.escalate_to_architect),
            refined_hypothesis=sr.refined_hypothesis,
        )

        # 3. Architect — somente se Senior escalou
        architect_out: ArchitectOutput | None = None
        if senior_out.escalate_to_architect:
            ar = self.architect(
                task=request.task,
                refined_hypothesis=senior_out.refined_hypothesis,
                risk_assessment=senior_out.risk_assessment,
            )
            architect_out = ArchitectOutput(
                structural_soundness=bool(ar.structural_soundness),
                composability_score=float(ar.composability_score),
                long_term_concerns=ar.long_term_concerns if isinstance(ar.long_term_concerns, list) else [],
                recommended_structure=ar.recommended_structure,
                blockers=ar.blockers if isinstance(ar.blockers, list) else [],
            )

        # 4. Tech-Leader — sempre executa
        all_inputs = json.dumps({
            "junior": junior_out.model_dump(),
            "senior": senior_out.model_dump(),
            "architect": architect_out.model_dump() if architect_out else None,
        }, ensure_ascii=False)

        tl = self.tech_leader(task=request.task, all_inputs=all_inputs)
        tech_leader_out = TechLeaderOutput(
            decision=tl.decision,
            rationale=tl.rationale,
            action_items=tl.action_items if isinstance(tl.action_items, list) else [],
            adr_title=tl.adr_title,
            session_summary=tl.session_summary,
        )

        result = PipelineResult(
            task_id=request.task_id,
            session_id=request.session_id,
            timestamp=datetime.now(tz=timezone.utc),
            junior=junior_out,
            senior=senior_out,
            architect=architect_out,
            tech_leader=tech_leader_out,
            checkpoint_path="",  # preenchido após save
        )

        # 5. Checkpoint automático
        checkpoint_path = await self.checkpoint.save(result)
        result.checkpoint_path = checkpoint_path

        return result

    async def close(self) -> None:
        await self.checkpoint.close()
