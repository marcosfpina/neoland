#!/usr/bin/env bash
# Full-stack smoke: Neoland → SecureLLM Bridge → agents pipeline
# Records evidence to /tmp/neoland-smoke-<timestamp>.log
# Usage: nix develop --command bash scripts/smoke-full-stack.sh

set -euo pipefail

RUN_TASK="${NEOLAND_SMOKE_RUN_TASK:-0}"
case "${1:-}" in
  --task|task) RUN_TASK=1 ;;
  -h|--help|help)
    cat <<'USAGE'
Usage: scripts/smoke-full-stack.sh [--task]

Default smoke validates service reachability, SecureLLM Bridge chat, and doctor.
Use --task to also run the long Neoland → DSPy pipeline task.
USAGE
    exit 0
    ;;
  "") ;;
  *)
    echo "Unknown argument: $1" >&2
    exit 2
    ;;
esac

# Load SOPS secrets to match the running server's API key
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOPS_ENV="${NEOLAND_SOPS_ENV_FILE:-$SCRIPT_DIR/../secrets/neoland.sops.env}"
if [[ -f "$SOPS_ENV" ]] && command -v sops >/dev/null 2>&1; then
    set -a; source <(sops decrypt --output-type dotenv "$SOPS_ENV" 2>/dev/null); set +a
fi

TIMESTAMP=$(date +%Y%m%d-%H%M%S)
LOG="/tmp/neoland-smoke-${TIMESTAMP}.log"
PASS=0
FAIL=0
WARN=0

log() { echo "$*" | tee -a "$LOG"; }
ok()   { log "  [OK]  $*"; PASS=$((PASS+1)); }
fail() { log "  [FAIL] $*"; FAIL=$((FAIL+1)); }
warn() { log "  [WARN] $*"; WARN=$((WARN+1)); }
section() { log ""; log "── $* ──────────────────────────────────────────────────"; }

NEOLAND_URL="${NEOLAND_CONTROL_PLANE_URL:-http://127.0.0.1:3001}"
GATEWAY_URL="${NEOLAND_GATEWAY_URL:-http://127.0.0.1:8080}"
MLOPS_URL="${ML_OPS_API_URL:-http://127.0.0.1:8083}"
LLAMACPP_URL="${LLAMACPP_URL:-http://127.0.0.1:8081}"
DSPY_URL="${NEOLAND_DSPY_URL:-http://127.0.0.1:8001}"
API_KEY="${NEOLAND_ADMIN_API_KEY:-neoland_admin_dev_key_change_in_production}"
REQUIRE_MLOPS="${REQUIRE_MLOPS:-0}"
TASK_TIMEOUT_SECS="${NEOLAND_SMOKE_TASK_TIMEOUT_SECS:-180}"

log "Neoland SecureLLM Smoke — ${TIMESTAMP}"
log "Log: ${LOG}"

# ── 1. Control Plane ──────────────────────────────────────────────────────
section "1. Control Plane (${NEOLAND_URL})"

if curl -sf "${NEOLAND_URL}/health" -o /dev/null; then
  STATUS=$(curl -sf "${NEOLAND_URL}/health" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('status','?'))" 2>/dev/null || echo "?")
  ok "/health → ${STATUS}"
else
  fail "/health unreachable — run: nix develop --command just server"
fi

if curl -sf "${NEOLAND_URL}/ready" -o /dev/null; then
  READY=$(curl -sf "${NEOLAND_URL}/ready" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('ready','?'))" 2>/dev/null || echo "?")
  ok "/ready → ${READY}"
else
  fail "/ready unreachable"
fi

if curl -sf "${NEOLAND_URL}/live" -o /dev/null; then
  ok "/live → alive"
else
  fail "/live unreachable"
fi

# ── 2. DSPy Pipeline ──────────────────────────────────────────────────────
section "2. DSPy Pipeline (${DSPY_URL})"

if curl -sf "${DSPY_URL}/health" -o /dev/null; then
  ok "/health reachable"
else
  fail "/health unreachable — run: nix develop --command just agents-start"
fi

# ── 3. SecureLLM Bridge Gateway ───────────────────────────────────────────
section "3. SecureLLM Bridge (${GATEWAY_URL})"

if curl -sf "${GATEWAY_URL}/api/health" -o /dev/null 2>/dev/null || \
   curl -sf "${GATEWAY_URL}/health"     -o /dev/null 2>/dev/null; then
  ok "gateway health reachable"
else
  fail "gateway unreachable at ${GATEWAY_URL} — start securellm-bridge"
fi

# ── 4. ml-ops-api ─────────────────────────────────────────────────────────
section "4. ml-ops-api (${MLOPS_URL}, optional)"

if curl -sf "${MLOPS_URL}/health" -o /dev/null 2>/dev/null; then
  ok "/health reachable"
elif [[ "$REQUIRE_MLOPS" == "1" ]]; then
  fail "/health unreachable at ${MLOPS_URL} — start ml-ops-api"
else
  warn "/health unreachable at ${MLOPS_URL} — optional for SecureLLM Bridge workflow"
fi

# ── 5. llama.cpp ──────────────────────────────────────────────────────────
section "5. llama.cpp (${LLAMACPP_URL})"

if curl -sf "${LLAMACPP_URL}/health" -o /dev/null 2>/dev/null; then
  ok "/health reachable"
else
  fail "/health unreachable at ${LLAMACPP_URL} — start llama-server"
fi

# ── 6. SecureLLM chat completion ──────────────────────────────────────────
section "6. SecureLLM chat completion"

BRIDGE_BODY_FILE="${LOG}.bridge-chat.json"
: >"$BRIDGE_BODY_FILE"
BRIDGE_STATUS=$(curl -s -o "$BRIDGE_BODY_FILE" -w "%{http_code}" -X POST "${GATEWAY_URL}/v1/chat/completions" \
  -H "Content-Type: application/json" \
  -d '{"model":"llamacpp/local-model","messages":[{"role":"user","content":"Say ok"}],"max_tokens":8}' \
  --max-time 45 2>/dev/null || true)
BRIDGE_STATUS="${BRIDGE_STATUS:-000}"
BRIDGE_RESP="$(tr '\n' ' ' <"$BRIDGE_BODY_FILE" 2>/dev/null || true)"

if [[ "$BRIDGE_STATUS" =~ ^2[0-9][0-9]$ ]]; then
  ok "POST /v1/chat/completions returned HTTP ${BRIDGE_STATUS}"
  log "    response: ${BRIDGE_RESP:0:200}..."
else
  fail "POST /v1/chat/completions returned HTTP ${BRIDGE_STATUS}"
  log "    response: ${BRIDGE_RESP:0:400}..."
fi

# ── 7. Optional end-to-end task ───────────────────────────────────────────
section "7. End-to-end task (optional Neoland → pipeline)"

TASK_BODY_FILE="${LOG}.task-response.json"
: >"$TASK_BODY_FILE"
if [[ "$RUN_TASK" == "1" ]]; then
  TASK_STATUS=$(curl -s -o "$TASK_BODY_FILE" -w "%{http_code}" -X POST "${NEOLAND_URL}/v1/agents/task" \
    -H "X-API-Key: ${API_KEY}" \
    -H "Content-Type: application/json" \
    -d '{"task":"smoke test: confirm pipeline is reachable"}' \
    --max-time "$TASK_TIMEOUT_SECS" 2>/dev/null || true)
  TASK_STATUS="${TASK_STATUS:-000}"
  TASK_RESP="$(tr '\n' ' ' <"$TASK_BODY_FILE" 2>/dev/null || true)"

  if [[ "$TASK_STATUS" =~ ^2[0-9][0-9]$ ]]; then
    ok "POST /v1/agents/task returned HTTP ${TASK_STATUS}"
    log "    response: ${TASK_RESP:0:200}..."
  else
    fail "POST /v1/agents/task returned HTTP ${TASK_STATUS}"
    log "    response: ${TASK_RESP:0:400}..."
  fi
else
  warn "skipped long task; run: just smoke-task"
fi

# ── 8. Doctor ─────────────────────────────────────────────────────────────
section "8. neoland doctor --json"

DOCTOR=$(bash scripts/neoland-run.sh doctor --json 2>/dev/null || echo "")
if [ -n "$DOCTOR" ]; then
  ok "doctor responded"
  log "    ${DOCTOR:0:400}..."
else
  fail "doctor failed"
fi

# ── Summary ───────────────────────────────────────────────────────────────
section "Summary"
log "  PASS: ${PASS}  WARN: ${WARN}  FAIL: ${FAIL}"
log "  Full log: ${LOG}"

if [ "$FAIL" -gt 0 ]; then
  log ""
  log "  Smoke FAILED — fix the failing layers before release."
  exit 1
else
  log ""
  log "  Smoke PASSED — all layers reachable."
fi
