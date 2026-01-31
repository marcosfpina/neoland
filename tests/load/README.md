

# NEOLAND Load Testing Suite

**Performance and scalability testing for production readiness**

---

## Overview

Comprehensive load testing infrastructure for NEOLAND covering:

- **gRPC Load Testing**: ghz-based load tests targeting 500 RPS sustained
- **REST API Load Testing**: wrk/Apache Bench tests for HTTP endpoints
- **Rust Benchmarks**: Criterion-based microbenchmarks for critical paths
- **Resource Monitoring**: CPU, memory, network, and thread usage tracking
- **Performance Profiling**: Identifying bottlenecks and optimization opportunities

---

## Quick Start

### Prerequisites

```bash
# Install load testing tools
# ghz (gRPC load testing)
go install github.com/bojand/ghz/cmd/ghz@latest

# wrk (HTTP load testing) - macOS
brew install wrk

# Or Apache Bench (alternative to wrk) - Linux
sudo apt install apache2-utils

# Rust toolchain (for benchmarks)
rustup component add llvm-tools-preview
```

### Running All Load Tests

```bash
# 1. Start NEOLAND server
cargo run --release --bin neoland -- server &

# 2. Wait for server to be ready
sleep 5

# 3. Run complete load test suite
./tests/load/run_load_tests.sh
```

### Running Individual Tests

```bash
# gRPC load test only
./tests/load/grpc_load_test.sh

# REST API load test only
./tests/load/rest_load_test.sh

# Resource monitoring only (60s)
DURATION=60 ./tests/load/monitor_resources.sh

# Rust benchmarks
cargo bench --bench inference_benchmark
```

---

## Test Categories

### 1. gRPC Load Testing (`grpc_load_test.sh`)

**Purpose**: Test gRPC endpoint performance under various load conditions

**Test Scenarios**:

1. **Warmup** (10 RPS, 10s)
   - Purpose: Prime caches, establish baseline
   - Requests: 100 total
   - Connections: 10

2. **Baseline** (100 RPS, 30s)
   - Purpose: Normal load performance
   - Duration: 30 seconds
   - Connections: 50
   - Target: Establish performance baseline

3. **Target Load** (500 RPS, 60s)
   - Purpose: Production target validation
   - Duration: 60 seconds
   - Connections: 100
   - **SLO**: p99 latency < 200ms

4. **Stress Test** (800 RPS, 30s)
   - Purpose: Test beyond expected load
   - Duration: 30 seconds
   - Connections: 200
   - Expected: Some degradation acceptable

5. **Spike Test** (1000 RPS, 10s)
   - Purpose: Test sudden load increase
   - Duration: 10 seconds
   - Connections: 300
   - Expected: System recovers after spike

**Command**:
```bash
./tests/load/grpc_load_test.sh
```

**Custom Configuration**:
```bash
# Custom duration and RPS
DURATION=120s RPS=750 ./tests/load/grpc_load_test.sh

# Custom gRPC endpoint
GRPC_URL=staging.example.com:50051 ./tests/load/grpc_load_test.sh
```

**Example Output**:
```
╔════════════════════════════════════════════╗
║   NEOLAND gRPC Load Testing (ghz)         ║
╚════════════════════════════════════════════╝

Configuration:
  gRPC URL:      localhost:50051
  Duration:      60s
  Target RPS:    500
  Connections:   100

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Test 3: Target Load (500 RPS, 60s)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📊 Target Load Results (500 RPS):
   Average latency:  45.2ms
   p50 latency:      38.5ms
   p95 latency:      125.3ms
   p99 latency:      186.7ms ✅
   Actual RPS:       498.3
   Successful:       29898

✅ Target (500 RPS): 186.7ms < 200ms
```

**Reports Generated**:
- `target/load-reports/grpc_warmup.json`
- `target/load-reports/grpc_baseline_100rps.json`
- `target/load-reports/grpc_target_500rps.json`
- `target/load-reports/grpc_stress_800rps.json`
- `target/load-reports/grpc_spike_1000rps.json`

---

### 2. REST API Load Testing (`rest_load_test.sh`)

**Purpose**: Test REST API endpoint performance

**Tool Selection**:
- Primary: `wrk` (preferred, more features)
- Fallback: Apache Bench `ab` (if wrk unavailable)

**Test Scenarios**:

1. **Warmup** (10s, 10 connections)
2. **Baseline** (50 connections, 30s)
3. **Target Load** (100 connections, 60s)
4. **Stress Test** (200 connections, 30s)
5. **Health Endpoint** (1000 connections, 10s)

**Command**:
```bash
./tests/load/rest_load_test.sh
```

**Custom Configuration**:
```bash
# Custom endpoint
ENDPOINT=/v1/custom/endpoint ./tests/load/rest_load_test.sh

# Custom server URL
REST_URL=https://api.example.com ./tests/load/rest_load_test.sh

# Custom duration and connections
DURATION=120s CONNECTIONS=200 ./tests/load/rest_load_test.sh
```

**Example Output (wrk)**:
```
Running 60s test @ http://localhost:3001/v1/chat/completions
  10 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    52.34ms   28.67ms  324.12ms   78.45%
    Req/Sec    51.23     12.45    98.00     69.23%
  30234 requests in 60.00s, 45.23MB read
Requests/sec:    503.90
Transfer/sec:    0.75MB

📊 Target Load Results (100 connections):
   Requests/sec:     503.90 ✅
   Average latency:  52.34ms
   p99 latency:      189.23ms ✅
```

**Reports Generated**:
- `target/load-reports/rest_baseline.txt`
- `target/load-reports/rest_target.txt`
- `target/load-reports/rest_stress.txt`
- `target/load-reports/rest_health.txt`

---

### 3. Resource Monitoring (`monitor_resources.sh`)

**Purpose**: Track system resource usage during load tests

**Metrics Collected**:
- **CPU Usage**: Percentage per core
- **Memory Usage**: RSS in MB and percentage
- **Thread Count**: Active threads
- **Open Files**: File descriptor count
- **Network I/O**: Bytes received/transmitted

**Command**:
```bash
# Monitor for 60 seconds
DURATION=60 ./tests/load/monitor_resources.sh

# Custom interval (default: 2s)
INTERVAL=5 DURATION=120 ./tests/load/monitor_resources.sh

# Monitor specific process
PROCESS_NAME=my-server ./tests/load/monitor_resources.sh
```

**Example Output**:
```
╔════════════════════════════════════════════╗
║   NEOLAND Resource Monitoring              ║
╚════════════════════════════════════════════╝

✅ Found process 'neoland' (PID: 12345)

🔍 Monitoring started...

Time    CPU%   Memory(MB)  Threads  Files  RX(MB)  TX(MB)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
14:30:00  45.2  1234.56     128      456    12.34   8.76
14:30:02  52.8  1289.34     142      478    15.67   11.23
14:30:04  48.3  1256.78     135      465    18.92   14.56
...

📊 Summary Statistics:

   CPU Usage:
     Average:  48.7%
     Peak:     62.3%

   Memory Usage:
     Average:  1256.45 MB
     Peak:     1432.67 MB

   Threads:
     Average:  136
     Peak:     152

   Network Transfer:
     Received: 125.45 MB
     Sent:     89.23 MB
```

**CSV Output**: `target/load-reports/resource_monitor_YYYYMMDD_HHMMSS.csv`

**Columns**:
```csv
timestamp,elapsed_sec,cpu_percent,mem_mb,mem_percent,threads,open_files,network_rx_mb,network_tx_mb
14:30:00,0,45.2,1234.56,12.3,128,456,0.00,0.00
14:30:02,2,52.8,1289.34,12.9,142,478,3.33,2.47
```

**Visualization**:
```bash
# Import CSV into spreadsheet (Excel, Google Sheets, LibreOffice)
# Or use gnuplot:

gnuplot << EOF
set terminal png size 1200,800
set output 'cpu_usage.png'
set datafile separator ','
set xlabel 'Time (seconds)'
set ylabel 'CPU %'
set title 'NEOLAND CPU Usage During Load Test'
plot 'resource_monitor.csv' using 2:3 with lines title 'CPU'
EOF
```

---

### 4. Rust Benchmarks (`benches/inference_benchmark.rs`)

**Purpose**: Microbenchmarks for critical code paths

**Benchmark Categories**:

1. **JSON Parsing**
   - Small payload (< 100 bytes)
   - Medium payload (~1KB)
   - Large payload (~10KB)

2. **String Operations**
   - Concatenation (100 items)
   - Format macro

3. **Vector Operations**
   - Vec::push (1000 items)
   - Vec::with_capacity (1000 items)
   - Iteration and sum

4. **Hash Operations**
   - HashMap insertion (1000 items)
   - HashMap lookup

5. **Async Operations**
   - Tokio task spawning (100 tasks)
   - Tokio channel (1000 messages)

**Command**:
```bash
# Run all benchmarks
cargo bench --bench inference_benchmark

# Run specific benchmark group
cargo bench --bench inference_benchmark json_parsing

# Save baseline for comparison
cargo bench --bench inference_benchmark -- --save-baseline main

# Compare against baseline
git checkout feature-branch
cargo bench --bench inference_benchmark -- --baseline main
```

**Example Output**:
```
json_parsing/small_payload
                        time:   [1.2345 µs 1.2567 µs 1.2789 µs]
                        change: [-2.34% -1.23% +0.12%] (p = 0.08 > 0.05)
                        No change in performance detected.

json_parsing/large_payload
                        time:   [45.678 µs 46.234 µs 46.789 µs]
                        thrpt:  [213.67 MB/s 216.23 MB/s 218.90 MB/s]
                        change: [+3.45% +4.56% +5.67%] (p = 0.00 < 0.05)
                        Performance has regressed.
```

**Reports**: `target/criterion/`

---

## Performance Targets (SLOs)

### Production SLOs

| Metric | Target | Critical Threshold |
|--------|--------|-------------------|
| **Throughput (gRPC)** | 500 RPS sustained | < 400 RPS |
| **Throughput (REST)** | 500 RPS sustained | < 400 RPS |
| **Latency (p50)** | < 50ms | > 100ms |
| **Latency (p95)** | < 150ms | > 300ms |
| **Latency (p99)** | < 200ms | > 500ms |
| **CPU Usage** | < 70% average | > 90% sustained |
| **Memory Usage** | < 2GB | > 4GB |
| **Thread Count** | < 200 | > 500 |
| **Error Rate** | < 0.1% | > 1% |

### Load Test Matrix

| Scenario | RPS | Connections | Duration | Purpose |
|----------|-----|-------------|----------|---------|
| Warmup | 10 | 10 | 10s | Cache priming |
| Baseline | 100 | 50 | 30s | Normal load |
| **Target** | **500** | **100** | **60s** | **Production** |
| Stress | 800 | 200 | 30s | Over-capacity |
| Spike | 1000 | 300 | 10s | Burst handling |

---

## Analyzing Results

### 1. Check SLO Compliance

```bash
# gRPC p99 latency (should be < 200ms)
jq -r '.latencyDistribution."99"' target/load-reports/grpc_target_500rps.json

# Actual RPS achieved
jq -r '.rps' target/load-reports/grpc_target_500rps.json

# Error rate
jq -r '.statusCodeDistribution' target/load-reports/grpc_target_500rps.json
```

### 2. Compare Baseline vs Target

```bash
# Baseline p99
BASELINE_P99=$(jq -r '.latencyDistribution."99"' target/load-reports/grpc_baseline_100rps.json | sed 's/ms//')

# Target p99
TARGET_P99=$(jq -r '.latencyDistribution."99"' target/load-reports/grpc_target_500rps.json | sed 's/ms//')

# Calculate degradation
echo "Latency increase: $(echo "scale=2; ($TARGET_P99 - $BASELINE_P99) / $BASELINE_P99 * 100" | bc)%"
```

### 3. Resource Correlation

```bash
# Find peak CPU during target load
awk -F',' 'NR>1 && $3!="N/A" {if($3>max) max=$3} END {print "Peak CPU:", max "%"}' \
    target/load-reports/resource_monitor_*.csv

# Check memory growth
awk -F',' 'NR>1 && $4!="N/A" {if(NR==2) start=$4; end=$4} END {print "Memory growth:", end-start, "MB"}' \
    target/load-reports/resource_monitor_*.csv
```

---

## Performance Profiling

### 1. CPU Profiling with Flamegraph

```bash
# Install flamegraph
cargo install flamegraph

# Profile server under load
cargo flamegraph --bin neoland -- server &
sleep 5

# Run load test
./tests/load/grpc_load_test.sh

# Stop profiling
kill $(pgrep flamegraph)

# View flamegraph
open flamegraph.svg
```

### 2. Memory Profiling with Valgrind

```bash
# Build with debug symbols
cargo build --release --bin neoland

# Run with massif (heap profiler)
valgrind --tool=massif \
    --massif-out-file=massif.out \
    ./target/release/neoland server &

# Run load test
./tests/load/rest_load_test.sh

# Analyze results
ms_print massif.out > memory_profile.txt
```

### 3. Async Runtime Profiling (Tokio Console)

```bash
# Enable tokio-console in Cargo.toml (dev dependency)
# [dependencies]
# tokio = { version = "1", features = ["full", "tracing"] }
# console-subscriber = "0.2"

# Run server with console
RUSTFLAGS="--cfg tokio_unstable" cargo run --bin neoland -- server &

# In another terminal, run tokio-console
tokio-console

# Run load test
./tests/load/grpc_load_test.sh
```

---

## Troubleshooting

### Issue 1: Low RPS Achieved

**Symptoms**:
```
Target: 500 RPS
Actual: 234 RPS
```

**Possible Causes**:
1. Server bottleneck (CPU/memory)
2. Network bottleneck
3. Client-side limiting

**Debug**:
```bash
# 1. Check server resource usage
htop -p $(pgrep neoland)

# 2. Check server is not throttling
curl http://localhost:3001/metrics | grep rate_limit

# 3. Check client can generate load
ghz --insecure -z 10s -c 100 --rps 1000 localhost:50051 # Test client capacity

# 4. Check network saturation
iftop  # Monitor network bandwidth
```

### Issue 2: High p99 Latency

**Symptoms**:
```
p99 latency: 456ms (target: 200ms)
```

**Possible Causes**:
1. GC pauses (if using GC languages, not Rust)
2. Thread starvation
3. Database/external API slowness
4. Lock contention

**Debug**:
```bash
# 1. Profile with flamegraph (see above)

# 2. Check thread count
pgrep -af neoland | wc -l

# 3. Enable detailed tracing
RUST_LOG=trace cargo run --bin neoland -- server 2>&1 | tee trace.log

# 4. Check for lock contention
cargo flamegraph --bin neoland -- server &
# Analyze for "parking_lot" or "std::sync" hot spots
```

### Issue 3: Memory Growth

**Symptoms**:
```
Start: 500 MB
End:   2.5 GB
```

**Possible Causes**:
1. Memory leak
2. Unbounded cache
3. Connection pooling leak

**Debug**:
```bash
# 1. Memory profiling (see Performance Profiling section)

# 2. Check for leaked connections
lsof -p $(pgrep neoland) | grep -c ESTABLISHED

# 3. Monitor allocations
MALLOC_CONF=prof:true cargo run --bin neoland -- server
```

### Issue 4: Server Crashes Under Load

**Symptoms**:
```
Server exits with: signal: 9 (SIGKILL)
```

**Possible Causes**:
1. OOM killer
2. Stack overflow
3. Panic in critical path

**Debug**:
```bash
# 1. Check kernel logs
sudo dmesg | grep -i "out of memory"

# 2. Run with core dumps enabled
ulimit -c unlimited
cargo run --bin neoland -- server

# 3. Check panic backtraces
RUST_BACKTRACE=full cargo run --bin neoland -- server 2>&1 | tee crash.log
```

---

## CI/CD Integration

### GitHub Actions Workflow

```yaml
name: Load Tests

on:
  push:
    branches: [main]
  schedule:
    # Run weekly on Sundays at 2 AM
    - cron: '0 2 * * 0'

jobs:
  load-test:
    runs-on: ubuntu-latest
    timeout-minutes: 30

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install ghz
        run: |
          go install github.com/bojand/ghz/cmd/ghz@latest
          echo "$HOME/go/bin" >> $GITHUB_PATH

      - name: Install wrk
        run: |
          sudo apt-get update
          sudo apt-get install -y build-essential libssl-dev git
          git clone https://github.com/wg/wrk.git /tmp/wrk
          cd /tmp/wrk && make && sudo cp wrk /usr/local/bin/

      - name: Build project
        run: cargo build --release

      - name: Run load tests
        run: ./tests/load/run_load_tests.sh
        env:
          RUN_BENCHMARKS: "false"  # Skip benchmarks in CI

      - name: Upload load test reports
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: load-test-reports
          path: target/load-reports/
```

---

## Best Practices

### 1. Baseline Before Optimization

Always establish a performance baseline before making changes:

```bash
# Baseline
git checkout main
./tests/load/run_load_tests.sh
mv target/load-reports target/baseline-reports

# After optimization
git checkout feature/optimization
./tests/load/run_load_tests.sh

# Compare
diff target/baseline-reports/grpc_target_500rps.json \
     target/load-reports/grpc_target_500rps.json
```

### 2. Realistic Data

Use production-like data in load tests:

```lua
-- wrk Lua script with varied payloads
local prompts = {
    "What is Rust?",
    "Explain async/await in Rust",
    "How does the borrow checker work?",
    -- Add more varied prompts
}

request = function()
    local prompt = prompts[math.random(#prompts)]
    wrk.body = string.format('{"messages":[{"role":"user","content":"%s"}]}', prompt)
    return wrk.format("POST", wrk.path, wrk.headers, wrk.body)
end
```

### 3. Gradual Load Increase

Don't jump directly to max load:

```bash
# Ramp up test
for RPS in 100 250 500 750 1000; do
    echo "Testing ${RPS} RPS..."
    RPS=$RPS ./tests/load/grpc_load_test.sh
    sleep 30  # Cool down between tests
done
```

### 4. Monitor During Tests

Always monitor resources during load tests:

```bash
# Terminal 1: Run load test
./tests/load/grpc_load_test.sh

# Terminal 2: Monitor resources
watch -n 2 'ps aux | grep neoland | head -5'

# Terminal 3: Monitor network
iftop -i eth0
```

---

## Related Documentation

- **Security Testing**: `tests/security/README.md`
- **E2E Testing**: `tests/e2e/README.md`
- **Integration Tests**: `tests/grpc_integration_test.rs`
- **Production Roadmap**: `docs/PRODUCTION_ROADMAP.md`
- **Prometheus Metrics**: `docs/runbooks/high-latency.md`

---

**Last Updated**: 2026-01-31
**Maintainer**: voidnx team
**Status**: Phase 2.5 - Load Testing Complete
**Next**: Phase 4.6 - Disaster Recovery Planning
