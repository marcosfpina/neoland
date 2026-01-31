#!/usr/bin/env bash
# NEOLAND Comprehensive Load Testing Suite
# Orchestrates all load tests: gRPC, REST, resource monitoring

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
LOAD_TEST_DIR="${PROJECT_ROOT}/tests/load"
REPORTS_DIR="${PROJECT_ROOT}/target/load-reports"
SERVER_URL="${SERVER_URL:-http://localhost:3001}"
GRPC_URL="${GRPC_URL:-localhost:50051}"

# Test modes
RUN_GRPC="${RUN_GRPC:-true}"
RUN_REST="${RUN_REST:-true}"
RUN_BENCHMARKS="${RUN_BENCHMARKS:-false}"
RUN_MONITORING="${RUN_MONITORING:-true}"

echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   NEOLAND Load Testing Suite              ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Configuration:${NC}"
echo -e "  Project Root:  ${PROJECT_ROOT}"
echo -e "  Server URL:    ${SERVER_URL}"
echo -e "  gRPC URL:      ${GRPC_URL}"
echo -e "  Reports Dir:   ${REPORTS_DIR}"
echo ""
echo -e "${BLUE}Test Plan:${NC}"
echo -e "  gRPC Load Test:    ${RUN_GRPC}"
echo -e "  REST Load Test:    ${RUN_REST}"
echo -e "  Rust Benchmarks:   ${RUN_BENCHMARKS}"
echo -e "  Resource Monitor:  ${RUN_MONITORING}"
echo ""

# Create reports directory
mkdir -p "${REPORTS_DIR}"

# Test results
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# ============================================================================
# Phase 1: Pre-flight Checks
# ============================================================================

echo -e "${MAGENTA}📋 Phase 1: Pre-flight Checks${NC}"
echo ""

# Check if server is running
echo -e "${CYAN}Checking NEOLAND server...${NC}"
if curl -s "${SERVER_URL}/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ REST server is running at ${SERVER_URL}${NC}"
    SERVER_RUNNING=true
else
    echo -e "${YELLOW}⚠️  WARNING: REST server not responding at ${SERVER_URL}${NC}"
    echo -e "   You can:"
    echo -e "   1. Start manually: cargo run --release --bin neoland -- server &"
    echo -e "   2. Let script start it (will run in background)"
    echo ""
    read -p "   Start server automatically? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${CYAN}Starting server in background...${NC}"
        cd "${PROJECT_ROOT}"
        cargo run --release --bin neoland -- server > "${REPORTS_DIR}/server.log" 2>&1 &
        SERVER_PID=$!
        echo -e "${BLUE}Server PID: ${SERVER_PID}${NC}"

        # Wait for server to be ready
        echo -e "${CYAN}Waiting for server to be ready...${NC}"
        for i in {1..30}; do
            if curl -s "${SERVER_URL}/health" > /dev/null 2>&1; then
                echo -e "${GREEN}✅ Server started successfully${NC}"
                SERVER_RUNNING=true
                break
            fi
            sleep 2
            echo -n "."
        done
        echo ""

        if [ "$SERVER_RUNNING" != "true" ]; then
            echo -e "${RED}❌ Server failed to start${NC}"
            echo -e "   Check logs: ${REPORTS_DIR}/server.log"
            exit 1
        fi
    else
        SERVER_RUNNING=false
    fi
fi

echo ""

# Check for required tools
echo -e "${CYAN}Checking required tools...${NC}"

TOOLS_OK=true

if [ "$RUN_GRPC" = "true" ]; then
    if ! command -v ghz &> /dev/null; then
        echo -e "${YELLOW}⚠️  ghz not installed (gRPC load test will be skipped)${NC}"
        RUN_GRPC=false
        TOOLS_OK=false
    else
        echo -e "${GREEN}✅ ghz installed${NC}"
    fi
fi

if [ "$RUN_REST" = "true" ]; then
    if ! command -v wrk &> /dev/null && ! command -v ab &> /dev/null; then
        echo -e "${YELLOW}⚠️  Neither wrk nor ab installed (REST load test will be skipped)${NC}"
        RUN_REST=false
        TOOLS_OK=false
    else
        if command -v wrk &> /dev/null; then
            echo -e "${GREEN}✅ wrk installed${NC}"
        else
            echo -e "${GREEN}✅ Apache Bench (ab) installed${NC}"
        fi
    fi
fi

echo ""

if [ "$TOOLS_OK" = "false" ]; then
    echo -e "${YELLOW}💡 Install missing tools:${NC}"
    echo -e "   ghz:  go install github.com/bojand/ghz/cmd/ghz@latest"
    echo -e "   wrk:  brew install wrk (macOS) or build from source"
    echo -e "   ab:   sudo apt install apache2-utils (Linux)"
    echo ""
fi

# ============================================================================
# Phase 2: Rust Benchmarks (Optional)
# ============================================================================

if [ "$RUN_BENCHMARKS" = "true" ]; then
    echo -e "${MAGENTA}📋 Phase 2: Rust Benchmarks (Criterion)${NC}"
    echo ""

    cd "${PROJECT_ROOT}"

    if cargo bench --no-run 2>&1 | grep -q "inference_benchmark"; then
        echo -e "${CYAN}Running Criterion benchmarks...${NC}"

        cargo bench --bench inference_benchmark \
            -- --output-format bencher \
            > "${REPORTS_DIR}/criterion_results.txt" 2>&1

        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✅ Benchmarks completed${NC}"
            echo -e "   Results: ${REPORTS_DIR}/criterion_results.txt"
            echo -e "   Detailed: ${PROJECT_ROOT}/target/criterion/"
            TESTS_PASSED=$((TESTS_PASSED + 1))
        else
            echo -e "${RED}❌ Benchmarks failed${NC}"
            TESTS_FAILED=$((TESTS_FAILED + 1))
        fi
        TESTS_RUN=$((TESTS_RUN + 1))
    else
        echo -e "${YELLOW}⚠️  No benchmarks found${NC}"
    fi

    echo ""
fi

# ============================================================================
# Phase 3: gRPC Load Testing
# ============================================================================

if [ "$RUN_GRPC" = "true" ] && [ "$SERVER_RUNNING" = "true" ]; then
    echo -e "${MAGENTA}📋 Phase 3: gRPC Load Testing${NC}"
    echo ""

    TESTS_RUN=$((TESTS_RUN + 1))

    # Start resource monitoring in background
    if [ "$RUN_MONITORING" = "true" ]; then
        echo -e "${CYAN}Starting resource monitoring (background)...${NC}"
        DURATION=90 INTERVAL=2 "${LOAD_TEST_DIR}/monitor_resources.sh" \
            > "${REPORTS_DIR}/grpc_monitoring.log" 2>&1 &
        MONITOR_PID=$!
        echo -e "${BLUE}Monitor PID: ${MONITOR_PID}${NC}"
        sleep 2
    fi

    # Run gRPC load test
    if "${LOAD_TEST_DIR}/grpc_load_test.sh"; then
        echo -e "${GREEN}✅ gRPC load test passed${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}❌ gRPC load test failed${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi

    # Wait for monitoring to complete
    if [ "$RUN_MONITORING" = "true" ] && [ -n "$MONITOR_PID" ]; then
        echo -e "${CYAN}Waiting for monitoring to complete...${NC}"
        wait $MONITOR_PID 2>/dev/null || true
    fi

    echo ""
fi

# ============================================================================
# Phase 4: REST API Load Testing
# ============================================================================

if [ "$RUN_REST" = "true" ] && [ "$SERVER_RUNNING" = "true" ]; then
    echo -e "${MAGENTA}📋 Phase 4: REST API Load Testing${NC}"
    echo ""

    TESTS_RUN=$((TESTS_RUN + 1))

    # Start resource monitoring in background
    if [ "$RUN_MONITORING" = "true" ]; then
        echo -e "${CYAN}Starting resource monitoring (background)...${NC}"
        DURATION=90 INTERVAL=2 "${LOAD_TEST_DIR}/monitor_resources.sh" \
            > "${REPORTS_DIR}/rest_monitoring.log" 2>&1 &
        MONITOR_PID=$!
        echo -e "${BLUE}Monitor PID: ${MONITOR_PID}${NC}"
        sleep 2
    fi

    # Run REST load test
    if "${LOAD_TEST_DIR}/rest_load_test.sh"; then
        echo -e "${GREEN}✅ REST load test passed${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}❌ REST load test failed${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi

    # Wait for monitoring to complete
    if [ "$RUN_MONITORING" = "true" ] && [ -n "$MONITOR_PID" ]; then
        echo -e "${CYAN}Waiting for monitoring to complete...${NC}"
        wait $MONITOR_PID 2>/dev/null || true
    fi

    echo ""
fi

# ============================================================================
# Cleanup
# ============================================================================

if [ -n "$SERVER_PID" ]; then
    echo -e "${CYAN}Stopping background server (PID: ${SERVER_PID})...${NC}"
    kill $SERVER_PID 2>/dev/null || true
    sleep 2
    echo -e "${GREEN}✅ Server stopped${NC}"
    echo ""
fi

# ============================================================================
# Final Report
# ============================================================================

echo ""
echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Load Test Results Summary                ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
echo ""
echo -e "  Tests Run:     ${TESTS_RUN}"
echo -e "  ${GREEN}Tests Passed:  ${TESTS_PASSED}${NC}"
echo -e "  ${RED}Tests Failed:  ${TESTS_FAILED}${NC}"
echo ""

echo -e "${MAGENTA}📁 Reports Directory:${NC}"
echo -e "   ${REPORTS_DIR}"
echo ""

echo -e "${MAGENTA}📊 Generated Reports:${NC}"
ls -lh "${REPORTS_DIR}"/*.{json,txt,csv,log} 2>/dev/null | awk '{print "   " $NF " (" $5 ")"}'
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}╔════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║   ✅ All load tests passed!               ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${CYAN}Next Steps:${NC}"
    echo -e "  1. Review performance metrics in ${REPORTS_DIR}"
    echo -e "  2. Compare with SLO targets (500 RPS, p99 <200ms)"
    echo -e "  3. Check resource usage trends"
    echo -e "  4. Identify optimization opportunities"
    echo ""
    exit 0
else
    echo -e "${RED}╔════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║   ❌ Some load tests failed                ║${NC}"
    echo -e "${RED}╚════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${YELLOW}Debugging Tips:${NC}"
    echo -e "  1. Check detailed logs in ${REPORTS_DIR}"
    echo -e "  2. Review server logs: ${REPORTS_DIR}/server.log"
    echo -e "  3. Analyze resource usage: ${REPORTS_DIR}/*monitoring*.csv"
    echo -e "  4. Profile with flamegraph: cargo flamegraph --bin neoland"
    echo ""
    exit 1
fi
