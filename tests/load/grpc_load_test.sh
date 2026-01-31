#!/usr/bin/env bash
# NEOLAND gRPC Load Testing with ghz
# Tests gRPC endpoint performance under load

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
GRPC_URL="${GRPC_URL:-localhost:50051}"
PROTO_FILE="${PROTO_FILE:-proto/llamachat.proto}"
REPORTS_DIR="${REPORTS_DIR:-target/load-reports}"
DURATION="${DURATION:-60s}"
CONNECTIONS="${CONNECTIONS:-100}"
RPS="${RPS:-500}"
WORKERS="${WORKERS:-10}"

echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   NEOLAND gRPC Load Testing (ghz)         ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Configuration:${NC}"
echo -e "  gRPC URL:      ${GRPC_URL}"
echo -e "  Proto:         ${PROTO_FILE}"
echo -e "  Duration:      ${DURATION}"
echo -e "  Connections:   ${CONNECTIONS}"
echo -e "  Target RPS:    ${RPS}"
echo -e "  Workers:       ${WORKERS}"
echo -e "  Reports:       ${REPORTS_DIR}"
echo ""

# Create reports directory
mkdir -p "${REPORTS_DIR}"

# Check prerequisites
echo -e "${MAGENTA}📋 Checking prerequisites...${NC}"

if ! command -v ghz &> /dev/null; then
    echo -e "${RED}❌ ghz not installed${NC}"
    echo -e "${YELLOW}Install: go install github.com/bojand/ghz/cmd/ghz@latest${NC}"
    echo -e "${YELLOW}Or: brew install ghz (macOS)${NC}"
    exit 1
fi
echo -e "${GREEN}✅ ghz installed${NC}"

if [ ! -f "${PROTO_FILE}" ]; then
    echo -e "${RED}❌ Proto file not found: ${PROTO_FILE}${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Proto file found${NC}"

# Check if server is running
echo ""
echo -e "${MAGENTA}🔍 Checking if gRPC server is running...${NC}"
if grpcurl -plaintext "${GRPC_URL}" list > /dev/null 2>&1; then
    echo -e "${GREEN}✅ gRPC server is responding${NC}"
else
    echo -e "${YELLOW}⚠️  WARNING: gRPC server not responding at ${GRPC_URL}${NC}"
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

# Test 1: Warmup (low load)
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}Test 1: Warmup (10 RPS, 10s)${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

ghz --insecure \
    --proto "${PROTO_FILE}" \
    --call llamachat.LlamaService/ChatStream \
    -d '{"prompt": "Hello, this is a warmup request"}' \
    -n 100 \
    -c 10 \
    --rps 10 \
    --format json \
    --output "${REPORTS_DIR}/grpc_warmup.json" \
    "${GRPC_URL}"

echo -e "${GREEN}✅ Warmup complete${NC}"
echo ""

# Test 2: Baseline (100 RPS)
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}Test 2: Baseline (100 RPS, 30s)${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

ghz --insecure \
    --proto "${PROTO_FILE}" \
    --call llamachat.LlamaService/ChatStream \
    -d '{"prompt": "What is Rust?"}' \
    -z 30s \
    -c 50 \
    --rps 100 \
    --format json \
    --output "${REPORTS_DIR}/grpc_baseline_100rps.json" \
    "${GRPC_URL}"

# Parse results
BASELINE_P50=$(jq -r '.latencyDistribution."50"' "${REPORTS_DIR}/grpc_baseline_100rps.json" | sed 's/ms//')
BASELINE_P95=$(jq -r '.latencyDistribution."95"' "${REPORTS_DIR}/grpc_baseline_100rps.json" | sed 's/ms//')
BASELINE_P99=$(jq -r '.latencyDistribution."99"' "${REPORTS_DIR}/grpc_baseline_100rps.json" | sed 's/ms//')
BASELINE_AVG=$(jq -r '.average' "${REPORTS_DIR}/grpc_baseline_100rps.json" | sed 's/ms//')
BASELINE_RPS=$(jq -r '.rps' "${REPORTS_DIR}/grpc_baseline_100rps.json")

echo ""
echo -e "${MAGENTA}📊 Baseline Results (100 RPS):${NC}"
echo -e "   Average latency:  ${BASELINE_AVG}ms"
echo -e "   p50 latency:      ${BASELINE_P50}ms"
echo -e "   p95 latency:      ${BASELINE_P95}ms"
echo -e "   p99 latency:      ${BASELINE_P99}ms"
echo -e "   Actual RPS:       ${BASELINE_RPS}"
echo ""

# Test 3: Target Load (500 RPS)
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}Test 3: Target Load (${RPS} RPS, ${DURATION})${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

ghz --insecure \
    --proto "${PROTO_FILE}" \
    --call llamachat.LlamaService/ChatStream \
    -d '{"prompt": "Explain the Rust borrow checker"}' \
    -z "${DURATION}" \
    -c "${CONNECTIONS}" \
    --rps "${RPS}" \
    --format json \
    --output "${REPORTS_DIR}/grpc_target_${RPS}rps.json" \
    "${GRPC_URL}"

# Parse results
TARGET_P50=$(jq -r '.latencyDistribution."50"' "${REPORTS_DIR}/grpc_target_${RPS}rps.json" | sed 's/ms//')
TARGET_P95=$(jq -r '.latencyDistribution."95"' "${REPORTS_DIR}/grpc_target_${RPS}rps.json" | sed 's/ms//')
TARGET_P99=$(jq -r '.latencyDistribution."99"' "${REPORTS_DIR}/grpc_target_${RPS}rps.json" | sed 's/ms//')
TARGET_AVG=$(jq -r '.average' "${REPORTS_DIR}/grpc_target_${RPS}rps.json" | sed 's/ms//')
TARGET_RPS=$(jq -r '.rps' "${REPORTS_DIR}/grpc_target_${RPS}rps.json")
TARGET_ERRORS=$(jq -r '.statusCodeDistribution.OK // 0' "${REPORTS_DIR}/grpc_target_${RPS}rps.json")

echo ""
echo -e "${MAGENTA}📊 Target Load Results (${RPS} RPS):${NC}"
echo -e "   Average latency:  ${TARGET_AVG}ms"
echo -e "   p50 latency:      ${TARGET_P50}ms"
echo -e "   p95 latency:      ${TARGET_P95}ms"
echo -e "   p99 latency:      ${TARGET_P99}ms"
echo -e "   Actual RPS:       ${TARGET_RPS}"
echo -e "   Successful:       ${TARGET_ERRORS}"
echo ""

# Test 4: Stress Test (800 RPS)
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}Test 4: Stress Test (800 RPS, 30s)${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

ghz --insecure \
    --proto "${PROTO_FILE}" \
    --call llamachat.LlamaService/ChatStream \
    -d '{"prompt": "What are the best practices for async Rust?"}' \
    -z 30s \
    -c 200 \
    --rps 800 \
    --format json \
    --output "${REPORTS_DIR}/grpc_stress_800rps.json" \
    "${GRPC_URL}"

# Parse results
STRESS_P50=$(jq -r '.latencyDistribution."50"' "${REPORTS_DIR}/grpc_stress_800rps.json" | sed 's/ms//')
STRESS_P95=$(jq -r '.latencyDistribution."95"' "${REPORTS_DIR}/grpc_stress_800rps.json" | sed 's/ms//')
STRESS_P99=$(jq -r '.latencyDistribution."99"' "${REPORTS_DIR}/grpc_stress_800rps.json" | sed 's/ms//')
STRESS_AVG=$(jq -r '.average' "${REPORTS_DIR}/grpc_stress_800rps.json" | sed 's/ms//')
STRESS_RPS=$(jq -r '.rps' "${REPORTS_DIR}/grpc_stress_800rps.json")

echo ""
echo -e "${MAGENTA}📊 Stress Test Results (800 RPS):${NC}"
echo -e "   Average latency:  ${STRESS_AVG}ms"
echo -e "   p50 latency:      ${STRESS_P50}ms"
echo -e "   p95 latency:      ${STRESS_P95}ms"
echo -e "   p99 latency:      ${STRESS_P99}ms"
echo -e "   Actual RPS:       ${STRESS_RPS}"
echo ""

# Test 5: Spike Test (sudden load increase)
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}Test 5: Spike Test (1000 RPS, 10s)${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

ghz --insecure \
    --proto "${PROTO_FILE}" \
    --call llamachat.LlamaService/ChatStream \
    -d '{"prompt": "Quick spike test"}' \
    -z 10s \
    -c 300 \
    --rps 1000 \
    --format json \
    --output "${REPORTS_DIR}/grpc_spike_1000rps.json" \
    "${GRPC_URL}"

# Parse results
SPIKE_P99=$(jq -r '.latencyDistribution."99"' "${REPORTS_DIR}/grpc_spike_1000rps.json" | sed 's/ms//')
SPIKE_RPS=$(jq -r '.rps' "${REPORTS_DIR}/grpc_spike_1000rps.json")

echo ""
echo -e "${MAGENTA}📊 Spike Test Results (1000 RPS):${NC}"
echo -e "   p99 latency:      ${SPIKE_P99}ms"
echo -e "   Actual RPS:       ${SPIKE_RPS}"
echo ""

# Final Report
echo ""
echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Load Test Summary                        ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
echo ""

# Check SLOs
SLO_P99_MS=200
PASS_COUNT=0
FAIL_COUNT=0

echo -e "${MAGENTA}📋 SLO Validation (p99 < ${SLO_P99_MS}ms):${NC}"
echo ""

# Baseline check
if (( $(echo "${BASELINE_P99} < ${SLO_P99_MS}" | bc -l) )); then
    echo -e "${GREEN}✅ Baseline (100 RPS): ${BASELINE_P99}ms < ${SLO_P99_MS}ms${NC}"
    PASS_COUNT=$((PASS_COUNT + 1))
else
    echo -e "${RED}❌ Baseline (100 RPS): ${BASELINE_P99}ms >= ${SLO_P99_MS}ms${NC}"
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

# Target check
if (( $(echo "${TARGET_P99} < ${SLO_P99_MS}" | bc -l) )); then
    echo -e "${GREEN}✅ Target (${RPS} RPS): ${TARGET_P99}ms < ${SLO_P99_MS}ms${NC}"
    PASS_COUNT=$((PASS_COUNT + 1))
else
    echo -e "${RED}❌ Target (${RPS} RPS): ${TARGET_P99}ms >= ${SLO_P99_MS}ms${NC}"
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

# Stress check
if (( $(echo "${STRESS_P99} < ${SLO_P99_MS}" | bc -l) )); then
    echo -e "${GREEN}✅ Stress (800 RPS): ${STRESS_P99}ms < ${SLO_P99_MS}ms${NC}"
    PASS_COUNT=$((PASS_COUNT + 1))
else
    echo -e "${YELLOW}⚠️  Stress (800 RPS): ${STRESS_P99}ms >= ${SLO_P99_MS}ms (acceptable degradation)${NC}"
    # Don't count as fail for stress test
fi

echo ""
echo -e "${MAGENTA}📁 Reports saved to:${NC}"
echo -e "   ${REPORTS_DIR}/grpc_warmup.json"
echo -e "   ${REPORTS_DIR}/grpc_baseline_100rps.json"
echo -e "   ${REPORTS_DIR}/grpc_target_${RPS}rps.json"
echo -e "   ${REPORTS_DIR}/grpc_stress_800rps.json"
echo -e "   ${REPORTS_DIR}/grpc_spike_1000rps.json"
echo ""

if [ $FAIL_COUNT -eq 0 ]; then
    echo -e "${GREEN}╔════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║   ✅ All SLO checks passed!               ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════╝${NC}"
    exit 0
else
    echo -e "${RED}╔════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║   ❌ Some SLO checks failed                ║${NC}"
    echo -e "${RED}╚════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${YELLOW}Next Steps:${NC}"
    echo -e "  1. Analyze latency bottlenecks"
    echo -e "  2. Profile with: cargo flamegraph --bin neoland"
    echo -e "  3. Check resource usage: htop, ps aux"
    echo -e "  4. Review Prometheus metrics"
    echo ""
    exit 1
fi
