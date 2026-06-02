#!/usr/bin/env bash
# Full-stack smoke: Neoland → SecureLLM Bridge → ml-ops-api → llama.cpp
# Records evidence to /tmp/neoland-smoke-<timestamp>.log
# Usage: nix develop --command bash scripts/smoke-full-stack.sh

set -euo pipefail

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

log() { echo "$*" | tee -a "$LOG"; }
ok()   { log "  [OK]  $*"; PASS=$((PASS+1)); }
fail() { log "  [FAIL] $*"; FAIL=$((FAIL+1)); }
section() { log ""; log "── $* ──────────────────────────────────────────────────"; }

NEOLAND_URL="${NEOLAND_CONTROL_PLANE_URL:-http://127.0.0.1:3001}"
GATEWAY_URL="${NEOLAND_GATEWAY_URL:-http://127.0.0.1:8080}"
MLOPS_URL="${ML_OPS_API_URL:-http://127.0.0.1:8083}"
LLAMACPP_URL="${LLAMACPP_URL:-http://127.0.0.1:8081}"
DSPY_URL="${NEOLAND_DSPY_URL:-http://127.0.0.1:8001}"
API_KEY="${NEOLAND_ADMIN_API_KEY:-neoland_admin_dev_key_change_in_production}"

log "Neoland Full-Stack Smoke — ${TIMESTAMP}"
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
section "4. ml-ops-api (${MLOPS_URL})"

if curl -sf "${MLOPS_URL}/health" -o /dev/null 2>/dev/null; then
  ok "/health reachable"
else
  fail "/health unreachable at ${MLOPS_URL} — start ml-ops-api"
fi

# ── 5. llama.cpp ──────────────────────────────────────────────────────────
section "5. llama.cpp (${LLAMACPP_URL})"

if curl -sf "${LLAMACPP_URL}/health" -o /dev/null 2>/dev/null; then
  ok "/health reachable"
else
  fail "/health unreachable at ${LLAMACPP_URL} — start llama-server"
fi

# ── 6. End-to-end task ───────────────────────────────────────────────────
section "6. End-to-end task (Neoland → pipeline)"

TASK_RESP=$(curl -s -X POST "${NEOLAND_URL}/v1/agents/task" \
  -H "X-API-Key: ${API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"task":"smoke test: confirm pipeline is reachable"}' \
  --max-time 30 2>/dev/null || echo "")

if [ -n "$TASK_RESP" ]; then
  ok "POST /v1/agents/task returned response"
  log "    response: ${TASK_RESP:0:200}..."
else
  fail "POST /v1/agents/task failed or timed out"
fi

# ── 7. Doctor ─────────────────────────────────────────────────────────────
section "7. neoland doctor --json"

DOCTOR=$(nix develop --command cargo run --bin neoland --quiet -- doctor --json 2>/dev/null || echo "")
if [ -n "$DOCTOR" ]; then
  ok "doctor responded"
  log "    ${DOCTOR:0:400}..."
else
  fail "doctor failed"
fi

# ── Summary ───────────────────────────────────────────────────────────────
section "Summary"
log "  PASS: ${PASS}  FAIL: ${FAIL}"
log "  Full log: ${LOG}"

if [ "$FAIL" -gt 0 ]; then
  log ""
  log "  Smoke FAILED — fix the failing layers before release."
  exit 1
else
  log ""
  log "  Smoke PASSED — all layers reachable."
fi
