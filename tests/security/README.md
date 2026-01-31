# NEOLAND Security Testing Suite

**Comprehensive security testing for production readiness**

---

## Overview

This directory contains the complete security testing infrastructure for NEOLAND, covering:

- **Static Analysis**: Dependency vulnerabilities, license compliance, code quality
- **Fuzzing**: REST API and gRPC endpoint fuzzing with invalid/malicious inputs
- **Penetration Testing**: Real-world attack scenarios (CORS, SSRF, timing attacks, etc.)
- **Secret Scanning**: Hardcoded credentials detection
- **Security Compliance**: OWASP Top 10 coverage

---

## Quick Start

### Prerequisites

```bash
# Install security testing tools
cargo install cargo-audit
cargo install cargo-deny

# Optional: Advanced tools
cargo install cargo-fuzz
cargo install cargo-geiger  # Unsafe code detection
```

### Running All Security Tests

```bash
# Start NEOLAND server first
cargo run --release --bin neoland -- server &

# Wait for server to be ready
sleep 5

# Run complete security suite
./tests/security/run_security_tests.sh
```

### Running Individual Test Categories

```bash
# Static analysis only
cargo audit
cargo deny check
cargo clippy -- -W clippy::security

# Fuzzing tests
cargo test --test fuzz_endpoints -- --ignored

# Penetration tests
cargo test --test pentest_scenarios -- --ignored
```

---

## Test Categories

### 1. Static Security Analysis

#### 1.1 Dependency Vulnerabilities (`cargo-audit`)

**Purpose**: Detect known security vulnerabilities in dependencies

**Command**:
```bash
cargo audit --json > target/security-reports/cargo-audit.json
```

**What it checks**:
- ✅ CVEs in dependencies
- ✅ Unmaintained crates
- ✅ Yanked versions
- ✅ Security advisories from RustSec

**Expected Result**: 0 vulnerabilities

**If vulnerabilities found**:
```bash
# Update dependencies
cargo update

# Check again
cargo audit

# If specific crate needs update
cargo update -p <crate-name>
```

#### 1.2 License & Ban Compliance (`cargo-deny`)

**Purpose**: Ensure license compliance and ban problematic dependencies

**Configuration**: `deny.toml` (project root)

**Command**:
```bash
cargo deny check
```

**What it checks**:
- ✅ Allowed licenses only (MIT, Apache-2.0, BSD, ISC)
- ✅ No GPL/AGPL (copyleft)
- ✅ No banned crates
- ✅ No multiple versions of same crate
- ✅ Trusted sources only (crates.io)

**Allowed Licenses**:
- MIT
- Apache-2.0
- BSD-2-Clause / BSD-3-Clause
- ISC
- Unicode-DFS-2016
- Unlicense
- Zlib

**Denied Licenses**:
- GPL-2.0, GPL-3.0
- AGPL-3.0
- Any copyleft

#### 1.3 Security Lints (`cargo-clippy`)

**Purpose**: Detect security-related code patterns

**Command**:
```bash
cargo clippy --all-targets --all-features -- \
    -D warnings \
    -W clippy::suspicious \
    -W clippy::security
```

**What it checks**:
- ✅ Integer overflow
- ✅ Unsafe code patterns
- ✅ Suspicious type conversions
- ✅ Potential panics
- ✅ Resource leaks

#### 1.4 Secret Scanning

**Purpose**: Detect hardcoded secrets in source code

**Patterns detected**:
- API keys (`api_key = "..."`)
- Passwords (`password = "..."`)
- Tokens (`token = "..."`)
- AWS credentials (`AKIA...`)
- Stripe keys (`sk_live_...`)
- GitHub tokens (`ghp_...`)

**Manual scan**:
```bash
rg -i 'api_key|password|secret|token' src/
```

**Expected Result**: 0 matches (all secrets in Vault)

---

### 2. Fuzzing Tests

**File**: `tests/security/fuzz_endpoints.rs`

**Purpose**: Test API robustness with malformed/malicious inputs

#### 2.1 REST API Invalid JSON

**Test**: `fuzz_rest_api_invalid_json`

**Attack Vectors**:
- Malformed JSON syntax
- Missing required fields
- Invalid field types
- Extremely large payloads (1MB+)
- Deep nested objects (1000+ levels)
- SQL injection attempts
- XSS payloads
- Command injection
- Null bytes
- Invalid UTF-16 surrogates

**Expected Behavior**:
- ✅ Return 4xx error (400, 401, 413)
- ✅ No server crash
- ✅ No timeout (respond within 5s)
- ❌ FAIL: 200 OK or server hang

**Run**:
```bash
cargo test --test fuzz_endpoints fuzz_rest_api_invalid_json -- --ignored --nocapture
```

#### 2.2 REST API Invalid Headers

**Test**: `fuzz_rest_api_invalid_headers`

**Attack Vectors**:
- Missing Content-Type
- Invalid Content-Type
- Header injection (`\r\n`)
- Extremely long headers (100KB+)
- Null bytes in headers
- Control characters

**Expected Behavior**:
- ✅ Reject invalid headers
- ✅ No header injection
- ✅ No crash

#### 2.3 Rate Limit Enforcement

**Test**: `test_rate_limit_enforcement`

**Attack**: Send 150 requests rapidly

**Expected Behavior**:
- ✅ First ~100 requests succeed
- ✅ Remaining requests return 429 (Too Many Requests)
- ✅ Rate limit window: 60 seconds

**If rate limiting not enabled**:
⚠️ WARNING: No rate limiting detected

**Implementation**: See Phase 1 roadmap (ADR-011)

#### 2.4 Authentication Bypass Attempts

**Test**: `test_authentication_bypass_attempts`

**Attack Vectors**:
- No auth header
- Empty API key
- SQL injection in API key (`' OR '1'='1`)
- Path traversal (`../../../etc/passwd`)
- JWT none algorithm
- Header case manipulation
- Multiple auth headers

**Expected Behavior**:
- ✅ All attempts return 401 Unauthorized
- ❌ FAIL: 200 OK without valid auth

**If auth not enabled**:
⚠️ WARNING: Authentication may not be enabled

#### 2.5 Path Traversal Attempts

**Test**: `test_path_traversal_attempts`

**Attack Vectors**:
- `/../../../etc/passwd`
- URL-encoded: `..%2f..%2f..%2fetc%2fpasswd`
- Double-encoded: `....//....//etc/passwd`
- Unicode bypass

**Expected Behavior**:
- ✅ All return 404 or 403
- ❌ CRITICAL: 200 OK + file contents

#### 2.6 Resource Exhaustion

**Test**: `test_resource_exhaustion_attacks`

**Attack Vectors**:
- Deep nested JSON (10,000 levels)
- ReDoS patterns (regex DoS)
- Billion laughs (exponential expansion)

**Expected Behavior**:
- ✅ Respond within 3 seconds
- ❌ CRITICAL: Server hang/timeout

---

### 3. Penetration Testing

**File**: `tests/security/pentest_scenarios.rs`

**Purpose**: Real-world attack scenarios

#### 3.1 CORS Policy

**Test**: `test_cors_policy`

**Checks**:
- ✅ CORS headers restrictive (not `*`)
- ✅ Preflight requests handled
- ✅ Cross-origin requests blocked

**Security Best Practice**:
```rust
// Only allow specific origins
Access-Control-Allow-Origin: https://neoland.example.com
```

⚠️ **Anti-pattern**:
```rust
Access-Control-Allow-Origin: *  // Allows all origins
```

#### 3.2 Information Disclosure

**Test**: `test_information_disclosure`

**Endpoints Tested**:
- `/health` (should be public)
- `/metrics` (should be protected)
- `/debug` (should not exist)
- `/.git/config` (should 404)
- `/.env` (should 404)
- `/swagger.json` (may be public, check for sensitive info)

**Sensitive Patterns Detected**:
- API keys
- Secrets/passwords
- Tokens
- Stack traces
- File paths (`/home/`, `/root/`)
- Environment variables

**Expected Behavior**:
- ✅ `/health` returns 200 (minimal info)
- ✅ All other endpoints return 404 or require auth
- ✅ No sensitive data in responses

#### 3.3 TLS Configuration

**Test**: `test_tls_configuration`

**Checks**:
- ✅ HTTP connections rejected
- ✅ HTTPS enforced
- ✅ TLS 1.3 preferred
- ❌ TLS 1.0/1.1 disabled

**Note**: Requires HTTPS endpoint for full testing

**Manual Testing**:
```bash
# Check TLS version
openssl s_client -connect neoland.example.com:443 -tls1_2
openssl s_client -connect neoland.example.com:443 -tls1_3

# Scan SSL configuration
nmap --script ssl-enum-ciphers -p 443 neoland.example.com
```

#### 3.4 Session Management

**Test**: `test_session_management`

**Checks**:
- ✅ Server doesn't accept external session IDs (session fixation)
- ✅ Cookies have secure flags (if used)
- ✅ Session cookies rejected on manipulation

**NEOLAND Status**: Stateless (no sessions) ✅

#### 3.5 File Upload Security

**Test**: `test_file_upload_security`

**Attack Vectors**:
- PHP shell (`shell.php`)
- Windows executable (`exploit.exe`)
- Shell script (`script.sh`)
- Extremely large file (100MB)
- Path traversal filename (`../../etc/passwd`)

**Expected Behavior**:
- ✅ All malicious files rejected
- ✅ File size limits enforced
- ✅ File type validation
- ❌ CRITICAL: Malicious file accepted

#### 3.6 gRPC Security

**Test**: `test_grpc_security`

**Checks**:
- ✅ mTLS required for connections
- ✅ Invalid metadata rejected
- ✅ Message validation

**Expected Behavior**:
- ✅ Connections without mTLS rejected
- ⚠️ WARNING: gRPC accepts plaintext

#### 3.7 Timing Attack Resistance

**Test**: `test_timing_attack_resistance`

**Purpose**: Prevent API key enumeration via timing side-channel

**Method**:
1. Measure response time for valid API key (10 requests)
2. Measure response time for invalid API key (10 requests)
3. Calculate average difference

**Expected Behavior**:
- ✅ Time difference < 100ms (constant-time comparison)
- ⚠️ WARNING: Time difference > 100ms (timing attack possible)

**Mitigation**:
```rust
use subtle::ConstantTimeEq;

fn validate_api_key(provided: &str, expected: &str) -> bool {
    provided.as_bytes().ct_eq(expected.as_bytes()).into()
}
```

#### 3.8 SSRF Protection

**Test**: `test_ssrf_protection`

**Attack Vectors**:
- Internal services: `http://localhost:22`
- Loopback: `http://127.0.0.1:22`
- IPv6 loopback: `http://[::]:22`
- Cloud metadata: `http://169.254.169.254/latest/meta-data/`
- File protocol: `file:///etc/passwd`
- Gopher protocol: `gopher://localhost:11211`

**Expected Behavior**:
- ✅ LLM doesn't fetch internal resources
- ✅ No file:// or gopher:// protocols
- ❌ CRITICAL: Response contains `/etc/passwd`, metadata, etc.

---

## Security Test Reports

All test results are saved to: `target/security-reports/`

### Report Files

```
target/security-reports/
├── cargo-audit.json              # Dependency vulnerabilities
├── cargo-deny.log                # License compliance
├── clippy-security.log           # Security lints
├── fuzz_rest_api_invalid_json.log
├── test_rate_limit_enforcement.log
├── test_authentication_bypass_attempts.log
├── test_path_traversal_attempts.log
├── test_cors_policy.log
├── test_information_disclosure.log
├── test_timing_attack_resistance.log
└── test_ssrf_protection.log
```

### Viewing Reports

```bash
# View all vulnerabilities
cat target/security-reports/cargo-audit.json | jq

# View failed tests
grep "FAIL" target/security-reports/*.log

# View warnings
grep "WARNING" target/security-reports/*.log
```

---

## CI/CD Integration

### GitHub Actions Workflow

**File**: `.github/workflows/security.yml` (to be created)

```yaml
name: Security Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  schedule:
    # Run daily at 2 AM UTC
    - cron: '0 2 * * *'

jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install security tools
        run: |
          cargo install cargo-audit
          cargo install cargo-deny

      - name: Run static security analysis
        run: |
          cargo audit
          cargo deny check
          cargo clippy -- -W clippy::security

      - name: Build project
        run: cargo build --release

      - name: Start server
        run: cargo run --release --bin neoland -- server &

      - name: Wait for server
        run: sleep 10

      - name: Run security tests
        run: ./tests/security/run_security_tests.sh

      - name: Upload security reports
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: security-reports
          path: target/security-reports/
```

---

## Security Compliance Matrix

### OWASP Top 10 (2021) Coverage

| Vulnerability | Test Coverage | Status |
|---------------|---------------|--------|
| **A01: Broken Access Control** | `test_authentication_bypass_attempts` | ✅ |
| **A02: Cryptographic Failures** | `test_tls_configuration` | ✅ |
| **A03: Injection** | `fuzz_rest_api_invalid_json`, SQL/XSS/Command injection | ✅ |
| **A04: Insecure Design** | Architecture review (manual) | 🔄 |
| **A05: Security Misconfiguration** | `test_information_disclosure`, `test_cors_policy` | ✅ |
| **A06: Vulnerable Components** | `cargo-audit` | ✅ |
| **A07: Auth Failures** | `test_authentication_bypass_attempts`, `test_timing_attack_resistance` | ✅ |
| **A08: Data Integrity** | `test_session_management` | ✅ |
| **A09: Logging Failures** | Audit logging (Phase 1) | ✅ |
| **A10: SSRF** | `test_ssrf_protection` | ✅ |

**Coverage**: 10/10 ✅

---

## Troubleshooting

### Issue 1: Server Not Running

**Error**:
```
⚠️  WARNING: Server not responding at http://localhost:3001
```

**Solution**:
```bash
# Start server manually
cargo run --release --bin neoland -- server &

# Verify it's running
curl http://localhost:3001/health
```

### Issue 2: cargo-audit Not Found

**Error**:
```
⏭️  SKIP: cargo-audit not installed
```

**Solution**:
```bash
cargo install cargo-audit
```

### Issue 3: Tests Timeout

**Error**:
```
❌ SECURITY ISSUE: Server hung on payload
```

**Root Causes**:
- Server processing very large payload
- Infinite loop in parsing logic
- Resource exhaustion

**Debug**:
```bash
# Monitor server logs
cargo run --bin neoland -- server 2>&1 | tee server.log

# Check resource usage
top -p $(pgrep neoland)

# Profile with perf
cargo build --release --bin neoland
perf record -g ./target/release/neoland server
```

### Issue 4: Rate Limit Not Working

**Error**:
```
⚠️  WARNING: No rate limiting detected
```

**Solution**: Implement rate limiting (see Phase 1 roadmap)

```rust
use tower_http::limit::RateLimitLayer;

let app = Router::new()
    .route("/v1/chat/completions", post(handle_rest_chat))
    .layer(RateLimitLayer::new(100, Duration::from_secs(60)));
```

### Issue 5: Vulnerabilities Found

**Error**:
```
❌ FAIL: Vulnerabilities detected
```

**Solution**:
```bash
# View details
cargo audit

# Update dependencies
cargo update

# If vulnerability persists, check for patches
cargo audit --ignore RUSTSEC-YYYY-XXXX  # Temporary ignore
```

---

## Adding New Security Tests

### 1. Fuzzing Test Template

```rust
#[tokio::test]
#[ignore]
async fn test_new_attack_vector() {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:3001";

    let payloads = vec![
        // Add malicious payloads
    ];

    for payload in payloads {
        let result = timeout(
            Duration::from_secs(5),
            client.post(format!("{}/endpoint", base_url))
                .body(payload)
                .send()
        ).await;

        match result {
            Ok(Ok(resp)) => {
                assert!(resp.status().is_client_error());
            }
            Ok(Err(_)) => { /* Connection error OK */ }
            Err(_) => {
                panic!("Server hung on payload");
            }
        }
    }
}
```

### 2. Penetration Test Template

```rust
#[tokio::test]
#[ignore]
async fn test_new_vulnerability() {
    println!("🧪 Testing new vulnerability");

    // Setup
    let client = reqwest::Client::new();

    // Attack
    let response = client.get("http://localhost:3001/endpoint").send().await;

    // Verify
    match response {
        Ok(resp) => {
            // Check response doesn't leak sensitive data
            assert_ne!(resp.status(), 200);
        }
        Err(_) => { /* Expected */ }
    }
}
```

### 3. Add to Test Runner

Edit `run_security_tests.sh`:

```bash
FUZZ_TESTS=(
    "fuzz_rest_api_invalid_json"
    "test_new_attack_vector"  # Add here
)
```

---

## Security Testing Roadmap

### Phase 2.4: Current Status (In Progress)

- [x] Create fuzzing test infrastructure
- [x] Create penetration test infrastructure
- [x] Configure cargo-audit
- [x] Configure cargo-deny
- [x] Create test runner script
- [x] Create comprehensive documentation
- [ ] Run initial security scan
- [ ] Fix identified vulnerabilities
- [ ] Achieve 100% pass rate

### Phase 2.4: Pending Tasks

1. **Run Initial Scan** (1h)
   - Execute `./tests/security/run_security_tests.sh`
   - Document all findings

2. **Fix Vulnerabilities** (6-8h)
   - Address cargo-audit findings
   - Fix clippy security lints
   - Implement missing protections

3. **Implement Missing Protections** (4-6h)
   - Rate limiting (if not yet done)
   - Authentication (if not yet done)
   - Input validation hardening
   - CORS policy configuration

4. **Re-run and Validate** (2h)
   - All tests passing
   - Security score 95/100+
   - Zero critical vulnerabilities

---

## Related Documentation

- **Production Roadmap**: `docs/PRODUCTION_ROADMAP.md`
- **Progress Tracker**: `docs/PROGRESS.md`
- **Security Architecture**: `SECURITY.md` (Phase 6)
- **Audit Logging**: `src/audit.rs`
- **Authentication**: `src/server/mod.rs` (Phase 1)

---

**Last Updated**: 2026-01-31
**Maintainer**: voidnx team
**Status**: Phase 2.4 - Security Testing (In Progress)
**Next**: Phase 2.5 - Load Testing
