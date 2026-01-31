# NEOLAND: Testing Guide

**Last Updated**: 2026-01-31

This document provides comprehensive guidance on running and writing tests for NEOLAND.

---

## Test Suite Overview

### Current Test Coverage

**Total Tests**: 73 ✅
- **Unit Tests**: 55 (library tests)
- **Integration Tests**: 18 (API/gRPC tests)

**Coverage**: ~60-65% (Target: 70%+)

**Test Execution Time**:
- Unit tests: ~5s
- Integration tests: ~16s
- Total: ~21s

---

## Running Tests

### All Tests

```bash
# Run all tests (unit + integration)
nix develop -c cargo test

# Run with output
nix develop -c cargo test -- --nocapture

# Run with specific test threads
nix develop -c cargo test -- --test-threads=1
```

### Unit Tests Only

```bash
# Run all unit tests
nix develop -c cargo test --lib

# Run specific module tests
nix develop -c cargo test --lib auth::tests
nix develop -c cargo test --lib audit::tests
nix develop -c cargo test --lib validation::tests

# Run with coverage (requires cargo-tarpaulin)
nix develop -c cargo tarpaulin --lib --out Html
```

### Integration Tests Only

```bash
# Run all integration tests
nix develop -c cargo test --tests

# Run specific integration test file
nix develop -c cargo test --test rest_api_test
nix develop -c cargo test --test grpc_integration_test
nix develop -c cargo test --test grpc_test

# Run specific test
nix develop -c cargo test --test rest_api_test test_health_endpoint
```

### Ignored Tests

Some tests are marked as `#[ignore]` due to long runtime:

```bash
# Run ignored tests
nix develop -c cargo test -- --ignored

# Run specific ignored test
nix develop -c cargo test --test rest_api_test test_rate_limiting -- --ignored
```

---

## Test Organization

### Unit Tests (`src/`)

Unit tests are located in `#[cfg(test)]` modules within source files:

```
src/
├── auth.rs              # 11 tests - RBAC, API key management
├── audit.rs             # 15 tests - Event logging, sanitization
├── validation.rs        # 14 tests - Input validation
├── test_utils.rs        # 8 tests - Test utilities (self-testing)
├── secrets.rs           # 4 tests - Secrets management
├── llm/proxy.rs         # 4 tests - LLM proxy
├── llm/unified_client.rs # 1 test - Client creation
├── ml_offload/client.rs # 3 tests - API client
└── nlp.rs               # 1 test - Vector store (requires model download)
```

### Integration Tests (`tests/`)

```
tests/
├── rest_api_test.rs          # 10 tests - REST API endpoints
├── grpc_integration_test.rs  # 8 tests - gRPC services
└── grpc_test.rs              # 1 test - Basic gRPC streaming
```

---

## Test Categories

### 1. Authentication Tests

**Location**: `src/auth.rs`

**Coverage**:
- ✅ RBAC permission hierarchy
- ✅ API key validation
- ✅ Key lifecycle (add/revoke)
- ✅ Multiple users per role
- ✅ Vault integration

**Example**:
```bash
cargo test --lib auth::tests::test_role_hierarchy
```

### 2. Audit Logging Tests

**Location**: `src/audit.rs`

**Coverage**:
- ✅ Event creation and sanitization
- ✅ Brute force detection
- ✅ Alert system
- ✅ File logging
- ✅ Severity mapping

**Example**:
```bash
cargo test --lib audit::tests::test_failed_auth_tracker
```

### 3. Input Validation Tests

**Location**: `src/validation.rs`

**Coverage**:
- ✅ Prompt size limits (100KB)
- ✅ Message count limits (100)
- ✅ Input sanitization
- ✅ Path traversal prevention
- ✅ Role validation

**Example**:
```bash
cargo test --lib validation::tests
```

### 4. REST API Tests

**Location**: `tests/rest_api_test.rs`

**Coverage**:
- ✅ Authentication flow (valid/invalid keys)
- ✅ RBAC role testing
- ✅ Input validation integration
- ✅ Rate limiting (ignored test)
- ✅ Error responses

**Example**:
```bash
cargo test --test rest_api_test test_chat_endpoint_requires_auth -- --nocapture
```

**Port Configuration**:
- Test server uses ports 50053 (gRPC) and 3003 (REST)
- Tests run sequentially to avoid port conflicts

### 5. gRPC Integration Tests

**Location**: `tests/grpc_integration_test.rs`

**Coverage**:
- ✅ Chat streaming
- ✅ Vector store operations (add/search)
- ✅ Context injection
- ✅ Concurrent requests
- ✅ Error handling

**Example**:
```bash
cargo test --test grpc_integration_test test_grpc_search -- --nocapture
```

**Port Configuration**:
- Test server uses ports 50054 (gRPC) and 3004 (REST)

---

## Writing New Tests

### Unit Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_feature() {
        // Arrange
        let input = "test data";

        // Act
        let result = my_function(input);

        // Assert
        assert_eq!(result, expected_output);
    }

    #[tokio::test]
    async fn test_async_feature() {
        // For async functions
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

### Integration Test Template

```rust
use tokio::time::{sleep, Duration};

const TEST_GRPC_PORT: u16 = 50055; // Unique port
const TEST_REST_PORT: u16 = 3005;

async fn start_test_server() -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let _ = llamachat_poc::server::run_server(TEST_GRPC_PORT, TEST_REST_PORT).await;
    })
}

#[tokio::test]
async fn test_my_integration() {
    let server = start_test_server().await;
    sleep(Duration::from_millis(500)).await;

    // Your test logic here

    server.abort(); // Cleanup
}
```

### Using Test Utilities

```rust
use llamachat_poc::test_utils::{mocks, fixtures, assertions};

#[tokio::test]
async fn test_with_utilities() {
    // Use mock factories
    let secrets_manager = mocks::test_secrets_manager().await.unwrap();
    let auth_manager = mocks::test_auth_manager();

    // Use fixtures
    let api_key = fixtures::VALID_API_KEY;
    let test_ip = fixtures::TEST_IP;

    // Use custom assertions
    let result: Result<()> = Err(anyhow!("Test error"));
    assertions::assert_error_contains(&result, "Test error");
}
```

---

## Test Best Practices

### 1. Naming Conventions

```rust
// Good
#[test]
fn test_validate_prompt_success() { }

#[test]
fn test_validate_prompt_empty() { }

// Bad
#[test]
fn test1() { }

#[test]
fn validation_test() { }
```

### 2. Test Independence

```rust
// Good - self-contained
#[test]
fn test_feature() {
    let manager = create_test_manager();
    assert!(manager.do_something());
}

// Bad - depends on external state
static mut GLOBAL_STATE: i32 = 0;

#[test]
fn test_with_global() {
    unsafe { GLOBAL_STATE += 1; }
}
```

### 3. Cleanup

```rust
#[tokio::test]
async fn test_with_cleanup() {
    // Setup
    std::env::set_var("TEST_VAR", "value");

    // Test
    let result = function_using_env().await;

    // Cleanup
    std::env::remove_var("TEST_VAR");

    assert!(result.is_ok());
}
```

### 4. Ignored Tests

```rust
// Use for slow tests
#[tokio::test]
#[ignore]
async fn test_rate_limiting() {
    // This test takes 60+ seconds
    for _ in 0..105 {
        send_request().await;
    }
}
```

---

## Continuous Integration

### GitHub Actions (Planned - Phase 3)

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: cachix/install-nix-action@v20
      - run: nix develop -c cargo test --all
```

### Pre-commit Hook (Planned - Phase 3)

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Run tests before commit
nix develop -c cargo test --lib || exit 1
```

---

## Troubleshooting

### Port Conflicts

**Problem**: Tests fail with "address already in use"

**Solution**:
```bash
# Run tests sequentially
cargo test -- --test-threads=1

# Or kill existing processes
killall neoland
```

### Model Download Issues

**Problem**: `nlp.rs` tests fail with model download errors

**Solution**:
```bash
# Pre-download models
nix develop -c cargo test --lib nlp::tests -- --ignored

# Or run with internet access
# Models cached in ~/.cache/huggingface
```

### Slow Tests

**Problem**: Integration tests take too long

**Solution**:
```bash
# Run only unit tests
cargo test --lib

# Skip ignored tests
cargo test -- --skip test_rate_limiting
```

### Vault Connection Failures

**Problem**: Secrets tests fail with Vault connection errors

**Solution**:
```bash
# Tests fallback to environment variables
# Vault tests are optional
export NEOLAND_ADMIN_API_KEY="test_key"
cargo test --lib secrets::tests
```

---

## Test Metrics

### Coverage by Module

| Module | Tests | Coverage | Status |
|--------|-------|----------|--------|
| validation.rs | 14 | 100% | ✅ |
| audit.rs | 15 | High | ✅ |
| auth.rs | 11 | High | ✅ |
| test_utils.rs | 8 | 100% | ✅ |
| secrets.rs | 4 | Medium | ⚠️ |
| llm/proxy.rs | 4 | Low | ⚠️ |
| nlp.rs | 1 | Low | ⚠️ |
| engine.rs | 0 | None | ❌ |
| server/mod.rs | 0 | None | ❌ |

### Test Success Rate

**Current**: 73/73 passing (100%) ✅
**Target**: Maintain 100% pass rate

### Performance Benchmarks

| Test Suite | Time | Tests |
|------------|------|-------|
| Unit tests | ~5s | 55 |
| gRPC integration | ~6.4s | 9 |
| REST integration | ~8.1s | 10 |
| **Total** | **~21s** | **73** |

---

## Future Improvements

### Phase 2.3: Additional Tests
- [ ] E2E TUI automation tests
- [ ] Mock provider tests
- [ ] Security fuzzing tests
- [ ] Property-based tests

### Phase 2.4: Load Testing
- [ ] Stress testing (500 RPS target)
- [ ] Concurrency testing
- [ ] Memory leak detection
- [ ] Performance benchmarks

### Phase 3: CI/CD Integration
- [ ] Automated test runs on PR
- [ ] Coverage reporting
- [ ] Test result badges
- [ ] Performance regression detection

---

## References

- **Test Utilities**: `src/test_utils.rs`
- **ADR-014**: Rate Limiting & Input Validation
- **ADR-013**: Audit Logging
- **PROGRESS.md**: Overall testing progress

---

## Contributing

When adding new features:
1. Write tests first (TDD approach)
2. Aim for 70%+ coverage
3. Include both positive and negative test cases
4. Add integration tests for API changes
5. Update this document if adding new test categories

---

**Maintained By**: AI Assistant + kernelcore
**Last Test Run**: 2026-01-31
**Status**: 73/73 passing ✅
