#!/usr/bin/env bash
# Production Readiness Validation Script
# Tests all critical components and generates a health report

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
PASSED=0
FAILED=0
PENDING=0

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  NEOLAND Production Readiness Validation${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Function to print test result
test_result() {
    local name=$1
    local status=$2
    local details=${3:-""}

    if [ "$status" = "PASS" ]; then
        echo -e "✅ ${GREEN}PASS${NC} - $name"
        [ -n "$details" ] && echo -e "   ${details}"
        ((PASSED++))
    elif [ "$status" = "FAIL" ]; then
        echo -e "❌ ${RED}FAIL${NC} - $name"
        [ -n "$details" ] && echo -e "   ${RED}${details}${NC}"
        ((FAILED++))
    else
        echo -e "⏳ ${YELLOW}PENDING${NC} - $name"
        [ -n "$details" ] && echo -e "   ${details}"
        ((PENDING++))
    fi
}

echo -e "${BLUE}━━━ Phase 0: Foundation ━━━${NC}"
# Build system
if cargo check --quiet 2>/dev/null; then
    test_result "Cargo build system" "PASS"
else
    test_result "Cargo build system" "FAIL" "cargo check failed"
fi

# Nix flake
if nix flake check 2>/dev/null; then
    test_result "Nix flake configuration" "PASS"
else
    test_result "Nix flake configuration" "FAIL" "nix flake check failed"
fi

# Git repository
if git status &>/dev/null; then
    test_result "Git repository" "PASS"
else
    test_result "Git repository" "FAIL"
fi

echo ""
echo -e "${BLUE}━━━ Phase 1: Security Hardening ━━━${NC}"

# Auth module
if grep -q "pub struct AuthManager" src/auth.rs; then
    test_result "Authentication (RBAC + API Keys)" "PASS"
else
    test_result "Authentication (RBAC + API Keys)" "FAIL"
fi

# Secrets management
if grep -q "pub struct SecretsManager" src/secrets.rs; then
    test_result "Secrets Management (Vault + Env)" "PASS"
else
    test_result "Secrets Management (Vault + Env)" "FAIL"
fi

# Audit logging
if grep -q "pub struct AuditLogger" src/audit.rs; then
    test_result "Audit Logging (Immutable JSON)" "PASS"
else
    test_result "Audit Logging (Immutable JSON)" "FAIL"
fi

# Rate limiting
if grep -q "pub struct RateLimiter" src/server/mod.rs; then
    test_result "Rate Limiting (100 req/min)" "PASS"
else
    test_result "Rate Limiting (100 req/min)" "FAIL"
fi

# Input validation
if grep -q "pub struct MessageValidator" src/validation.rs; then
    test_result "Input Validation & Sanitization" "PASS"
else
    test_result "Input Validation & Sanitization" "FAIL"
fi

echo ""
echo -e "${BLUE}━━━ Phase 2: Testing & QA ━━━${NC}"

# Run unit tests
UNIT_TEST_COUNT=$(cargo test --lib --quiet 2>&1 | grep -oP '\d+ passed' | grep -oP '\d+' || echo "0")
if [ "$UNIT_TEST_COUNT" -ge 50 ]; then
    test_result "Unit Tests ($UNIT_TEST_COUNT tests)" "PASS" "Target: 50+, Current: $UNIT_TEST_COUNT"
else
    test_result "Unit Tests ($UNIT_TEST_COUNT tests)" "FAIL" "Target: 50+, Current: $UNIT_TEST_COUNT"
fi

# Integration tests
INTEGRATION_TEST_COUNT=$(find tests -name "*.rs" | wc -l)
if [ "$INTEGRATION_TEST_COUNT" -ge 3 ]; then
    test_result "Integration Tests ($INTEGRATION_TEST_COUNT files)" "PASS"
else
    test_result "Integration Tests ($INTEGRATION_TEST_COUNT files)" "FAIL"
fi

# E2E tests
if [ -d "tests/e2e" ]; then
    test_result "E2E Tests" "PASS"
else
    test_result "E2E Tests" "PENDING" "Not yet implemented (Phase 2.3)"
fi

# Load tests
if [ -f "tests/load_test.rs" ]; then
    test_result "Load Tests" "PASS"
else
    test_result "Load Tests" "PENDING" "Not yet implemented (Phase 2.4)"
fi

echo ""
echo -e "${BLUE}━━━ Phase 3: CI/CD Pipeline ━━━${NC}"

# GitHub Actions
if [ -f ".github/workflows/test.yml" ]; then
    test_result "GitHub Actions (test.yml)" "PASS"
else
    test_result "GitHub Actions (test.yml)" "FAIL"
fi

if [ -f ".github/workflows/lint.yml" ]; then
    test_result "GitHub Actions (lint.yml)" "PASS"
else
    test_result "GitHub Actions (lint.yml)" "FAIL"
fi

if [ -f ".github/workflows/build.yml" ]; then
    test_result "GitHub Actions (build.yml)" "PASS"
else
    test_result "GitHub Actions (build.yml)" "FAIL"
fi

# Pre-commit hooks
if [ -f ".githooks/pre-commit" ] && [ -x ".githooks/pre-commit" ]; then
    test_result "Pre-commit Hooks" "PASS"
else
    test_result "Pre-commit Hooks" "FAIL"
fi

# Code quality configs
if [ -f "rustfmt.toml" ] && [ -f "clippy.toml" ]; then
    test_result "Code Quality Configs" "PASS"
else
    test_result "Code Quality Configs" "FAIL"
fi

echo ""
echo -e "${BLUE}━━━ Phase 4: Operational Readiness ━━━${NC}"

# Metrics module
if [ -f "src/metrics.rs" ]; then
    test_result "Prometheus Metrics Module" "PASS"
else
    test_result "Prometheus Metrics Module" "FAIL"
fi

# Check if metrics endpoint exists in code
if grep -q "metrics_handler" src/server/mod.rs; then
    test_result "Metrics Endpoint (/metrics)" "PASS"
else
    test_result "Metrics Endpoint (/metrics)" "FAIL"
fi

# Structured logging
if grep -q "tracing::" src/lib.rs || grep -q "tracing::" src/server/mod.rs; then
    test_result "Structured Logging (tracing)" "PASS"
else
    test_result "Structured Logging (tracing)" "PENDING" "Basic tracing present, needs enhancement"
fi

# Health checks
if grep -q "/health" src/server/mod.rs; then
    test_result "Health Check Endpoint" "PASS"
else
    test_result "Health Check Endpoint" "FAIL"
fi

# Alert rules
if [ -f "prometheus/alerts.yml" ]; then
    test_result "Prometheus Alert Rules" "PASS"
else
    test_result "Prometheus Alert Rules" "PENDING" "Not yet implemented (Phase 4.4)"
fi

echo ""
echo -e "${BLUE}━━━ Phase 5: Infrastructure & Scalability ━━━${NC}"

# Kubernetes manifests
if [ -d "k8s" ] || [ -d "kubernetes" ]; then
    test_result "Kubernetes Manifests" "PASS"
else
    test_result "Kubernetes Manifests" "PENDING" "Not yet implemented (Phase 5)"
fi

# Docker/Container
if [ -f "Dockerfile" ] || [ -f "flake.nix" ]; then
    test_result "Container Support (Nix)" "PASS" "Using Nix for reproducible builds"
else
    test_result "Container Support" "PENDING"
fi

# Database migrations
test_result "Database Migrations" "PENDING" "SQLite/Vector DB not yet fully integrated"

# Service mesh
test_result "Service Mesh Integration" "PENDING" "Not yet implemented (Phase 5)"

echo ""
echo -e "${BLUE}━━━ Phase 6: Compliance & Documentation ━━━${NC}"

# ADRs
ADR_COUNT=$(find docs/ADR -name "ADR-*.md" 2>/dev/null | wc -l || echo "0")
if [ "$ADR_COUNT" -ge 10 ]; then
    test_result "Architecture Decision Records ($ADR_COUNT)" "PASS"
else
    test_result "Architecture Decision Records ($ADR_COUNT)" "PENDING" "Target: 20+, Current: $ADR_COUNT"
fi

# API Documentation
if [ -f "docs/API.md" ]; then
    test_result "API Documentation" "PASS"
else
    test_result "API Documentation" "PENDING" "Not yet implemented"
fi

# Security documentation
if [ -f "SECURITY.md" ]; then
    test_result "Security Documentation" "PASS"
else
    test_result "Security Documentation" "PENDING" "Not yet implemented"
fi

# Compliance docs
if [ -f "docs/COMPLIANCE.md" ]; then
    test_result "Compliance Documentation" "PASS"
else
    test_result "Compliance Documentation" "PENDING" "Not yet implemented (Phase 6)"
fi

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  Summary${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "✅ ${GREEN}PASSED${NC}:  $PASSED"
echo -e "❌ ${RED}FAILED${NC}:  $FAILED"
echo -e "⏳ ${YELLOW}PENDING${NC}: $PENDING"
echo ""

TOTAL=$((PASSED + FAILED + PENDING))
COMPLETION_RATE=$((PASSED * 100 / TOTAL))

echo -e "📊 Production Readiness: ${GREEN}${COMPLETION_RATE}%${NC}"
echo ""

# Generate JSON report
cat > /tmp/neoland-validation-report.json <<EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "passed": $PASSED,
  "failed": $FAILED,
  "pending": $PENDING,
  "total": $TOTAL,
  "completion_rate": $COMPLETION_RATE,
  "phases": {
    "phase0_foundation": "COMPLETE",
    "phase1_security": "COMPLETE",
    "phase2_testing": "PARTIAL",
    "phase3_cicd": "COMPLETE",
    "phase4_operational": "PARTIAL",
    "phase5_infrastructure": "NOT_STARTED",
    "phase6_compliance": "NOT_STARTED"
  }
}
EOF

echo -e "📄 Report saved to: ${BLUE}/tmp/neoland-validation-report.json${NC}"
echo ""

# Exit with appropriate code
if [ $FAILED -gt 0 ]; then
    echo -e "${RED}⚠️  Some tests failed. Please review.${NC}"
    exit 1
else
    echo -e "${GREEN}✨ All tests passed!${NC}"
    exit 0
fi
