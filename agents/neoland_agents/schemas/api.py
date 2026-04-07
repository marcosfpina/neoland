"""
Pydantic schemas for the Neoland Agent Pipeline API.
These contracts are mirrored in src/agents/client.rs (Rust side).
Any change here must be reflected there.
"""
from __future__ import annotations

from datetime import datetime
from typing import Literal
from uuid import UUID

from pydantic import BaseModel, Field


class TaskRequest(BaseModel):
    task_id: UUID = Field(description="UUID gerado pelo Rust control plane")
    session_id: UUID = Field(description="Identificador lógico da sessão de trabalho")
    task: str = Field(description="Descrição da tarefa a ser processada pelo pipeline")
    requester_role: str = Field(description="Role RBAC do solicitante (admin, user, readonly)")
    rag_context: str = Field(default="", description="Contexto RAG pré-buscado pelo control plane")
    start_from: Literal["junior", "senior", "architect", "tech_leader"] = Field(
        default="junior",
        description="Estágio de entrada no pipeline (escalation policy do Rust)",
    )


class JuniorOutput(BaseModel):
    hypothesis: str
    confidence: float = Field(ge=0.0, le=1.0)
    risk_level: Literal["low", "medium", "high"]
    unknowns: list[str]
    innovation_vectors: list[str]


class SeniorOutput(BaseModel):
    valid_parts: list[str]
    rejected_parts: list[str]
    risk_assessment: str
    escalate_to_architect: bool
    refined_hypothesis: str


class ArchitectOutput(BaseModel):
    structural_soundness: bool
    composability_score: float = Field(ge=0.0, le=1.0)
    long_term_concerns: list[str]
    recommended_structure: str
    blockers: list[str]


class TechLeaderOutput(BaseModel):
    decision: Literal["approve", "reject", "defer", "escalate"]
    rationale: str
    action_items: list[str]
    adr_title: str
    session_summary: str


class PipelineResult(BaseModel):
    task_id: UUID
    session_id: UUID
    timestamp: datetime
    junior: JuniorOutput
    senior: SeniorOutput
    architect: ArchitectOutput | None = None  # presente apenas se senior.escalate_to_architect
    tech_leader: TechLeaderOutput
    checkpoint_path: str = Field(description="Caminho do arquivo ADR JSON gerado")
