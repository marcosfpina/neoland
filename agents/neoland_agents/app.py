"""FastAPI entry point do pipeline DSPy. Escuta em :8001 por padrão."""
from __future__ import annotations

from contextlib import asynccontextmanager
from typing import Any

from fastapi import FastAPI, HTTPException

from .config import settings
from .pipeline.orchestrator import AgentOrchestrator
from .schemas.api import PipelineResult, TaskRequest

_orchestrator: AgentOrchestrator | None = None


@asynccontextmanager
async def lifespan(app: FastAPI):  # type: ignore[type-arg]
    global _orchestrator
    lm = settings.dspy_lm()
    _orchestrator = AgentOrchestrator(
        lm=lm,
        checkpoint_dir=settings.checkpoint_dir,
        db_url=settings.db_url,
    )
    yield
    if _orchestrator:
        await _orchestrator.close()


app = FastAPI(title="Neoland Agent Pipeline", version="1.0.0", lifespan=lifespan)


def get_orchestrator() -> AgentOrchestrator:
    if _orchestrator is None:
        raise RuntimeError("Orchestrator not initialized")
    return _orchestrator


@app.post("/v1/pipeline/run", response_model=PipelineResult)
async def run_pipeline(request: TaskRequest) -> PipelineResult:
    try:
        return await get_orchestrator().run(request)
    except Exception as exc:
        raise HTTPException(status_code=500, detail=str(exc)) from exc


@app.get("/v1/pipeline/session/{session_id}")
async def get_session(session_id: str) -> dict[str, Any]:
    """Checkpoints de uma sessão, do mais recente ao mais antigo."""
    try:
        runs = await get_orchestrator().checkpoint.list_by_session(session_id)
        return {"session_id": session_id, "runs": runs, "total": len(runs)}
    except ValueError:
        raise HTTPException(status_code=400, detail="session_id must be a valid UUID") from None
    except Exception as exc:
        raise HTTPException(status_code=500, detail=str(exc)) from exc


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok", "version": "1.0.0"}
