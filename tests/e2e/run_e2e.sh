#!/usr/bin/env bash
# E2E Test Runner for NEOLAND TUI
# Runs all end-to-end tests in sequence

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SERVER_URL="${SERVER_URL:-http://localhost:3001}"
ML_API_URL="${ML_API_URL:-http://localhost:8000}"
E2E_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$E2E_DIR/../.." && pwd)"

# Test results
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_SKIPPED=0

echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   NEOLAND E2E Test Suite                  ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Configuration:${NC}"
echo -e "  Server URL:  ${SERVER_URL}"
echo -e "  ML API URL:  ${ML_API_URL}"
echo -e "  Project:     ${PROJECT_ROOT}"
echo -e "  E2E Tests:   ${E2E_DIR}"
echo ""

# Check prerequisites
echo -e "${BLUE}Checking prerequisites...${NC}"

if ! command -v expect &> /dev/null; then
    echo -e "${RED}❌ FAIL: expect not installed${NC}"
    echo -e "   Install: sudo apt install expect (Debian/Ubuntu)"
    echo -e "           sudo dnf install expect (Fedora/RHEL)"
    echo -e "           brew install expect (macOS)"
    exit 1
fi
echo -e "${GREEN}✅ expect installed${NC}"

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ FAIL: cargo not installed${NC}"
    exit 1
fi
echo -e "${GREEN}✅ cargo installed${NC}"

# Build project first
echo ""
echo -e "${BLUE}Building project...${NC}"
cd "$PROJECT_ROOT"
if cargo build --release; then
    echo -e "${GREEN}✅ Build successful${NC}"
else
    echo -e "${RED}❌ FAIL: Build failed${NC}"
    exit 1
fi

# Check if server is running
echo ""
echo -e "${BLUE}Checking server availability...${NC}"
if curl -s "${SERVER_URL}/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Server is running at ${SERVER_URL}${NC}"
else
    echo -e "${YELLOW}⚠️  WARNING: Server not responding at ${SERVER_URL}${NC}"
    echo -e "   Starting server in background..."
    cargo run --release --bin neoland -- server &
    SERVER_PID=$!
    sleep 5

    if curl -s "${SERVER_URL}/health" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Server started successfully${NC}"
    else
        echo -e "${RED}❌ FAIL: Could not start server${NC}"
        kill $SERVER_PID 2>/dev/null || true
        exit 1
    fi
fi

# Run tests
echo ""
echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Running E2E Tests                        ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
echo ""

run_test() {
    local test_name="$1"
    local test_file="$2"
    local skip_reason="$3"

    TESTS_RUN=$((TESTS_RUN + 1))

    if [ -n "$skip_reason" ]; then
        echo -e "${YELLOW}⏭️  SKIP: ${test_name}${NC}"
        echo -e "   Reason: ${skip_reason}"
        TESTS_SKIPPED=$((TESTS_SKIPPED + 1))
        return 0
    fi

    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}Running: ${test_name}${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

    if expect "$test_file" "$SERVER_URL" "$ML_API_URL"; then
        echo -e "${GREEN}✅ PASS: ${test_name}${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        echo -e "${RED}❌ FAIL: ${test_name}${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Test 1: Smoke Test
run_test "TUI Smoke Test" "${E2E_DIR}/tui_smoke_test.exp"

# Test 2: Presets Test
run_test "TUI Presets Test" "${E2E_DIR}/tui_presets_test.exp"

# Test 3: Fallback Test (optional - requires ml-offload to be down)
if [ "${RUN_FALLBACK_TEST}" = "1" ]; then
    run_test "TUI Fallback Chain Test" "${E2E_DIR}/tui_fallback_test.exp"
else
    run_test "TUI Fallback Chain Test" "${E2E_DIR}/tui_fallback_test.exp" "Requires ml-offload to be stopped (set RUN_FALLBACK_TEST=1)"
fi

# Cleanup
if [ -n "$SERVER_PID" ]; then
    echo ""
    echo -e "${BLUE}Stopping background server...${NC}"
    kill $SERVER_PID 2>/dev/null || true
    echo -e "${GREEN}✅ Cleanup complete${NC}"
fi

# Final report
echo ""
echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Test Results Summary                     ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
echo ""
echo -e "  Tests Run:     ${TESTS_RUN}"
echo -e "  ${GREEN}Tests Passed:  ${TESTS_PASSED}${NC}"
echo -e "  ${RED}Tests Failed:  ${TESTS_FAILED}${NC}"
echo -e "  ${YELLOW}Tests Skipped: ${TESTS_SKIPPED}${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}╔════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║   ✅ All E2E tests passed!                ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════╝${NC}"
    exit 0
else
    echo -e "${RED}╔════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║   ❌ Some E2E tests failed                ║${NC}"
    echo -e "${RED}╚════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${YELLOW}Debugging tips:${NC}"
    echo -e "  1. Check server logs: journalctl -u neoland -f"
    echo -e "  2. Verify server health: curl ${SERVER_URL}/health"
    echo -e "  3. Check ml-offload: curl ${ML_API_URL}/health"
    echo -e "  4. Run tests individually:"
    echo -e "     expect ${E2E_DIR}/tui_smoke_test.exp ${SERVER_URL} ${ML_API_URL}"
    echo ""
    exit 1
fi
