#!/usr/bin/env bash
# NEOLAND Security Testing Suite Runner
# Runs all security tests: fuzzing, penetration testing, static analysis

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SERVER_URL="${SERVER_URL:-http://localhost:3001}"
GRPC_URL="${GRPC_URL:-http://localhost:50051}"
REPORTS_DIR="${PROJECT_ROOT}/target/security-reports"

echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   NEOLAND Security Testing Suite          ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Configuration:${NC}"
echo -e "  Project Root:  ${PROJECT_ROOT}"
echo -e "  Server URL:    ${SERVER_URL}"
echo -e "  gRPC URL:      ${GRPC_URL}"
echo -e "  Reports Dir:   ${REPORTS_DIR}"
echo ""

# Create reports directory
mkdir -p "${REPORTS_DIR}"

# Test categories
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_SKIPPED=0

run_test_category() {
    local category="$1"
    local description="$2"

    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}Category: ${category}${NC}"
    echo -e "${CYAN}Description: ${description}${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

# ============================================================================
# Phase 1: Static Security Analysis
# ============================================================================

run_test_category "Static Analysis" "Dependency vulnerabilities and code quality"

echo -e "${MAGENTA}📋 Phase 1.1: cargo-audit (Dependency Vulnerabilities)${NC}"
if command -v cargo-audit &> /dev/null; then
    if cargo audit --json > "${REPORTS_DIR}/cargo-audit.json" 2>&1; then
        echo -e "${GREEN}✅ PASS: No known vulnerabilities found${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}❌ FAIL: Vulnerabilities detected${NC}"
        echo -e "   Report: ${REPORTS_DIR}/cargo-audit.json"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
else
    echo -e "${YELLOW}⏭️  SKIP: cargo-audit not installed${NC}"
    echo -e "   Install: cargo install cargo-audit"
    TESTS_SKIPPED=$((TESTS_SKIPPED + 1))
fi
TESTS_RUN=$((TESTS_RUN + 1))
echo ""

echo -e "${MAGENTA}📋 Phase 1.2: cargo-deny (License & Ban Compliance)${NC}"
if command -v cargo-deny &> /dev/null; then
    if cargo deny check --config deny.toml > "${REPORTS_DIR}/cargo-deny.log" 2>&1; then
        echo -e "${GREEN}✅ PASS: All deny checks passed${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}❌ FAIL: cargo-deny checks failed${NC}"
        echo -e "   Report: ${REPORTS_DIR}/cargo-deny.log"
        cat "${REPORTS_DIR}/cargo-deny.log"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
else
    echo -e "${YELLOW}⏭️  SKIP: cargo-deny not installed${NC}"
    echo -e "   Install: cargo install cargo-deny"
    TESTS_SKIPPED=$((TESTS_SKIPPED + 1))
fi
TESTS_RUN=$((TESTS_RUN + 1))
echo ""

echo -e "${MAGENTA}📋 Phase 1.3: cargo-clippy (Security Lints)${NC}"
if cargo clippy --all-targets --all-features -- \
    -D warnings \
    -W clippy::suspicious \
    -W clippy::security \
    > "${REPORTS_DIR}/clippy-security.log" 2>&1; then
    echo -e "${GREEN}✅ PASS: No security lints triggered${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}❌ FAIL: Security lints detected${NC}"
    echo -e "   Report: ${REPORTS_DIR}/clippy-security.log"
    tail -20 "${REPORTS_DIR}/clippy-security.log"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
TESTS_RUN=$((TESTS_RUN + 1))
echo ""

echo -e "${MAGENTA}📋 Phase 1.4: Secret Scanning${NC}"
# Check for hardcoded secrets in source code
if command -v rg &> /dev/null; then
    echo "   Scanning for hardcoded secrets..."

    SECRET_PATTERNS=(
        'api[_-]?key.*=.*["\047][a-zA-Z0-9]{20,}["\047]'
        'password.*=.*["\047].*["\047]'
        'secret.*=.*["\047].*["\047]'
        'token.*=.*["\047][a-zA-Z0-9]{20,}["\047]'
        'AKIA[0-9A-Z]{16}' # AWS access key
        'sk_live_[a-zA-Z0-9]{24}' # Stripe live key
        'ghp_[a-zA-Z0-9]{36}' # GitHub personal access token
    )

    SECRETS_FOUND=0
    for pattern in "${SECRET_PATTERNS[@]}"; do
        if rg -i "$pattern" src/ --json > /dev/null 2>&1; then
            SECRETS_FOUND=$((SECRETS_FOUND + 1))
            echo -e "${RED}   ⚠️  Potential secret found: ${pattern}${NC}"
        fi
    done

    if [ $SECRETS_FOUND -eq 0 ]; then
        echo -e "${GREEN}✅ PASS: No hardcoded secrets detected${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}❌ FAIL: Found ${SECRETS_FOUND} potential secrets${NC}"
        echo -e "   Run: rg -i 'api_key|password|secret|token' src/"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
else
    echo -e "${YELLOW}⏭️  SKIP: ripgrep not installed${NC}"
    TESTS_SKIPPED=$((TESTS_SKIPPED + 1))
fi
TESTS_RUN=$((TESTS_RUN + 1))
echo ""

# ============================================================================
# Phase 2: Server Availability Check
# ============================================================================

run_test_category "Server Check" "Verify server is running for dynamic tests"

echo -e "${MAGENTA}🔍 Checking if NEOLAND server is running...${NC}"
if curl -s "${SERVER_URL}/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Server is running at ${SERVER_URL}${NC}"
    SERVER_RUNNING=true
else
    echo -e "${YELLOW}⚠️  WARNING: Server not responding at ${SERVER_URL}${NC}"
    echo -e "   Dynamic tests will be skipped."
    echo -e "   Start server: cargo run --release --bin neoland -- server"
    SERVER_RUNNING=false
fi
echo ""

# ============================================================================
# Phase 3: Dynamic Security Testing (Fuzzing)
# ============================================================================

if [ "$SERVER_RUNNING" = true ]; then
    run_test_category "Fuzzing Tests" "REST API and gRPC endpoint fuzzing"

    FUZZ_TESTS=(
        "fuzz_rest_api_invalid_json"
        "fuzz_rest_api_invalid_headers"
        "test_rate_limit_enforcement"
        "test_authentication_bypass_attempts"
        "test_path_traversal_attempts"
        "test_resource_exhaustion_attacks"
    )

    for test in "${FUZZ_TESTS[@]}"; do
        echo -e "${MAGENTA}🧪 Running: ${test}${NC}"

        if cargo test --test fuzz_endpoints "${test}" -- --ignored --nocapture 2>&1 | tee "${REPORTS_DIR}/${test}.log"; then
            echo -e "${GREEN}✅ PASS: ${test}${NC}"
            TESTS_PASSED=$((TESTS_PASSED + 1))
        else
            echo -e "${RED}❌ FAIL: ${test}${NC}"
            echo -e "   Log: ${REPORTS_DIR}/${test}.log"
            TESTS_FAILED=$((TESTS_FAILED + 1))
        fi
        TESTS_RUN=$((TESTS_RUN + 1))
        echo ""
    done
else
    echo -e "${YELLOW}⏭️  SKIP: Fuzzing tests (server not running)${NC}"
    TESTS_SKIPPED=$((TESTS_SKIPPED + 6))
    TESTS_RUN=$((TESTS_RUN + 6))
fi

# ============================================================================
# Phase 4: Penetration Testing
# ============================================================================

if [ "$SERVER_RUNNING" = true ]; then
    run_test_category "Penetration Tests" "Real-world attack scenarios"

    PENTEST_TESTS=(
        "test_cors_policy"
        "test_information_disclosure"
        "test_tls_configuration"
        "test_session_management"
        "test_file_upload_security"
        "test_grpc_security"
        "test_timing_attack_resistance"
        "test_ssrf_protection"
    )

    for test in "${PENTEST_TESTS[@]}"; do
        echo -e "${MAGENTA}🔓 Running: ${test}${NC}"

        if cargo test --test pentest_scenarios "${test}" -- --ignored --nocapture 2>&1 | tee "${REPORTS_DIR}/${test}.log"; then
            echo -e "${GREEN}✅ PASS: ${test}${NC}"
            TESTS_PASSED=$((TESTS_PASSED + 1))
        else
            echo -e "${RED}❌ FAIL: ${test}${NC}"
            echo -e "   Log: ${REPORTS_DIR}/${test}.log"
            TESTS_FAILED=$((TESTS_FAILED + 1))
        fi
        TESTS_RUN=$((TESTS_RUN + 1))
        echo ""
    done
else
    echo -e "${YELLOW}⏭️  SKIP: Penetration tests (server not running)${NC}"
    TESTS_SKIPPED=$((TESTS_SKIPPED + 8))
    TESTS_RUN=$((TESTS_RUN + 8))
fi

# ============================================================================
# Final Report
# ============================================================================

echo ""
echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Security Test Results Summary            ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
echo ""
echo -e "  Tests Run:     ${TESTS_RUN}"
echo -e "  ${GREEN}Tests Passed:  ${TESTS_PASSED}${NC}"
echo -e "  ${RED}Tests Failed:  ${TESTS_FAILED}${NC}"
echo -e "  ${YELLOW}Tests Skipped: ${TESTS_SKIPPED}${NC}"
echo ""
echo -e "  Reports saved to: ${REPORTS_DIR}"
echo ""

# Security score calculation
TOTAL_DYNAMIC=$((TESTS_RUN - TESTS_SKIPPED))
if [ $TOTAL_DYNAMIC -gt 0 ]; then
    SECURITY_SCORE=$(( (TESTS_PASSED * 100) / TOTAL_DYNAMIC ))
else
    SECURITY_SCORE=0
fi

echo -e "${BLUE}Security Score: ${SECURITY_SCORE}/100${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}╔════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║   ✅ All security tests passed!           ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════╝${NC}"
    exit 0
else
    echo -e "${RED}╔════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║   ❌ Some security tests failed           ║${NC}"
    echo -e "${RED}╚════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${YELLOW}Next Steps:${NC}"
    echo -e "  1. Review failed test logs in ${REPORTS_DIR}"
    echo -e "  2. Fix security issues identified"
    echo -e "  3. Re-run: ./tests/security/run_security_tests.sh"
    echo ""
    exit 1
fi
