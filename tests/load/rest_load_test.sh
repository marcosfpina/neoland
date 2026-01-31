#!/usr/bin/env bash
# NEOLAND REST API Load Testing with wrk
# Tests REST endpoint performance under load

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
REST_URL="${REST_URL:-http://localhost:3001}"
ENDPOINT="${ENDPOINT:-/v1/chat/completions}"
REPORTS_DIR="${REPORTS_DIR:-target/load-reports}"
DURATION="${DURATION:-60s}"
THREADS="${THREADS:-10}"
CONNECTIONS="${CONNECTIONS:-100}"

echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   NEOLAND REST API Load Testing (wrk)     ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Configuration:${NC}"
echo -e "  REST URL:      ${REST_URL}"
echo -e "  Endpoint:      ${ENDPOINT}"
echo -e "  Duration:      ${DURATION}"
echo -e "  Threads:       ${THREADS}"
echo -e "  Connections:   ${CONNECTIONS}"
echo -e "  Reports:       ${REPORTS_DIR}"
echo ""

# Create reports directory
mkdir -p "${REPORTS_DIR}"

# Check prerequisites
echo -e "${MAGENTA}📋 Checking prerequisites...${NC}"

if ! command -v wrk &> /dev/null; then
    echo -e "${YELLOW}⚠️  wrk not installed, trying Apache Bench (ab)${NC}"

    if ! command -v ab &> /dev/null; then
        echo -e "${RED}❌ Neither wrk nor ab installed${NC}"
        echo -e "${YELLOW}Install wrk: brew install wrk (macOS) or build from source${NC}"
        echo -e "${YELLOW}Or install ab: sudo apt install apache2-utils (Debian/Ubuntu)${NC}"
        exit 1
    else
        USE_AB=true
        echo -e "${GREEN}✅ Apache Bench (ab) available${NC}"
    fi
else
    USE_AB=false
    echo -e "${GREEN}✅ wrk installed${NC}"
fi

# Check if server is running
echo ""
echo -e "${MAGENTA}🔍 Checking if REST server is running...${NC}"
if curl -s "${REST_URL}/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ REST server is responding${NC}"
else
    echo -e "${YELLOW}⚠️  WARNING: REST server not responding at ${REST_URL}${NC}"
    echo -e "   Start server: cargo run --release --bin neoland -- server"
    read -p "   Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo ""
echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Running Load Tests                       ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
echo ""

# Create Lua script for wrk (POST with JSON)
cat > "${REPORTS_DIR}/post_request.lua" << 'EOF'
wrk.method = "POST"
wrk.body = '{"messages": [{"role": "user", "content": "What is Rust?"}]}'
wrk.headers["Content-Type"] = "application/json"
EOF

# Test 1: Warmup
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}Test 1: Warmup (10s, 10 connections)${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

if [ "$USE_AB" = true ]; then
    ab -n 100 -c 10 \
        -p <(echo '{"messages": [{"role": "user", "content": "warmup"}]}') \
        -T "application/json" \
        "${REST_URL}${ENDPOINT}" \
        > "${REPORTS_DIR}/rest_warmup_ab.txt" 2>&1
else
    wrk -t2 -c10 -d10s \
        -s "${REPORTS_DIR}/post_request.lua" \
        "${REST_URL}${ENDPOINT}" \
        > "${REPORTS_DIR}/rest_warmup.txt"
fi

echo -e "${GREEN}✅ Warmup complete${NC}"
echo ""

# Test 2: Baseline (50 connections, 30s)
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}Test 2: Baseline (50 connections, 30s)${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

if [ "$USE_AB" = true ]; then
    ab -n 10000 -c 50 \
        -p <(echo '{"messages": [{"role": "user", "content": "What is Rust?"}]}') \
        -T "application/json" \
        "${REST_URL}${ENDPOINT}" \
        > "${REPORTS_DIR}/rest_baseline_ab.txt" 2>&1

    # Parse ab results
    BASELINE_RPS=$(grep "Requests per second:" "${REPORTS_DIR}/rest_baseline_ab.txt" | awk '{print $4}')
    BASELINE_AVG=$(grep "Time per request:" "${REPORTS_DIR}/rest_baseline_ab.txt" | head -1 | awk '{print $4}')
    BASELINE_P50=$(grep "50%" "${REPORTS_DIR}/rest_baseline_ab.txt" | awk '{print $2}')
    BASELINE_P95=$(grep "95%" "${REPORTS_DIR}/rest_baseline_ab.txt" | awk '{print $2}')
    BASELINE_P99=$(grep "99%" "${REPORTS_DIR}/rest_baseline_ab.txt" | awk '{print $2}')
else
    wrk -t5 -c50 -d30s \
        -s "${REPORTS_DIR}/post_request.lua" \
        "${REST_URL}${ENDPOINT}" \
        > "${REPORTS_DIR}/rest_baseline.txt"

    # Parse wrk results
    BASELINE_RPS=$(grep "Requests/sec:" "${REPORTS_DIR}/rest_baseline.txt" | awk '{print $2}')
    BASELINE_AVG=$(grep "Latency" "${REPORTS_DIR}/rest_baseline.txt" | awk '{print $2}')
    BASELINE_P50=$(grep "50%" "${REPORTS_DIR}/rest_baseline.txt" | awk '{print $2}')
    BASELINE_P99=$(grep "99%" "${REPORTS_DIR}/rest_baseline.txt" | awk '{print $2}')
fi

echo ""
echo -e "${MAGENTA}📊 Baseline Results (50 connections):${NC}"
echo -e "   Requests/sec:     ${BASELINE_RPS:-N/A}"
echo -e "   Average latency:  ${BASELINE_AVG:-N/A}"
echo -e "   p50 latency:      ${BASELINE_P50:-N/A}"
echo -e "   p99 latency:      ${BASELINE_P99:-N/A}"
echo ""

# Test 3: Target Load (100 connections, 60s)
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}Test 3: Target Load (${CONNECTIONS} connections, ${DURATION})${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

if [ "$USE_AB" = true ]; then
    ab -n 30000 -c "${CONNECTIONS}" \
        -p <(echo '{"messages": [{"role": "user", "content": "Explain the Rust borrow checker"}]}') \
        -T "application/json" \
        "${REST_URL}${ENDPOINT}" \
        > "${REPORTS_DIR}/rest_target_ab.txt" 2>&1

    TARGET_RPS=$(grep "Requests per second:" "${REPORTS_DIR}/rest_target_ab.txt" | awk '{print $4}')
    TARGET_AVG=$(grep "Time per request:" "${REPORTS_DIR}/rest_target_ab.txt" | head -1 | awk '{print $4}')
    TARGET_P99=$(grep "99%" "${REPORTS_DIR}/rest_target_ab.txt" | awk '{print $2}')
else
    wrk -t"${THREADS}" -c"${CONNECTIONS}" -d"${DURATION}" \
        -s "${REPORTS_DIR}/post_request.lua" \
        "${REST_URL}${ENDPOINT}" \
        > "${REPORTS_DIR}/rest_target.txt"

    TARGET_RPS=$(grep "Requests/sec:" "${REPORTS_DIR}/rest_target.txt" | awk '{print $2}')
    TARGET_AVG=$(grep "Latency" "${REPORTS_DIR}/rest_target.txt" | awk '{print $2}')
    TARGET_P99=$(grep "99%" "${REPORTS_DIR}/rest_target.txt" | awk '{print $2}')
fi

echo ""
echo -e "${MAGENTA}📊 Target Load Results (${CONNECTIONS} connections):${NC}"
echo -e "   Requests/sec:     ${TARGET_RPS:-N/A}"
echo -e "   Average latency:  ${TARGET_AVG:-N/A}"
echo -e "   p99 latency:      ${TARGET_P99:-N/A}"
echo ""

# Test 4: Stress Test (200 connections)
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}Test 4: Stress Test (200 connections, 30s)${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

if [ "$USE_AB" = true ]; then
    ab -n 20000 -c 200 \
        -p <(echo '{"messages": [{"role": "user", "content": "stress test"}]}') \
        -T "application/json" \
        "${REST_URL}${ENDPOINT}" \
        > "${REPORTS_DIR}/rest_stress_ab.txt" 2>&1

    STRESS_RPS=$(grep "Requests per second:" "${REPORTS_DIR}/rest_stress_ab.txt" | awk '{print $4}')
    STRESS_P99=$(grep "99%" "${REPORTS_DIR}/rest_stress_ab.txt" | awk '{print $2}')
else
    wrk -t10 -c200 -d30s \
        -s "${REPORTS_DIR}/post_request.lua" \
        "${REST_URL}${ENDPOINT}" \
        > "${REPORTS_DIR}/rest_stress.txt"

    STRESS_RPS=$(grep "Requests/sec:" "${REPORTS_DIR}/rest_stress.txt" | awk '{print $2}')
    STRESS_P99=$(grep "99%" "${REPORTS_DIR}/rest_stress.txt" | awk '{print $2}')
fi

echo ""
echo -e "${MAGENTA}📊 Stress Test Results (200 connections):${NC}"
echo -e "   Requests/sec:     ${STRESS_RPS:-N/A}"
echo -e "   p99 latency:      ${STRESS_P99:-N/A}"
echo ""

# Test 5: Health endpoint (lightweight)
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}Test 5: Health Endpoint (1000 connections, 10s)${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

if [ "$USE_AB" = true ]; then
    ab -n 50000 -c 1000 \
        "${REST_URL}/health" \
        > "${REPORTS_DIR}/rest_health_ab.txt" 2>&1

    HEALTH_RPS=$(grep "Requests per second:" "${REPORTS_DIR}/rest_health_ab.txt" | awk '{print $4}')
else
    wrk -t10 -c1000 -d10s \
        "${REST_URL}/health" \
        > "${REPORTS_DIR}/rest_health.txt"

    HEALTH_RPS=$(grep "Requests/sec:" "${REPORTS_DIR}/rest_health.txt" | awk '{print $2}')
fi

echo ""
echo -e "${MAGENTA}📊 Health Endpoint Results:${NC}"
echo -e "   Requests/sec:     ${HEALTH_RPS:-N/A}"
echo ""

# Final Report
echo ""
echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Load Test Summary                        ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
echo ""

echo -e "${MAGENTA}📋 Performance Targets:${NC}"
echo -e "   • Target: 500 RPS sustained"
echo -e "   • p99 latency: < 200ms"
echo ""

# Check if target met (simplified check)
if [ -n "${TARGET_RPS}" ]; then
    TARGET_RPS_NUM=$(echo "${TARGET_RPS}" | sed 's/[^0-9.]//g')

    if (( $(echo "${TARGET_RPS_NUM} >= 500" | bc -l 2>/dev/null || echo 0) )); then
        echo -e "${GREEN}✅ RPS target met: ${TARGET_RPS} >= 500${NC}"
    else
        echo -e "${YELLOW}⚠️  RPS below target: ${TARGET_RPS} < 500${NC}"
    fi
fi

echo ""
echo -e "${MAGENTA}📁 Reports saved to:${NC}"
if [ "$USE_AB" = true ]; then
    echo -e "   ${REPORTS_DIR}/rest_warmup_ab.txt"
    echo -e "   ${REPORTS_DIR}/rest_baseline_ab.txt"
    echo -e "   ${REPORTS_DIR}/rest_target_ab.txt"
    echo -e "   ${REPORTS_DIR}/rest_stress_ab.txt"
    echo -e "   ${REPORTS_DIR}/rest_health_ab.txt"
else
    echo -e "   ${REPORTS_DIR}/rest_warmup.txt"
    echo -e "   ${REPORTS_DIR}/rest_baseline.txt"
    echo -e "   ${REPORTS_DIR}/rest_target.txt"
    echo -e "   ${REPORTS_DIR}/rest_stress.txt"
    echo -e "   ${REPORTS_DIR}/rest_health.txt"
fi
echo ""

echo -e "${GREEN}╔════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║   ✅ REST API load testing complete       ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════╝${NC}"
