"""Persiste o output do TechLeader como ADR JSON + PostgreSQL."""
from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any
from uuid import UUID

import asyncpg

from ..schemas.api import PipelineResult


class CheckpointManager:
    def __init__(self, checkpoint_dir: str, db_url: str) -> None:
        self.checkpoint_dir = Path(checkpoint_dir)
        self.checkpoint_dir.mkdir(parents=True, exist_ok=True)
        self._db_url = db_url
        self._pool: asyncpg.Pool | None = None

    async def _get_pool(self) -> asyncpg.Pool:
        if self._pool is None:
            self._pool = await asyncpg.create_pool(self._db_url, min_size=1, max_size=5)
        return self._pool

    async def save(self, result: PipelineResult) -> str:
        """Salva checkpoint em arquivo JSON e atualiza PostgreSQL. Retorna o path do arquivo."""
        adr_id = f"ADR-{result.session_id!s:.8}-{result.timestamp.strftime('%Y%m%d%H%M%S')}"
        filename = f"{result.timestamp.strftime('%Y-%m-%d')}-{result.tech_leader.adr_title[:60].replace(' ', '-')}.json"
        filepath = self.checkpoint_dir / filename

        payload = {
            "adr_id": adr_id,
            "title": result.tech_leader.adr_title,
            "status": _decision_to_status(result.tech_leader.decision),
            "context": result.task,
            "decision": result.tech_leader.rationale,
            "action_items": result.tech_leader.action_items,
            "session_summary": result.tech_leader.session_summary,
            "timestamp": result.timestamp.isoformat(),
            "full_pipeline": {
                "junior": result.junior.model_dump(),
                "senior": result.senior.model_dump(),
                "architect": result.architect.model_dump() if result.architect else None,
                "tech_leader": result.tech_leader.model_dump(),
            },
        }

        filepath.write_text(json.dumps(payload, indent=2, ensure_ascii=False))

        await self._persist_to_db(result, str(filepath))
        return str(filepath)

    async def _persist_to_db(self, result: PipelineResult, checkpoint_path: str) -> None:
        pool = await self._get_pool()
        async with pool.acquire() as conn:
            await conn.execute(
                """
                INSERT INTO agent_sessions (
                    session_id, task_id, task, requester_role,
                    junior_output, senior_output, architect_output, tech_leader_output,
                    final_decision, checkpoint_path, completed_at
                ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
                ON CONFLICT (task_id) DO UPDATE SET
                    tech_leader_output = EXCLUDED.tech_leader_output,
                    final_decision = EXCLUDED.final_decision,
                    checkpoint_path = EXCLUDED.checkpoint_path,
                    completed_at = EXCLUDED.completed_at
                """,
                result.session_id,
                result.task_id,
                result.task,
                "unknown",  # preenchido pelo control plane antes de enviar
                json.dumps(result.junior.model_dump()),
                json.dumps(result.senior.model_dump()),
                json.dumps(result.architect.model_dump()) if result.architect else None,
                json.dumps(result.tech_leader.model_dump()),
                result.tech_leader.decision,
                checkpoint_path,
                result.timestamp,
            )
            # Atualiza metadata da sessão
            await conn.execute(
                """
                INSERT INTO agent_session_metadata (session_id, task_count, last_activity, last_decision)
                VALUES ($1, 1, $2, $3)
                ON CONFLICT (session_id) DO UPDATE SET
                    task_count = agent_session_metadata.task_count + 1,
                    last_activity = EXCLUDED.last_activity,
                    last_decision = EXCLUDED.last_decision
                """,
                result.session_id,
                result.timestamp,
                json.dumps(result.tech_leader.model_dump()),
            )

    async def list_by_session(self, session_id: str, limit: int = 20) -> list[dict[str, Any]]:
        """Retorna os checkpoints de uma sessão ordenados do mais recente para o mais antigo."""
        pool = await self._get_pool()
        async with pool.acquire() as conn:
            rows = await conn.fetch(
                """
                SELECT task_id, task, final_decision, checkpoint_path, completed_at
                FROM agent_sessions
                WHERE session_id = $1
                ORDER BY completed_at DESC
                LIMIT $2
                """,
                UUID(session_id),
                limit,
            )
            return [
                {
                    "task_id": str(row["task_id"]),
                    "task": row["task"],
                    "decision": row["final_decision"],
                    "checkpoint_path": row["checkpoint_path"],
                    "completed_at": row["completed_at"].isoformat() if row["completed_at"] else None,
                }
                for row in rows
            ]

    async def close(self) -> None:
        if self._pool:
            await self._pool.close()


def _decision_to_status(decision: str) -> str:
    return {"approve": "accepted", "reject": "rejected", "defer": "deferred", "escalate": "escalated"}.get(
        decision, decision
    )
