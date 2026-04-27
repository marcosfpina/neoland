from fastapi import FastAPI, HTTPException, BackgroundTasks, Response
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import Dict, Any, Optional, List
import time
import logging
import os
import uuid
from pathlib import Path

# In-memory pipeline run store (Phase 4: neoland integration)
_pipeline_runs: List[Dict[str, Any]] = []

# STF Integration
import sys
sys.path.insert(0, str(Path(__file__).parent.parent))
from stf.parser import STFParser

# Configuration
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("ai-ranking")

app = FastAPI(title="AI Agent Ranking System - STF Enforced")

# CORS Configuration
# ===================
# Allow requests from frontend (development and production)
ALLOWED_ORIGINS = os.getenv(
    "ALLOWED_ORIGINS",
    "http://localhost:3006,http://localhost:3001,https://ai-assistant.yourdomain.com"
).split(",")

app.add_middleware(
    CORSMiddleware,
    allow_origins=ALLOWED_ORIGINS,
    allow_credentials=True,
    allow_methods=["GET", "POST", "PUT", "DELETE", "OPTIONS"],
    allow_headers=["*"],
    max_age=3600,  # Cache preflight requests for 1 hour
)

logger.info(f"CORS enabled for origins: {ALLOWED_ORIGINS}")

# Load STF Protocol
STF_PATH = os.getenv('STF_PATH', '/home/kernelcore/arch/adr-ledger/.stf/neutron.stf')
stf_parser = None

try:
    stf_parser = STFParser(STF_PATH)
    logger.info(f"✓ STF Protocol loaded: {stf_parser.protocol.protocol} v{stf_parser.protocol.version}")
    logger.info(f"  Mode: {stf_parser.protocol.mode.value}")
    logger.info(f"  Directives: {len(stf_parser.protocol.directives)} | Constraints: {len(stf_parser.protocol.constraints)}")
except Exception as e:
    logger.error(f"✗ Failed to load STF: {e}")
    stf_parser = None

# Load ML Model
from ml import ModelRegistry

ml_registry = None
ml_model = None

try:
    ml_registry = ModelRegistry()
    ml_model = ml_registry.load_model("decision-scorer")
    logger.info(f"✓ ML Model loaded: decision-scorer v{ml_model.version}")
except Exception as e:
    logger.error(f"✗ Failed to load ML model: {e}")
    ml_model = None

class DecisionRequest(BaseModel):
    agent_id: str
    decision_type: str
    context: Dict[str, Any]
    proposed_action: str
    project_id: Optional[str] = None
    model_version: Optional[str] = "v1"

class RankingResponse(BaseModel):
    agent_id: str
    score: float
    approved: bool
    requires_human_review: bool
    stf_compliant: bool
    stf_violations: List[str]
    timestamp: float

class AgentMessage(BaseModel):
    sender_id: str
    receiver_id: str
    intent: str
    content: Dict[str, Any]
    priority: str = "normal"

class MessageDeliveryResponse(BaseModel):
    delivered: bool
    message_id: str
    governance_score: float
    status: str
    block_reason: Optional[str] = None

# Thresholds (STF: Developer as Fallback)
AUTO_APPROVAL_THRESHOLD = 0.90
CRITICAL_DECISION_TYPES = ["deploy", "delete_db", "modify_acl"]

def log_decision(decision: DecisionRequest, score: float, approved: bool, stf_compliant: bool, violations: List[str], latency_ms: float):
    # Import observability tracking
    try:
        from observability import track_decision

        # Track in observability system (metrics + structured logs)
        track_decision(
            agent_id=decision.agent_id,
            score=score,
            approved=approved,
            stf_compliant=stf_compliant,
            latency_ms=latency_ms,
            decision_type=decision.decision_type,
            stf_violations=violations,
            context=decision.context,
            metadata={"model_version": decision.model_version}
        )
    except ImportError:
        logger.warning("Observability module not found, skipping detailed tracking")

    # Also log to console for immediate visibility
    logger.info(
        f"Decision: {decision.agent_id} | "
        f"Score: {score:.3f} | "
        f"Approved: {approved} | "
        f"STF Compliant: {stf_compliant} | "
        f"Violations: {len(violations)} | "
        f"Latency: {latency_ms:.1f}ms"
    )
    if violations:
        for violation in violations:
            logger.warning(f"  ⚠ STF Violation: {violation}")

def validate_stf_compliance(decision: DecisionRequest) -> tuple[bool, List[str]]:
    """
    Validate decision against STF directives.

    Returns:
        (is_compliant, violations)
    """
    if not stf_parser:
        logger.warning("STF parser not loaded, skipping validation")
        return True, []

    decision_dict = {
        'action': decision.proposed_action,
        'type': decision.decision_type,
        'context': decision.context
    }

    return stf_parser.validate_decision(decision_dict)

def calculate_score(decision: DecisionRequest) -> tuple[float, bool, List[str]]:
    """
    Calculate decision score with STF validation and ML model.

    Returns:
        (score, stf_compliant, violations)
    """
    # 1. STF Validation (highest priority)
    stf_compliant, violations = validate_stf_compliance(decision)

    # 2. ML model inference
    if ml_model:
        try:
            # Prepare decision dict for model
            decision_dict = {
                'decision_type': decision.decision_type,
                'agent_id': decision.agent_id,
                'stf_compliant': stf_compliant,
                'stf_compliant': stf_compliant,
                'context': decision.context,
                'proposed_action': decision.proposed_action,
                'project_id': decision.project_id
            }

            # Get ML prediction
            score = ml_model.predict(decision_dict)
            logger.debug(f"ML model score: {score:.3f}")
        except Exception as e:
            logger.error(f"ML model prediction failed: {e}, using heuristic fallback")
            score = 0.95  # Fallback to optimistic default
    else:
        # Fallback to heuristic scoring
        score = 0.95

        # Heuristic penalties (only if ML model unavailable)
        if decision.decision_type in CRITICAL_DECISION_TYPES:
            score -= 0.1

        if "entropy" in decision.context and decision.context["entropy"] > 0.8:
            score -= 0.2

    # 3. STF violations heavily penalize score (override ML if needed)
    if not stf_compliant:
        violation_penalty = 0.5 * len(violations)
        score = min(score, score - violation_penalty)
        logger.warning(f"STF violations detected, score reduced to {score:.3f}")

    return max(0.0, min(1.0, score)), stf_compliant, violations

@app.post("/communicate", response_model=MessageDeliveryResponse)
async def agent_communication(message: AgentMessage, background_tasks: BackgroundTasks):
    """
    A2A (Agent-to-Agent) Communication Bus with Governance.
    
    Intercepts messages between agents, validates intent against STF,
    and allows or blocks the communication.
    """
    import uuid
    msg_id = str(uuid.uuid4())
    
    # Transform Message to Decision Request for Governance
    decision_request = DecisionRequest(
        agent_id=message.sender_id,
        decision_type="a2a_communication",
        proposed_action=f"send_to:{message.receiver_id}",
        context={
            "intent": message.intent,
            "receiver": message.receiver_id,
            "content_preview": str(message.content)[:100],
            "priority": message.priority
        },
        model_version="v2-a2a"
    )
    
    # Calculate Governance Score
    score, stf_compliant, violations = calculate_score(decision_request)
    
    # Logic: High Stakes messages require higher scores
    threshold = AUTO_APPROVAL_THRESHOLD
    if message.priority == "high":
        threshold = 0.95
        
    approved = stf_compliant and (score >= threshold)
    
    # Log the attempt
    background_tasks.add_task(
        log_decision, 
        decision_request, 
        score, 
        approved, 
        stf_compliant, 
        violations, 
        0.0
    )
    
    if approved:
        logger.info(f"📨 A2A Message Delivered: {message.sender_id} -> {message.receiver_id} [{message.intent}]")
        return MessageDeliveryResponse(
            delivered=True,
            message_id=msg_id,
            governance_score=score,
            status="delivered"
        )
    else:
        reason = "STF Violation" if not stf_compliant else f"Low Trust Score ({score:.2f} < {threshold})"
        logger.warning(f"🛡️ A2A Message Blocked: {message.sender_id} -> {message.receiver_id}. Reason: {reason}")
        return MessageDeliveryResponse(
            delivered=False,
            message_id=msg_id,
            governance_score=score,
            status="blocked",
            block_reason=reason
        )

@app.post("/rank", response_model=RankingResponse)
async def rank_decision(request: DecisionRequest, background_tasks: BackgroundTasks):
    """
    Rank an agent decision with STF compliance validation.

    STF violations result in:
    - Score reduction (50% per violation)
    - Automatic rejection if non-compliant
    - Human review requirement
    """
    start_time = time.time()

    # 1. Calculate Score + STF Validation
    score, stf_compliant, violations = calculate_score(request)

    # 2. Apply Policy (Human-in-the-loop)
    approved = False
    requires_human = False

    # 2a. STF violations always require human review
    if not stf_compliant:
        approved = False
        requires_human = True
        logger.warning(f"Decision rejected due to STF violations: {violations}")

    # 2b. Score-based approval (only if STF compliant)
    elif score > AUTO_APPROVAL_THRESHOLD:
        approved = True
        requires_human = False
    else:
        approved = False
        requires_human = True

    # 3. Observability
    latency_ms = (time.time() - start_time) * 1000
    background_tasks.add_task(log_decision, request, score, approved, stf_compliant, violations, latency_ms)

    return RankingResponse(
        agent_id=request.agent_id,
        score=score,
        approved=approved,
        requires_human_review=requires_human,
        stf_compliant=stf_compliant,
        stf_violations=violations,
        timestamp=time.time()
    )

@app.get("/health")
def health():
    """Health check with STF and ML model status"""
    stf_status = "loaded" if stf_parser else "not_loaded"
    ml_status = "loaded" if ml_model else "not_loaded"

    return {
        "status": "ok",
        "mode": "strict_compliance",
        "stf_protocol": stf_status,
        "stf_version": stf_parser.protocol.version if stf_parser else None,
        "ml_model": ml_status,
        "ml_version": ml_model.version if ml_model else None
    }

@app.get("/stf/context")
def get_stf_context(format: str = "markdown"):
    """
    Get STF context for agent injection.

    Query params:
        format: 'markdown', 'json', or 'xml'
    """
    if not stf_parser:
        raise HTTPException(status_code=503, detail="STF parser not loaded")

    try:
        context = stf_parser.get_injection_context(format=format)
        return {"format": format, "context": context}
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))

@app.get("/metrics")
def get_prometheus_metrics():
    """
    Prometheus metrics endpoint.

    Returns metrics in Prometheus exposition format.
    """
    from observability import get_metrics, get_metrics_content_type

    metrics_data = get_metrics()
    return Response(
        content=metrics_data,
        media_type=get_metrics_content_type()
    )


# ── Phase 4: Neoland pipeline tracking ───────────────────────────────────────

class AgentRunMetrics(BaseModel):
    confidence: Optional[float] = None
    latency_ms: Optional[int] = None
    escalated: Optional[bool] = None
    decision: Optional[str] = None

class PipelineMetricsRequest(BaseModel):
    session_id: str
    task: str
    agents: Dict[str, AgentRunMetrics]
    total_latency_ms: int
    decision: str
    adr_title: Optional[str] = None

class PipelineRunSummary(BaseModel):
    run_id: str
    session_id: str
    task_preview: str
    decision: str
    adr_title: Optional[str]
    junior_confidence: Optional[float]
    total_latency_ms: int
    score: float
    timestamp: float

@app.post("/pipeline/metrics", response_model=PipelineRunSummary)
async def store_pipeline_metrics(request: PipelineMetricsRequest):
    """Store a completed neoland pipeline run for the matrix dashboard."""
    run_id = str(uuid.uuid4())
    junior = request.agents.get("junior") or AgentRunMetrics()
    junior_confidence = junior.confidence

    score = (junior_confidence or 0.5)
    if request.decision != "accepted":
        score *= 0.5

    entry: Dict[str, Any] = {
        "run_id": run_id,
        "session_id": request.session_id,
        "task": request.task,
        "task_preview": request.task[:120],
        "agents": {k: v.model_dump() for k, v in request.agents.items()},
        "total_latency_ms": request.total_latency_ms,
        "decision": request.decision,
        "adr_title": request.adr_title,
        "junior_confidence": junior_confidence,
        "score": round(score, 4),
        "timestamp": time.time(),
    }
    _pipeline_runs.append(entry)
    logger.info(f"Pipeline run stored: {run_id} | {request.decision} | {request.total_latency_ms}ms")

    return PipelineRunSummary(
        run_id=run_id,
        session_id=request.session_id,
        task_preview=entry["task_preview"],
        decision=request.decision,
        adr_title=request.adr_title,
        junior_confidence=junior_confidence,
        total_latency_ms=request.total_latency_ms,
        score=score,
        timestamp=entry["timestamp"],
    )

@app.get("/pipeline/history")
def get_pipeline_history(limit: int = 50):
    """Return recent neoland pipeline runs for the matrix dashboard."""
    runs = _pipeline_runs[-limit:]
    return {
        "runs": list(reversed(runs)),
        "total": len(_pipeline_runs),
    }
