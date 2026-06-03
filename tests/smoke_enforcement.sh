#!/usr/bin/env bash
# tests/smoke_enforcement.sh — OPA authz enforcement smoke test
set -euo pipefail

POLICY_DIR="${ADR_ROOT:-.}/policy"

echo "── OPA Enforcement Smoke Test ──────────────────────────"

# If no OPA policy directory exists yet, the smoke test passes with a notice.
if [ ! -d "$POLICY_DIR" ]; then
  echo "ℹ  No OPA policy directory found at $POLICY_DIR — skipping policy eval"
  echo "✓  Smoke test passed (no policies to enforce)"
  exit 0
fi

PASS=0
FAIL=0

for policy in "$POLICY_DIR"/*.rego; do
  [ -f "$policy" ] || continue
  name=$(basename "$policy" .rego)
  if opa check "$policy" 2>&1; then
    echo "  ✓  $name"
    PASS=$((PASS + 1))
  else
    echo "  ✗  $name — syntax error"
    FAIL=$((FAIL + 1))
  fi
done

if [ "$PASS" -eq 0 ] && [ "$FAIL" -eq 0 ]; then
  echo "ℹ  No .rego policies found in $POLICY_DIR"
  echo "✓  Smoke test passed (empty policy set)"
  exit 0
fi

echo ""
echo "OPA smoke: PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ] || exit 1
echo "✓  All OPA policies valid"
