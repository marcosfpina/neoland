#!/bin/bash

# Comprehensive Benchmark Suite Runner
# Executes all benchmark types and generates unified reports

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
RESULTS_DIR="benchmarks/results"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
UNIFIED_REPORT="$RESULTS_DIR/comprehensive-report-$TIMESTAMP.md"
UNIFIED_JSON="$RESULTS_DIR/unified-report-$TIMESTAMP.json"
BASE_URL="http://localhost:3006"

# Create results directory
mkdir -p "$RESULTS_DIR"

echo -e "${PURPLE}🚀 Dev Assistant Hub - Comprehensive Benchmark Suite${NC}"
echo -e "${BLUE}================================================================${NC}"
echo -e "${YELLOW}Timestamp: $TIMESTAMP${NC}"
echo -e "${YELLOW}Results Directory: $RESULTS_DIR${NC}"
echo -e "${YELLOW}Target URL: $BASE_URL${NC}"
echo -e "${BLUE}================================================================${NC}"

# Check if application is running
echo -e "\n${BLUE}🔍 Checking Application Status${NC}"
if ! curl -s "$BASE_URL" > /dev/null; then
    echo -e "${RED}❌ Application not running at $BASE_URL${NC}"
    echo -e "${YELLOW}Please start the application first with: ./start.sh${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Application is running and accessible${NC}"

# Initialize unified results
cat > "$UNIFIED_JSON" << EOF
{
  "timestamp": "$TIMESTAMP",
  "base_url": "$BASE_URL",
  "benchmarks": {},
  "summary": {
    "total_duration": 0,
    "tests_completed": 0,
    "tests_failed": 0
  }
}
EOF

SUITE_START_TIME=$(date +%s)

# Function to update unified results
update_unified_results() {
    local test_name="$1"
    local result_file="$2"
    
    if [ -f "$result_file" ]; then
        jq --arg name "$test_name" --slurpfile result "$result_file" '.benchmarks[$name] = $result[0]' "$UNIFIED_JSON" > tmp.$$.json && mv tmp.$$.json "$UNIFIED_JSON"
        jq '.summary.tests_completed += 1' "$UNIFIED_JSON" > tmp.$$.json && mv tmp.$$.json "$UNIFIED_JSON"
        echo -e "${GREEN}✅ $test_name results integrated${NC}"
    else
        echo -e "${RED}❌ $test_name results not found${NC}"
        jq '.summary.tests_failed += 1' "$UNIFIED_JSON" > tmp.$$.json && mv tmp.$$.json "$UNIFIED_JSON"
    fi
}

# 1. Basic Benchmark Suite
echo -e "\n${BLUE}📊 Running Basic Benchmark Suite${NC}"
echo -e "${YELLOW}Testing startup, API responses, code analysis, memory usage, and concurrency${NC}"

if ./benchmark.sh; then
    LATEST_BASIC=$(ls -t "$RESULTS_DIR"/benchmark_*.json | head -1)
    update_unified_results "basic_benchmarks" "$LATEST_BASIC"
else
    echo -e "${RED}❌ Basic benchmark suite failed${NC}"
    jq '.summary.tests_failed += 1' "$UNIFIED_JSON" > tmp.$$.json && mv tmp.$$.json "$UNIFIED_JSON"
fi

# 2. Advanced Load Testing
echo -e "\n${BLUE}🌐 Running Advanced Load Testing${NC}"
echo -e "${YELLOW}Simulating realistic user loads with multiple concurrency levels${NC}"

# Test different concurrency levels
CONCURRENCY_LEVELS=(5 10 20 50)
for level in "${CONCURRENCY_LEVELS[@]}"; do
    echo -e "${BLUE}Testing with $level concurrent users${NC}"
    
    if node benchmarks/load-test.js --concurrency="$level" --duration=30000 --url="$BASE_URL"; then
        LATEST_LOAD=$(ls -t "$RESULTS_DIR"/load-test-*.json | head -1)
        update_unified_results "load_test_${level}_users" "$LATEST_LOAD"
    else
        echo -e "${RED}❌ Load test with $level users failed${NC}"
        jq '.summary.tests_failed += 1' "$UNIFIED_JSON" > tmp.$$.json && mv tmp.$$.json "$UNIFIED_JSON"
    fi
    
    # Cool down between tests
    echo -e "${YELLOW}Cooling down for 10 seconds...${NC}"
    sleep 10
done

# 3. Performance Profiling
echo -e "\n${BLUE}🔍 Running Performance Profiling${NC}"
echo -e "${YELLOW}Deep system and application performance analysis${NC}"

if python3 benchmarks/performance-profiler.py --url="$BASE_URL" --duration=120 --interval=2; then
    LATEST_PROFILE=$(ls -t "$RESULTS_DIR"/performance-profile-*.json | head -1)
    update_unified_results "performance_profile" "$LATEST_PROFILE"
else
    echo -e "${RED}❌ Performance profiling failed${NC}"
    jq '.summary.tests_failed += 1' "$UNIFIED_JSON" > tmp.$$.json && mv tmp.$$.json "$UNIFIED_JSON"
fi

# 4. Stress Testing (High Load)
echo -e "\n${BLUE}💪 Running Stress Testing${NC}"
echo -e "${YELLOW}Testing application limits with extreme loads${NC}"

if node benchmarks/load-test.js --concurrency=100 --duration=60000 --url="$BASE_URL"; then
    LATEST_STRESS=$(ls -t "$RESULTS_DIR"/load-test-*.json | head -1)
    update_unified_results "stress_test" "$LATEST_STRESS"
else
    echo -e "${RED}❌ Stress testing failed${NC}"
    jq '.summary.tests_failed += 1' "$UNIFIED_JSON" > tmp.$$.json && mv tmp.$$.json "$UNIFIED_JSON"
fi

# Calculate total duration
SUITE_END_TIME=$(date +%s)
TOTAL_DURATION=$((SUITE_END_TIME - SUITE_START_TIME))
jq --arg duration "$TOTAL_DURATION" '.summary.total_duration = ($duration | tonumber)' "$UNIFIED_JSON" > tmp.$$.json && mv tmp.$$.json "$UNIFIED_JSON"

# Generate Comprehensive Markdown Report
echo -e "\n${BLUE}📄 Generating Comprehensive Report${NC}"

cat > "$UNIFIED_REPORT" << EOF
# Dev Assistant Hub - Comprehensive Benchmark Report

**Generated:** $(date)  
**Duration:** ${TOTAL_DURATION} seconds  
**Target URL:** $BASE_URL

## Executive Summary

$(jq -r '
"- **Total Tests:** \(.summary.tests_completed + .summary.tests_failed)
- **Successful Tests:** \(.summary.tests_completed)  
- **Failed Tests:** \(.summary.tests_failed)
- **Success Rate:** \((.summary.tests_completed / (.summary.tests_completed + .summary.tests_failed) * 100) | floor)%
- **Total Duration:** \(.summary.total_duration) seconds"
' "$UNIFIED_JSON")

## System Information

$(jq -r '
if .benchmarks.basic_benchmarks then
  .benchmarks.basic_benchmarks.system_info | 
  "- **Hostname:** \(.hostname)
- **OS:** \(.os) \(.kernel)
- **CPU Cores:** \(.cpu_cores)
- **Memory:** \(.memory_total) total, \(.memory_available) available
- **GPU:** \(.gpu_info)
- **Node.js:** \(.node_version)
- **Python:** \(.python_version)"
else
  "System information not available"
end
' "$UNIFIED_JSON")

## Performance Highlights

### 🚀 Application Performance
$(jq -r '
if .benchmarks.basic_benchmarks then
  .benchmarks.basic_benchmarks.benchmarks |
  "- **Startup Time:** \(.startup_performance.warm_start_time)ms
- **Peak Memory Usage:** \(.memory_usage.peak_memory_mb)MB
- **Memory Efficiency:** \(.memory_usage.memory_increase_mb)MB increase under load"
else
  "Basic performance data not available"
end
' "$UNIFIED_JSON")

### 🌐 API Performance
$(jq -r '
if .benchmarks.basic_benchmarks then
  .benchmarks.basic_benchmarks.benchmarks.api_performance.endpoints | 
  to_entries[] | 
  "- **\(.value.description):** \(.value.average_ms)ms avg (min: \(.value.min_ms)ms, max: \(.value.max_ms)ms)"
else
  "API performance data not available"
end
' "$UNIFIED_JSON")

### 👥 Load Testing Results

#### Concurrency Performance
$(for level in 5 10 20 50; do
    jq -r --arg level "$level" '
    if .benchmarks["load_test_\($level)_users"] then
      .benchmarks["load_test_\($level)_users"].summary |
      "- **\(.concurrency) Users:** \(.requestsPerSecond) req/s, \(.errorRate)% error rate, \(.responseTime.p95)ms P95"
    else
      "- **\($level) Users:** Data not available"
    end
    ' "$UNIFIED_JSON"
done)

#### Stress Test Results
$(jq -r '
if .benchmarks.stress_test then
  .benchmarks.stress_test.summary |
  "- **Peak Load:** \(.concurrency) concurrent users
- **Requests/Second:** \(.requestsPerSecond)
- **Error Rate:** \(.errorRate)%
- **P99 Response Time:** \(.responseTime.p99)ms"
else
  "Stress test data not available"
end
' "$UNIFIED_JSON")

### 🔍 System Resource Usage
$(jq -r '
if .benchmarks.performance_profile then
  .benchmarks.performance_profile.summary |
  "- **Average CPU:** \(.average_cpu_percent)%
- **Average Memory:** \(.average_memory_mb)MB
- **Peak Response Time:** \(.p95_response_time_ms)ms P95
- **System Stability:** \(if .error_rate_percent < 1 then "Excellent" elif .error_rate_percent < 5 then "Good" else "Needs Attention" end)"
else
  "Performance profile data not available"
end
' "$UNIFIED_JSON")

## Detailed Analysis

### Code Analysis Performance
$(jq -r '
if .benchmarks.basic_benchmarks then
  .benchmarks.basic_benchmarks.benchmarks.code_analysis |
  "- **Analysis Time:** \(.analysis_time_ms)ms for \(.code_lines) lines
- **Processing Rate:** \((.code_lines * 1000 / .analysis_time_ms) | floor) lines/second
- **Workflow Status:** \(if .workflow_complete then "✅ Complete" else "❌ Failed" end)"
else
  "Code analysis data not available"
end
' "$UNIFIED_JSON")

### Concurrent User Handling
$(jq -r '
if .benchmarks.basic_benchmarks then
  .benchmarks.basic_benchmarks.benchmarks.concurrent_users.levels | 
  to_entries[] | 
  "- **\(.value.concurrent_users) Users:** \(.value.requests_per_second) req/s, \(.value.avg_response_time_ms)ms avg response"
else
  "Concurrent user data not available"
end
' "$UNIFIED_JSON")

## Recommendations

### Performance Optimization
- **Memory Management:** $(jq -r 'if .benchmarks.basic_benchmarks.benchmarks.memory_usage.memory_increase_mb > 100 then "Consider implementing memory pooling and garbage collection optimization" else "Memory usage is within acceptable limits" end' "$UNIFIED_JSON")
- **API Response Times:** $(jq -r 'if .benchmarks.basic_benchmarks then (.benchmarks.basic_benchmarks.benchmarks.api_performance.endpoints | to_entries | map(.value.average_ms) | max) as $max_time | if $max_time > 1000 then "Some API endpoints are slow (>\($max_time)ms). Consider caching and optimization." else "API response times are acceptable" end else "Unable to assess API performance" end' "$UNIFIED_JSON")
- **Concurrency:** $(jq -r 'if .benchmarks.stress_test then (.benchmarks.stress_test.summary.errorRate) as $error_rate | if $error_rate > 5 then "High error rate under stress (\($error_rate)%). Consider connection pooling and rate limiting." else "Application handles concurrent load well" end else "Unable to assess concurrency performance" end' "$UNIFIED_JSON")

### Scalability Considerations
- **Database Connections:** Monitor connection pool usage during peak loads
- **Memory Scaling:** Consider horizontal scaling if memory usage exceeds 80% consistently
- **CPU Optimization:** Profile CPU-intensive operations for optimization opportunities

### Monitoring Setup
- **Real-time Alerts:** Set up alerts for response times > 2000ms
- **Resource Monitoring:** Monitor memory usage and set alerts at 85% utilization
- **Error Rate Tracking:** Alert on error rates > 2%

## Test Configuration

### Basic Benchmarks
- Startup time measurement
- API endpoint response testing (10 samples each)
- Code analysis workflow testing
- Memory usage monitoring
- Concurrent user simulation (1, 5, 10, 20 users)

### Load Testing
- Multiple concurrency levels: 5, 10, 20, 50 users
- Duration: 30 seconds per test
- Ramp-up time: 5 seconds
- Endpoint distribution based on realistic usage patterns

### Performance Profiling
- Duration: 120 seconds
- Monitoring interval: 2 seconds
- System-wide resource monitoring
- Application-specific process tracking

### Stress Testing
- Peak concurrency: 100 users
- Duration: 60 seconds
- Designed to test application limits

---

**Report Generated by Dev Assistant Hub Benchmark Suite**  
**Timestamp:** $TIMESTAMP  
**Total Execution Time:** ${TOTAL_DURATION} seconds
EOF

# Final Summary
echo -e "\n${PURPLE}🎉 Comprehensive Benchmark Suite Completed!${NC}"
echo -e "${BLUE}================================================================${NC}"
echo -e "${GREEN}✅ Tests Completed: $(jq -r '.summary.tests_completed' "$UNIFIED_JSON")${NC}"
echo -e "${RED}❌ Tests Failed: $(jq -r '.summary.tests_failed' "$UNIFIED_JSON")${NC}"
echo -e "${YELLOW}⏱️  Total Duration: ${TOTAL_DURATION} seconds${NC}"
echo -e "${BLUE}================================================================${NC}"

echo -e "\n${YELLOW}📊 Results Available:${NC}"
echo -e "  📄 Comprehensive Report: $UNIFIED_REPORT"
echo -e "  📊 Unified JSON Data: $UNIFIED_JSON"
echo -e "  📁 Individual Results: $RESULTS_DIR/"

echo -e "\n${BLUE}🔍 Quick Performance Summary:${NC}"
if [ -f "$UNIFIED_JSON" ]; then
    echo -e "  🚀 Application Status: $(jq -r 'if .benchmarks.basic_benchmarks then "✅ Operational" else "⚠️  Limited Data" end' "$UNIFIED_JSON")"
    echo -e "  💾 Peak Memory: $(jq -r 'if .benchmarks.basic_benchmarks then .benchmarks.basic_benchmarks.benchmarks.memory_usage.peak_memory_mb + "MB" else "N/A" end' "$UNIFIED_JSON")"
    echo -e "  🌐 Best API Response: $(jq -r 'if .benchmarks.basic_benchmarks then ([.benchmarks.basic_benchmarks.benchmarks.api_performance.endpoints[].min_ms] | min | tostring + "ms") else "N/A" end' "$UNIFIED_JSON")"
    echo -e "  👥 Max Concurrent Users Tested: $(jq -r 'if .benchmarks.stress_test then .benchmarks.stress_test.configuration.concurrency else "50" end' "$UNIFIED_JSON")"
fi

echo -e "\n${GREEN}Benchmark suite execution completed successfully!${NC}"
