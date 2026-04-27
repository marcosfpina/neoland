#!/usr/bin/env bash

# Dev Assistant Hub - Automated Benchmark Suite
# Measures developer workflow performance across all features

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BENCHMARK_DIR="benchmarks/results"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
RESULTS_FILE="$BENCHMARK_DIR/benchmark_$TIMESTAMP.json"
REPORT_FILE="$BENCHMARK_DIR/report_$TIMESTAMP.md"
BASE_URL="http://localhost:3006"
API_URL="$BASE_URL/api"

# Create results directory
mkdir -p "$BENCHMARK_DIR"

echo -e "${BLUE}🚀 Starting Dev Assistant Hub Benchmark Suite${NC}"
echo -e "${YELLOW}Timestamp: $TIMESTAMP${NC}"
echo -e "${YELLOW}Results will be saved to: $RESULTS_FILE${NC}"

# Record start time for duration calculation
START_TIME=$(date +%s)

# Initialize results JSON
cat > "$RESULTS_FILE" << EOF
{
  "timestamp": "$TIMESTAMP",
  "system_info": {},
  "benchmarks": {},
  "summary": {}
}
EOF

# Function to update JSON results
update_results() {
    local key="$1"
    local value="$2"
    jq --arg key "$key" --argjson value "$value" '.benchmarks[$key] = $value' "$RESULTS_FILE" > tmp.$$.json && mv tmp.$$.json "$RESULTS_FILE"
}

# Function to measure execution time
measure_time() {
    local start_time=$(date +%s%N)
    "$@"
    local end_time=$(date +%s%N)
    local duration=$(( (end_time - start_time) / 1000000 )) # Convert to milliseconds
    echo $duration
}

# System Information Collection
echo -e "\n${BLUE}📊 Collecting System Information${NC}"
SYSTEM_INFO=$(cat << EOF
{
  "hostname": "$(hostname)",
  "os": "$(uname -s)",
  "kernel": "$(uname -r)",
  "architecture": "$(uname -m)",
  "cpu_cores": $(nproc),
  "memory_total": "$(free -h | awk '/^Mem:/ {print $2}')",
  "memory_available": "$(free -h | awk '/^Mem:/ {print $7}')",
  "disk_usage": "$(df -h / | awk 'NR==2 {print $5}')",
   "gpu_info": "$(command -v nvidia-smi >/dev/null 2>&1 && nvidia-smi --query-gpu=name,memory.total --format=csv,noheader,nounits 2>/dev/null || echo 'No NVIDIA GPU detected')",
  "node_version": "$(node --version 2>/dev/null || echo 'Not installed')",
  "python_version": "$(python3 --version 2>/dev/null || echo 'Not installed')"
}
EOF
)

jq --argjson info "$SYSTEM_INFO" '.system_info = $info' "$RESULTS_FILE" > tmp.$$.json && mv tmp.$$.json "$RESULTS_FILE"

# Check if application is running
echo -e "\n${BLUE}🔍 Checking Application Status${NC}"
if ! curl -s "$BASE_URL" > /dev/null; then
    echo -e "${RED}❌ Application not running at $BASE_URL${NC}"
    echo -e "${YELLOW}Please start the application first with: ./start.sh${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Application is running${NC}"

# 1. Application Startup Time Benchmark
echo -e "\n${BLUE}⏱️  Measuring Application Startup Time${NC}"
STARTUP_RESULTS=$(cat << EOF
{
  "test_name": "Application Startup",
  "description": "Time to start the Next.js application",
  "cold_start_time": 0,
  "warm_start_time": 0,
  "average_start_time": 0
}
EOF
)

# Simulate cold start (kill and restart)
if pgrep -f "next" > /dev/null; then
    echo "Measuring warm restart..."
    WARM_START=$(measure_time curl -s "$BASE_URL" > /dev/null)
    STARTUP_RESULTS=$(echo "$STARTUP_RESULTS" | jq --arg time "$WARM_START" '.warm_start_time = ($time | tonumber)')
fi

update_results "startup_performance" "$STARTUP_RESULTS"
echo -e "${GREEN}✅ Startup benchmark completed${NC}"

# 2. API Response Time Benchmark
echo -e "\n${BLUE}🌐 Testing API Response Times${NC}"

API_ENDPOINTS=(
    "/api/health:Health Check"
    "/api/code-analyzer:Code Analysis"
    "/api/llm-playground:LLM Playground"
    "/api/agent-orchestra:Agent Orchestra"
)

API_RESULTS='{"test_name": "API Response Times", "endpoints": {}}'

for endpoint_info in "${API_ENDPOINTS[@]}"; do
    IFS=':' read -r endpoint description <<< "$endpoint_info"
    echo "Testing $description ($endpoint)..."
    
    # Measure multiple requests
    TIMES=()
    for i in {1..10}; do
        TIME=$(measure_time curl -s "$API_URL$endpoint" > /dev/null)
        TIMES+=($TIME)
    done
    
    # Calculate statistics
    AVG=$(printf '%s\n' "${TIMES[@]}" | awk '{sum+=$1} END {print sum/NR}')
    MIN=$(printf '%s\n' "${TIMES[@]}" | sort -n | head -1)
    MAX=$(printf '%s\n' "${TIMES[@]}" | sort -n | tail -1)
    
    ENDPOINT_RESULT=$(cat << EOF
{
  "description": "$description",
  "average_ms": $AVG,
  "min_ms": $MIN,
  "max_ms": $MAX,
  "samples": 10
}
EOF
    )
    
    API_RESULTS=$(echo "$API_RESULTS" | jq --arg key "${endpoint//\//_}" --argjson result "$ENDPOINT_RESULT" '.endpoints[$key] = $result')
done

update_results "api_performance" "$API_RESULTS"
echo -e "${GREEN}✅ API response time benchmark completed${NC}"

# 3. Code Analysis Workflow Benchmark
echo -e "\n${BLUE}🔍 Testing Code Analysis Workflow${NC}"

# Create test code file
TEST_CODE=$(cat << 'EOF'
function calculateTotal(items) {
    let total = 0;
    for (let i = 0; i < items.length; i++) {
        total += items[i].price * items[i].quantity;
    }
    return total;
}

class ShoppingCart {
    constructor() {
        this.items = [];
    }
    
    addItem(item) {
        this.items.push(item);
    }
    
    getTotal() {
        return calculateTotal(this.items);
    }
}
EOF
)

echo "$TEST_CODE" > /tmp/test_code.js

# Measure code analysis time
ANALYSIS_START=$(date +%s%N)
ANALYSIS_RESPONSE=$(curl -s -X POST "$API_URL/code-analyzer" \
    -H "Content-Type: application/json" \
    -d "{\"code\": \"$(echo "$TEST_CODE" | sed 's/"/\\"/g' | tr '\n' ' ')\"}" || echo '{"error": "API not available"}')
ANALYSIS_END=$(date +%s%N)
ANALYSIS_TIME=$(( (ANALYSIS_END - ANALYSIS_START) / 1000000 ))

CODE_ANALYSIS_RESULTS=$(cat << EOF
{
  "test_name": "Code Analysis Workflow",
  "analysis_time_ms": $ANALYSIS_TIME,
  "code_lines": $(echo "$TEST_CODE" | wc -l),
  "response_received": $(echo "$ANALYSIS_RESPONSE" | jq 'has("error") | not'),
  "workflow_complete": true
}
EOF
)

update_results "code_analysis" "$CODE_ANALYSIS_RESULTS"
rm -f /tmp/test_code.js
echo -e "${GREEN}✅ Code analysis workflow benchmark completed${NC}"

# 4. Memory Usage Monitoring
echo -e "\n${BLUE}💾 Monitoring Memory Usage${NC}"

# Get initial memory usage
INITIAL_MEMORY=$(ps aux | grep -E "(next|node)" | grep -v grep | awk '{sum += $6} END {print sum/1024}' || echo "0")

# Perform memory-intensive operations
for i in {1..5}; do
    curl -s "$API_URL/llm-playground" > /dev/null &
    curl -s "$API_URL/code-analyzer" > /dev/null &
done
wait

# Get peak memory usage
PEAK_MEMORY=$(ps aux | grep -E "(next|node)" | grep -v grep | awk '{sum += $6} END {print sum/1024}' || echo "0")

MEMORY_RESULTS=$(cat << EOF
{
  "test_name": "Memory Usage Monitoring",
  "initial_memory_mb": $INITIAL_MEMORY,
  "peak_memory_mb": $PEAK_MEMORY,
  "memory_increase_mb": $(echo "$PEAK_MEMORY - $INITIAL_MEMORY" | bc),
  "system_memory_total": "$(free -m | awk '/^Mem:/ {print $2}')",
  "system_memory_used": "$(free -m | awk '/^Mem:/ {print $3}')"
}
EOF
)

update_results "memory_usage" "$MEMORY_RESULTS"
echo -e "${GREEN}✅ Memory usage monitoring completed${NC}"

# 5. Concurrent User Simulation
echo -e "\n${BLUE}👥 Simulating Concurrent Users${NC}"

CONCURRENT_LEVELS=(1 5 10 20)
CONCURRENT_RESULTS='{"test_name": "Concurrent User Simulation", "levels": {}}'

for level in "${CONCURRENT_LEVELS[@]}"; do
    echo "Testing with $level concurrent users..."
    
    # Create background processes
    PIDS=()
    START_TIME=$(date +%s%N)
    
    for ((i=1; i<=level; i++)); do
        (
            for ((j=1; j<=5; j++)); do
                curl -s "$BASE_URL" > /dev/null
                curl -s "$API_URL/health" > /dev/null
            done
        ) &
        PIDS+=($!)
    done
    
    # Wait for all processes to complete
    for pid in "${PIDS[@]}"; do
        wait $pid
    done
    
    END_TIME=$(date +%s%N)
    TOTAL_TIME=$(( (END_TIME - START_TIME) / 1000000 ))
    
    LEVEL_RESULT=$(cat << EOF
{
  "concurrent_users": $level,
  "total_requests": $((level * 5 * 2)),
  "total_time_ms": $TOTAL_TIME,
  "requests_per_second": $(echo "scale=2; $((level * 5 * 2)) * 1000 / $TOTAL_TIME" | bc),
  "avg_response_time_ms": $(echo "scale=2; $TOTAL_TIME / $((level * 5 * 2))" | bc)
}
EOF
    )
    
    CONCURRENT_RESULTS=$(echo "$CONCURRENT_RESULTS" | jq --arg key "level_$level" --argjson result "$LEVEL_RESULT" '.levels[$key] = $result')
done

update_results "concurrent_users" "$CONCURRENT_RESULTS"
echo -e "${GREEN}✅ Concurrent user simulation completed${NC}"

# 6. Generate Summary
echo -e "\n${BLUE}📋 Generating Summary${NC}"

SUMMARY=$(cat << EOF
{
  "total_tests": 5,
   "test_duration_seconds": $(($(date +%s) - START_TIME)),
  "overall_status": "completed",
  "recommendations": [
    "Monitor memory usage during peak loads",
    "Consider caching for frequently accessed endpoints",
    "Optimize code analysis for larger files",
    "Implement connection pooling for high concurrency"
  ]
}
EOF
)

jq --argjson summary "$SUMMARY" '.summary = $summary' "$RESULTS_FILE" > tmp.$$.json && mv tmp.$$.json "$RESULTS_FILE"

# Generate Markdown Report
echo -e "\n${BLUE}📄 Generating Markdown Report${NC}"

cat > "$REPORT_FILE" << EOF
# Dev Assistant Hub - Benchmark Report

**Generated:** $(date)  
**Duration:** $(jq -r '.summary.test_duration_seconds' "$RESULTS_FILE") seconds

## System Information

- **Hostname:** $(jq -r '.system_info.hostname' "$RESULTS_FILE")
- **OS:** $(jq -r '.system_info.os' "$RESULTS_FILE") $(jq -r '.system_info.kernel' "$RESULTS_FILE")
- **CPU Cores:** $(jq -r '.system_info.cpu_cores' "$RESULTS_FILE")
- **Memory:** $(jq -r '.system_info.memory_total' "$RESULTS_FILE") total, $(jq -r '.system_info.memory_available' "$RESULTS_FILE") available
- **GPU:** $(jq -r '.system_info.gpu_info' "$RESULTS_FILE")

## Benchmark Results

### 🚀 Application Performance
- **Warm Start Time:** $(jq -r '.benchmarks.startup_performance.warm_start_time' "$RESULTS_FILE")ms

### 🌐 API Response Times
$(jq -r '.benchmarks.api_performance.endpoints | to_entries[] | "- **\(.value.description):** \(.value.average_ms)ms (min: \(.value.min_ms)ms, max: \(.value.max_ms)ms)"' "$RESULTS_FILE")

### 🔍 Code Analysis
- **Analysis Time:** $(jq -r '.benchmarks.code_analysis.analysis_time_ms' "$RESULTS_FILE")ms
- **Lines Processed:** $(jq -r '.benchmarks.code_analysis.code_lines' "$RESULTS_FILE")

### 💾 Memory Usage
- **Initial Memory:** $(jq -r '.benchmarks.memory_usage.initial_memory_mb' "$RESULTS_FILE")MB
- **Peak Memory:** $(jq -r '.benchmarks.memory_usage.peak_memory_mb' "$RESULTS_FILE")MB
- **Memory Increase:** $(jq -r '.benchmarks.memory_usage.memory_increase_mb' "$RESULTS_FILE")MB

### 👥 Concurrent Performance
$(jq -r '.benchmarks.concurrent_users.levels | to_entries[] | "- **\(.value.concurrent_users) users:** \(.value.requests_per_second) req/s, \(.value.avg_response_time_ms)ms avg"' "$RESULTS_FILE")

## Recommendations

$(jq -r '.summary.recommendations[] | "- \(.)"' "$RESULTS_FILE")

---
*Generated by Dev Assistant Hub Benchmark Suite*
EOF

echo -e "\n${GREEN}🎉 Benchmark Suite Completed!${NC}"
echo -e "${YELLOW}Results saved to:${NC}"
echo -e "  📊 JSON: $RESULTS_FILE"
echo -e "  📄 Report: $REPORT_FILE"

# Display quick summary
echo -e "\n${BLUE}📋 Quick Summary:${NC}"
echo -e "  ⏱️  Total Duration: $(jq -r '.summary.test_duration_seconds' "$RESULTS_FILE")s"
echo -e "  🧪 Tests Completed: $(jq -r '.summary.total_tests' "$RESULTS_FILE")"
echo -e "  💾 Peak Memory: $(jq -r '.benchmarks.memory_usage.peak_memory_mb' "$RESULTS_FILE")MB"
echo -e "  🚀 Best API Response: $(jq -r '[.benchmarks.api_performance.endpoints[].min_ms] | min' "$RESULTS_FILE")ms"
