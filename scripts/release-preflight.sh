#!/usr/bin/env bash
# Neoland Release Preflight
# Runs all gates required before tagging a public pre-release.
# Usage: nix develop --command bash scripts/release-preflight.sh
# Exit 0 = all gates pass. Exit 1 = one or more gates failed.

set -euo pipefail

TIMESTAMP=$(date +%Y%m%d-%H%M%S)
LOG="/tmp/neoland-preflight-${TIMESTAMP}.log"
PASS=0
FAIL=0

log()     { echo "$*" | tee -a "$LOG"; }
ok()      { log "  ✓  $*"; PASS=$((PASS+1)); }
fail()    { log "  ✗  $*"; FAIL=$((FAIL+1)); }
section() { log ""; log "── $* "; }

log "Neoland Release Preflight — ${TIMESTAMP}"
log "Log: ${LOG}"

# ── Gate 1: Rust unit tests ──────────────────────────────────────────────
section "Gate 1: Rust unit tests (cargo test --lib)"

if nix develop --command cargo test --lib --quiet 2>&1 | tee -a "$LOG" | grep -q "test result: ok"; then
  ok "cargo test --lib passed"
else
  fail "cargo test --lib failed"
fi

# ── Gate 2: Clippy ──────────────────────────────────────────────────────
section "Gate 2: Clippy (cargo clippy)"

if nix develop --command cargo clippy --all-targets -- -D warnings 2>&1 | tee -a "$LOG"; then
  ok "clippy clean"
else
  fail "clippy warnings/errors"
fi

# ── Gate 3: E2E REST tests ───────────────────────────────────────────────
section "Gate 3: E2E REST API tests"

if nix develop --command cargo test --test rest_api_test -- --nocapture 2>&1 | tee -a "$LOG" | grep -q "test result: ok"; then
  ok "REST API E2E passed"
else
  fail "REST API E2E failed"
fi

# ── Gate 4: TUI functional E2E ──────────────────────────────────────────
section "Gate 4: TUI functional E2E"

if nix develop --command bash tests/e2e/run_e2e.sh 2>&1 | tee -a "$LOG"; then
  ok "TUI functional E2E passed"
else
  fail "TUI functional E2E failed"
fi

# ── Gate 5: Python contract tests ────────────────────────────────────────
section "Gate 5: Python contract tests (pytest -m contract)"

if nix develop --command bash -c "cd agents && poetry run pytest tests/ -m contract -q" 2>&1 | tee -a "$LOG"; then
  ok "Python contract tests passed"
else
  fail "Python contract tests failed"
fi

# ── Gate 6: neoland doctor ──────────────────────────────────────────────
section "Gate 6: neoland doctor --json"

DOCTOR=$(nix develop --command cargo run --bin neoland --quiet -- doctor --json 2>/dev/null || echo "")
if [ -n "$DOCTOR" ]; then
  ok "doctor responded"
  log "    ${DOCTOR:0:500}..."
else
  fail "doctor failed to respond"
fi

# ── Gate 7: Full stack task smoke ────────────────────────────────────────
section "Gate 7: Full stack task smoke"

if bash scripts/smoke-full-stack.sh --task 2>&1 | tee -a "$LOG"; then
  ok "Full stack task smoke passed"
else
  fail "Full stack task smoke failed — check PostgreSQL, DSPy, and LLM availability"
fi

# ── Summary ──────────────────────────────────────────────────────────────
section "Preflight Summary"
log "  PASS: ${PASS}  FAIL: ${FAIL}"
log "  Full log: ${LOG}"

if [ "$FAIL" -gt 0 ]; then
  log ""
  log "  PREFLIGHT FAILED — do not tag release until all gates pass."
  exit 1
else
  log ""
  log "  PREFLIGHT PASSED — ready to tag public pre-release."
fi
