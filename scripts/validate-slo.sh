#!/usr/bin/env bash
# SLO validation for Neoland control plane endpoints.
# Runs a short load burst and checks p99 and RPS against defined targets.
# Requires: hey (load generator) — available in nix develop shell.
# Usage: nix develop --command bash scripts/validate-slo.sh

set -euo pipefail

NEOLAND_URL="${NEOLAND_CONTROL_PLANE_URL:-http://127.0.0.1:3001}"
REQUESTS="${SLO_REQUESTS:-2000}"
CONCURRENCY="${SLO_CONCURRENCY:-50}"
PASS=0
FAIL=0

log()  { echo "$*"; }
ok()   { echo "  ✓  $*"; PASS=$((PASS+1)); }
fail() { echo "  ✗  $*" >&2; FAIL=$((FAIL+1)); }

# ── SLO targets (edit here to tighten/relax) ─────────────────────────────────
# /live  — pure in-memory liveness check; no I/O, must be extremely fast
LIVE_MIN_RPS=10000
LIVE_MAX_P99_MS=50

# /health — polls every runtime layer (DB, LLM, NATS, ...); intentionally I/O-bound
HEALTH_MAX_P99_MS=1000

check_hey() {
  if ! command -v hey >/dev/null 2>&1; then
    echo "hey not found — install via nix develop or: go install github.com/rakyll/hey@latest"
    exit 1
  fi
}

run_hey() {
  local endpoint="$1"
  hey -n "$REQUESTS" -c "$CONCURRENCY" "${NEOLAND_URL}${endpoint}" 2>/dev/null
}

extract_p99() {
  # hey outputs: "  99% in X.XXXX secs"
  grep "99%" | awk '{print $3}' | awk '{printf "%.0f", $1 * 1000}'
}

extract_rps() {
  grep "Requests/sec:" | awk '{printf "%.0f", $2}'
}

# ── Gate: server reachable ────────────────────────────────────────────────────
log ""
log "── SLO Validation — ${NEOLAND_URL} ──"
log "   Requests per endpoint: ${REQUESTS}   Concurrency: ${CONCURRENCY}"
log ""

if ! curl -sf "${NEOLAND_URL}/live" -o /dev/null 2>/dev/null; then
  echo "Server unreachable at ${NEOLAND_URL}/live — start with 'just server' first."
  exit 1
fi

check_hey

# ── /live ─────────────────────────────────────────────────────────────────────
log "── /live (liveness — no I/O)"
LIVE_OUT=$(run_hey /live)
LIVE_P99=$(echo "$LIVE_OUT" | extract_p99)
LIVE_RPS=$(echo "$LIVE_OUT" | extract_rps)

log "   RPS: ${LIVE_RPS}   p99: ${LIVE_P99}ms   (targets: >=${LIVE_MIN_RPS} RPS, p99 <=${LIVE_MAX_P99_MS}ms)"

if [ "${LIVE_RPS}" -ge "${LIVE_MIN_RPS}" ]; then
  ok "/live RPS ${LIVE_RPS} >= ${LIVE_MIN_RPS}"
else
  fail "/live RPS ${LIVE_RPS} < target ${LIVE_MIN_RPS}"
fi

if [ "${LIVE_P99}" -le "${LIVE_MAX_P99_MS}" ]; then
  ok "/live p99 ${LIVE_P99}ms <= ${LIVE_MAX_P99_MS}ms"
else
  fail "/live p99 ${LIVE_P99}ms > target ${LIVE_MAX_P99_MS}ms"
fi

# ── /health ───────────────────────────────────────────────────────────────────
log ""
log "── /health (composite — I/O-bound by design)"
HEALTH_OUT=$(run_hey /health)
HEALTH_P99=$(echo "$HEALTH_OUT" | extract_p99)
HEALTH_RPS=$(echo "$HEALTH_OUT" | extract_rps)

log "   RPS: ${HEALTH_RPS}   p99: ${HEALTH_P99}ms   (target: p99 <=${HEALTH_MAX_P99_MS}ms)"

if [ "${HEALTH_P99}" -le "${HEALTH_MAX_P99_MS}" ]; then
  ok "/health p99 ${HEALTH_P99}ms <= ${HEALTH_MAX_P99_MS}ms"
else
  fail "/health p99 ${HEALTH_P99}ms > target ${HEALTH_MAX_P99_MS}ms"
fi

# ── Summary ───────────────────────────────────────────────────────────────────
log ""
log "── SLO Summary"
log "   PASS: ${PASS}  FAIL: ${FAIL}"

if [ "${FAIL}" -gt 0 ]; then
  log "   SLO FAILED"
  exit 1
fi

log "   SLO PASSED"
