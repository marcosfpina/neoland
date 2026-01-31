# NEOLAND E2E Tests

**End-to-End testing suite for NEOLAND TUI using expect scripts**

---

## Overview

This directory contains automated E2E tests for the NEOLAND Terminal User Interface (TUI). Tests are written using `expect` scripts to automate terminal interactions and verify the complete user workflow.

## Prerequisites

### Required Tools

```bash
# Debian/Ubuntu
sudo apt install expect

# Fedora/RHEL
sudo dnf install expect

# macOS
brew install expect

# Arch Linux
sudo pacman -S expect
```

### Running Services

Tests require these services to be running:

1. **NEOLAND Server** (port 3001)
   ```bash
   cargo run --release --bin neoland -- server
   ```

2. **ml-offload Service** (port 8000) - Optional
   ```bash
   docker start ml-offload
   # OR
   # Start your ml-offload service
   ```

---

## Test Suite

### 1. Smoke Test (`tui_smoke_test.exp`)

**Purpose**: Verify basic TUI functionality

**Tests**:
- TUI starts successfully
- Send simple message → receive response
- Send follow-up message (conversation flow)
- Clear chat (Ctrl+L)
- Quit application (Ctrl+C)

**Run**:
```bash
./tui_smoke_test.exp http://localhost:3001 http://localhost:8000
```

### 2. Presets Test (`tui_presets_test.exp`)

**Purpose**: Verify preset configurations work

**Tests**:
- Apply Balanced preset (Ctrl+1)
- Apply Creative preset (Ctrl+2)
- Apply Precise preset (Ctrl+3) - may skip
- Apply Research preset (Ctrl+4)
- Apply Safe preset (Ctrl+5)
- Each preset receives LLM response

**Run**:
```bash
./tui_presets_test.exp http://localhost:3001 http://localhost:8000
```

### 3. Fallback Chain Test (`tui_fallback_test.exp`)

**Purpose**: Verify LLM fallback chain works

**Tests**:
- TUI works with ml-offload DOWN
- Fallback to local engine or SecureLLM
- Multiple requests handled consistently
- RAG context still functional with fallback

**Prerequisites**: ml-offload must be STOPPED
```bash
# Stop ml-offload first
docker stop ml-offload
# OR
pkill ml-offload

# Run test
./tui_fallback_test.exp http://localhost:3001 http://localhost:8000

# Restart ml-offload after
docker start ml-offload
```

**Run**:
```bash
RUN_FALLBACK_TEST=1 ./run_e2e.sh
```

---

## Running All Tests

### Quick Start

```bash
# Run all tests (except fallback test)
./run_e2e.sh

# Run all tests INCLUDING fallback test
RUN_FALLBACK_TEST=1 ./run_e2e.sh
```

### Custom URLs

```bash
# Use custom server/ML API URLs
SERVER_URL=http://staging:3001 ML_API_URL=http://staging:8000 ./run_e2e.sh
```

### CI/CD Integration

```bash
# Run in CI environment
export SERVER_URL=http://neoland:3001
export ML_API_URL=http://ml-offload:8000
./run_e2e.sh
```

---

## Test Output

### Successful Run
```
🧪 TUI Smoke Test
   Server: http://localhost:3001
   ML API: http://localhost:8000

✅ PASS: TUI started successfully

📝 Test 1: Send simple message 'Hello'
✅ PASS: Received response from LLM

📝 Test 2: Send follow-up message
✅ PASS: Received correct answer (4)

📝 Test 3: Clear chat (Ctrl+L)
✅ PASS: Chat cleared successfully

📝 Test 4: Quit application (Ctrl+C)
✅ PASS: Application quit cleanly

✅ All TUI smoke tests passed!
```

### Failed Run
```
❌ FAIL: TUI failed to start within 30 seconds
```

---

## Debugging Failed Tests

### Common Issues

#### 1. Server Not Running
```
Error: Connection refused
```

**Fix**:
```bash
cargo run --release --bin neoland -- server &
sleep 5
```

#### 2. Timeout Waiting for Response
```
❌ FAIL: No response received within 30 seconds
```

**Possible causes**:
- ml-offload down (check fallback is working)
- SecureLLM API key missing
- Network issues

**Debug**:
```bash
# Check server health
curl http://localhost:3001/health | jq

# Check ml-offload health
curl http://localhost:8000/health | jq

# Check server logs
cargo run --bin neoland -- server 2>&1 | tee server.log
```

#### 3. Expect Script Timeout
```
❌ FAIL: timeout
```

**Fix**: Increase timeout in test script
```tcl
set timeout 60  ;# Increase from 30 to 60 seconds
```

#### 4. TUI Not Rendering
```
❌ FAIL: TUI failed to start
```

**Fix**: Verify terminal supports ANSI/VT100
```bash
# Test terminal capabilities
echo $TERM
# Expected: xterm, xterm-256color, screen, etc.

# Run with explicit TERM
TERM=xterm-256color ./tui_smoke_test.exp
```

---

## Writing New Tests

### Template

```tcl
#!/usr/bin/expect -f
# E2E Test: <Test Name>
# Tests: <What this test verifies>

set timeout 30
set server_url [lindex $argv 0]
set ml_api_url [lindex $argv 1]

if {$server_url == ""} {
    set server_url "http://localhost:3001"
}
if {$ml_api_url == ""} {
    set ml_api_url "http://localhost:8000"
}

puts "🧪 <Test Name>"
puts "   Server: $server_url"
puts "   ML API: $ml_api_url"
puts ""

# Start TUI
spawn cargo run --bin neoland -- tui --server-url $server_url --ml-api-url $ml_api_url

# Wait for TUI
expect {
    timeout {
        puts "❌ FAIL: TUI failed to start"
        exit 1
    }
    "NEOLAND" {
        puts "✅ PASS: TUI started"
    }
}

# Your test logic here
send "Test message\r"

expect {
    timeout {
        puts "❌ FAIL: No response"
        exit 1
    }
    "response" {
        puts "✅ PASS: Response received"
    }
}

# Cleanup
send "\003"  ;# Ctrl+C
expect eof

puts "\n✅ Test passed!"
exit 0
```

### Best Practices

1. **Always set timeout**: Default is too short for LLM responses
   ```tcl
   set timeout 30  ;# 30 seconds
   ```

2. **Handle both success and failure**:
   ```tcl
   expect {
       timeout { puts "❌ FAIL"; exit 1 }
       "success" { puts "✅ PASS" }
   }
   ```

3. **Clean up after test**:
   ```tcl
   send "\003"  ;# Ctrl+C to quit
   expect eof
   ```

4. **Use descriptive output**:
   ```tcl
   puts "\n📝 Test 1: Send message"
   ```

5. **Accept partial matches**:
   ```tcl
   expect -re "(Hello|Hi|Hey)"  ;# Regex for flexibility
   ```

---

## Integration with CI/CD

### GitHub Actions

```yaml
name: E2E Tests

on: [push, pull_request]

jobs:
  e2e:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install expect
        run: sudo apt-get install -y expect

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build project
        run: cargo build --release

      - name: Start server
        run: cargo run --release --bin neoland -- server &

      - name: Wait for server
        run: sleep 10

      - name: Run E2E tests
        run: ./tests/e2e/run_e2e.sh
        env:
          SERVER_URL: http://localhost:3001
          ML_API_URL: http://localhost:8000
```

---

## Maintenance

### Updating Tests

When TUI changes:
1. Update corresponding expect script
2. Test manually first
3. Update this README if new tests added
4. Keep timeout values reasonable

### Adding New Tests

1. Create new `.exp` file
2. Make executable: `chmod +x <test>.exp`
3. Add to `run_e2e.sh`
4. Document in this README
5. Add to CI/CD pipeline

---

## Troubleshooting

### General Debug Mode

Run expect with debug flag:
```bash
expect -d tui_smoke_test.exp
```

### Capture TUI Output

```bash
script -c "./tui_smoke_test.exp" tui_output.log
```

### Manual Testing

Run TUI manually to verify behavior:
```bash
cargo run --bin neoland -- tui \
  --server-url http://localhost:3001 \
  --ml-api-url http://localhost:8000
```

---

## Performance

- **Smoke Test**: ~30-60 seconds
- **Presets Test**: ~60-90 seconds
- **Fallback Test**: ~90-120 seconds
- **Full Suite**: ~3-5 minutes

---

## Related Documentation

- **TUI Source**: `src/tui/`
- **Integration Tests**: `tests/grpc_integration_test.rs`
- **REST API Tests**: `tests/rest_api_test.rs`
- **CI/CD Pipeline**: `.github/workflows/ci.yml`

---

**Last Updated**: 2026-01-31
**Maintainer**: voidnx team
**Status**: Phase 2.3 - E2E Testing Complete
