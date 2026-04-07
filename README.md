# Neoland - AI Agent Platform

**Status**: Active Development | Security-Hardened | NixOS/Hyprland Integrated

Neoland is a terminal-based AI assistant built in Rust, designed as a foundational component of a larger AI agent ecosystem. It features a TUI interface, comprehensive security hardening, robust fallback mechanisms, and deep OS integration.

**Version**: 0.1.0
**Production Readiness**: 65/100

- ✅ Security Hardening (Phase 1 Complete)
- ✅ Testing: 140 tests (114 passing, 26 ignored/require external services)
- ✅ CI/CD Pipeline (GitHub Actions + Pre-commit Hooks)
- ✅ Config file support (neoland.toml + env overrides)
- ✅ Prometheus metrics wired (HTTP, gRPC, LLM, Auth, Circuit Breakers)
- ✅ TUI: connection status, session stats, streaming display
- ⚠️ Coverage: \~60% (Target: 70%)
- ⏳ Operational Readiness (In Progress — see docs/ROADMAP.md)

---

## 🎯 Architecture Overview

### Core Components

```javascript
neoland (Unified CLI Binary)
├── server     - gRPC + REST API server
├── client     - Modern TUI (ratatui + crossterm)
├── test       - Health check suite
├── restart    - Process management
└── doctor     - Environment diagnostics
```

### Module Structure

```javascript
src/
├── bin/neoland.rs         # Unified CLI entrypoint
├── lib.rs                 # Library root
├── cli.rs                 # Argument parsing (clap)
├── config.rs              # Config loading (neoland.toml → env → CLI)
├── server/mod.rs          # gRPC/REST server with security middleware
├── engine.rs              # Local inference (Qwen 1.8B)
├── nlp.rs                 # Vector store (RAG)
├── auth.rs                # ✨ Authentication & RBAC (Phase 1.1)
├── secrets.rs             # ✨ HashiCorp Vault integration (Phase 1.2)
├── audit.rs               # ✨ Audit logging & brute force detection (Phase 1.3)
├── validation.rs          # ✨ Input validation & sanitization (Phase 1.4)
├── test_utils.rs          # ✨ Test utilities & mocks (Phase 2.1)
├── tui/                   # Terminal UI
│   ├── mod.rs            # Event loop & message handling
│   ├── app.rs            # Application state
│   ├── ui.rs             # Rendering (Tokyo Night theme)
│   ├── events.rs         # Keyboard/mouse input
│   └── presets.rs        # Pre-configured inference profiles
├── ml_offload/           # External ML API client
│   ├── mod.rs
│   ├── client.rs         # HTTP client (OpenAI-compatible)
│   └── models.rs         # Data structures
├── llm/                  # SecureLLM integration
│   ├── mod.rs
│   ├── proxy.rs          # Security proxy layer
│   └── unified_client.rs # Unified LLM client
└── hyprland_ops.rs       # Window manager IPC
```

**✨ New in Production Readiness Phase**: Security hardening modules (auth, secrets, audit, validation)

---

## 🔐 Security Features

Neoland implements enterprise-grade security hardening (Phase 1 Complete):

### Authentication & Authorization (ADR-011)

- ✅ **REST API Key Authentication**: X-API-Key header validation
- ✅ **Role-Based Access Control (RBAC)**: 3 roles (Admin, User, ReadOnly)
- ✅ **Hierarchical Permissions**: Admin > User > ReadOnly
- ✅ **Development Keys**: Pre-configured for quick start
- ⏳ **gRPC mTLS**: Planned for Phase 1.5

### Secrets Management (ADR-012)

- ✅ **HashiCorp Vault Integration**: Production-grade secrets storage
- ✅ **Three-Tier Retrieval**: Cache (30s) → Vault → Environment Variables
- ✅ **Automatic Fallback**: Graceful degradation on Vault unavailability
- ✅ **SOPS Dev Workflow**: Encrypted dotenv secrets can be decrypted only at process launch
- ✅ **Secret Types**: LLM API keys, NEOLAND API keys, DB credentials, TLS certs
- ✅ **Performance**: <1ms cache hit, 50-100ms Vault read

### Audit Logging (ADR-013)

- ✅ **Structured JSON Events**: Immutable append-only logs
- ✅ **15 Action Types**: Auth, secrets, API, config, admin operations
- ✅ **Automatic Sanitization**: PII/credentials redacted from logs
- ✅ **Brute Force Detection**: >5 failed auth in 1 minute triggers alert
- ✅ **Alert System**: Pluggable handlers (Console, future: email, Slack)
- ✅ **Compliance Ready**: SOC 2, GDPR, ISO 27001 compatible

### Rate Limiting & Input Validation (ADR-014)

- ✅ **Rate Limiting**: 100 requests/minute per user/IP
- ✅ **Input Validation**: Max 100KB prompt, 100 messages, 1MB request
- ✅ **Sanitization**: Null byte removal, control character filtering
- ✅ **Path Traversal Prevention**: Secure document upload
- ✅ **DoS Protection**: Size limits prevent memory exhaustion

**Security Posture**: Phase 1 Complete (RBAC + Vault + Audit + Rate Limiting)

See: `docs/AUTHENTICATION.md`, `docs/VAULT_SETUP.md`, `docs/SOPS_SETUP.md`, `docs/ADR/`

---

## Testing & Quality

**Test Suite**: 118 total tests (98 passing, 20 ignored)

- **Unit Tests**: 82 tests (77 passing, 5 ignored - require PostgreSQL/ml-offload)
- **Integration Tests**: 19 tests (18 passing, 1 ignored)
- **Security Tests**: 14 tests (all ignored - require `--ignored` flag)
- **Storage Tests**: 2 tests (ignored - require PostgreSQL + pgvector)
- **Benchmark Suite**: Compiles, not yet baselined

**Coverage by Module** (\~60%):

- ✅ High: validation.rs (11), auth.rs (11), audit.rs (14), health.rs (9)
- ✅ Medium: logging.rs (8), secrets.rs (5), metrics.rs (5), test\_utils.rs (8)
- ⚠️ Low: llm/proxy.rs (4), nlp.rs (1)
- ❌ None: engine.rs (0 tests - core inference engine)

**Run Tests**:

```bash
# All tests
nix develop -c cargo test

# Unit tests only
nix develop -c cargo test --lib

# Integration tests
nix develop -c cargo test --tests
```

See: `docs/TESTING.md`

---

## 🚀 Quick Start

### Installation

Neoland uses Nix for reproducible builds:

```bash
# Enter development environment
cd /home/kernelcore/master/neoland
nix develop

# Inside the dev shell
neoland-secrets
neoland-server
neoland-client --ml-api-url http://localhost:9000

# Build release binary
cargo build --bin neoland --release

# Run TUI client
./target/release/neoland client --ml-api-url http://localhost:9000
```

### NixOS Integration

```javascript
# /etc/nixos/configuration.nix
imports = [ ./modules/applications/neoland.nix ];

services.neoland.enable = true;
```

This enables:

- Hyprland scratchpad (Super+N toggle)
- Agent Hub launcher integration
- Declarative window rules

---

## 🔧 CLI Usage

### Server Mode

```bash
# Start gRPC (50051) + REST (3001) server
neoland server

# In nix develop, the alias is shorter
neoland-server

# Custom ports
neoland server --grpc-port 50052 --rest-port 3002
```

### Client Mode (TUI)

```bash
# Connect to local server
neoland client

# In nix develop, the alias is shorter
neoland-client

# Custom endpoints
neoland client \
  --server-url http://[::1]:50051 \
  --ml-api-url http://localhost:8080
```

### SOPS Workflow

For local development, this repo can keep encrypted dotenv secrets in
`secrets/neoland.sops.env` and decrypt them only when launching the process.

See: [`docs/SOPS_SETUP.md`](docs/SOPS_SETUP.md)

**Key Features**:

- Visual "Thinking..." status indicator
- Tokyo Night color scheme
- Vim-style navigation (j/k scroll)
- 5 preset profiles (Ctrl+1-5)

### Health Checks

```bash
# Run diagnostics
neoland test

# Machine-readable output for scripts/CI
neoland test --json
neoland doctor --json
```

---

## 🏗️ Architectural Decisions

All major architectural decisions are documented in [**Architecture Decision Records (ADR)**](docs/ADR.md).

**Key Decisions**:

- **ADR-001**: Arquitetura de 3 Camadas (Infra/Security/Compliance)
- **ADR-002**: Estratégia LocalFirst para Roteamento LLM
- **ADR-003**: SecureLLM Proxy com Factory Pattern
- **ADR-004**: Connection Pooling para Baixa Latência
- **ADR-007**: Neutron Integration (Planned Q2 2026)

### 1. TUI over GTK4

**Rationale**:

- 50ms startup vs 2-3s for GTK
- 15MB memory vs 300MB
- Native terminal integration (tmux/zellij)
- Better fit for scratchpad UX

**Implementation**: Replaced legacy GTK client with `ratatui` + `crossterm`.

### 2. LLM Fallback Chain

**Primary** → **Secondary** → **Tertiary**

1. **ml-offload-api** (Port 9000): GPU-accelerated, multi-backend orchestrator
2. **gRPC Internal**: Local Qwen 1.8B (CPU fallback)
3. **SecureLLM Proxy**: External providers (DeepSeek/Claude) with audit logs

**Configuration**: `--ml-api-url` flag makes endpoint configurable.

### 3. Module Refactoring

**Before**:

```rust
// Anti-pattern: include! macro
pub mod server {
    include!("server.rs");
}
```

**After**:

```rust
// Idiomatic: proper module hierarchy
pub mod server;  // References src/server/mod.rs
```

**Impact**: Improved compilation times, better LSP support.

### 4. State Management

Added `is_thinking` flag to `AppState` for real-time UI feedback:

```rust
pub struct AppState {
    pub is_thinking: bool,  // Toggles "⏳ Thinking..." in header
    // ...
}
```

---

## 🔌 Integrations

### ml-offload-api

External service providing:

- Backend routing (Ollama/vLLM/llama.cpp)
- GPU acceleration
- OpenAI-compatible API

**Usage**: Pass `--ml-api-url` to client.

### securellm-bridge

Security layer for external LLM providers:

- Rate limiting
- Audit logging
- API key rotation

**Path dependency**: `../securellm-bridge/crates/core`

### intelagent-core (Phantom)

Task orchestration framework:

- Multi-step reasoning
- Tool execution
- Memory management

**Path dependency**: `../phantom/intelagent/crates/core`

### Hyprland IPC

Window management via `hyprland-ipc` crate:

- Scratchpad toggle
- Floating window rules
- Opacity control

**Path dependency**: `../ai-agent-os/crates/hyprland-ipc`

---

## 🛣️ Roadmap

### Current Status ✅

- Unified CLI (replaces 4 shell scripts)
- TUI client (production-ready)
- Configurable endpoints
- NixOS declarative config
- Agent Hub integration

### Next Phase 🚧

#### Neutron Integration

**Objective**: Supply chain and distributed data trust layer.

**Planned Features**:

- Cryptographic verification of training data provenance
- Distributed consensus for model updates
- Tamper-proof audit logs for LLM interactions
- Zero-knowledge proofs for sensitive inference
  **Timeline**: Implementation after Neutron v1.0 release (Q2 2026)

**Architecture**:

```rust
// Future module structure
src/
└── neutron/
    ├── provenance.rs    # Data lineage tracking
    ├── consensus.rs     # Distributed verification
    └── zkp.rs          # Zero-knowledge layer
```

**Integration Points**:

1. `ml-offload-api`: Verify backend attestations
2. `securellm-bridge`: Audit log immutability
3. `VectorStore`: Document source verification

**See Also**: [`docs/ADR.md#ADR-007`](docs/ADR.md) for architectural decisions

---

### Strategic Roadmap (Q2-Q3 2026)

> **Note**: The following features are in planning/early design phase. No code has been written for these yet.

#### Neutron (NEXUS Platform) - AI Compliance (Q2 2026)

**Status**: Separate project, PoC stage. See `neutron/` directory.

Kernel-level AI compliance enforcement using seccomp-BPF. Targeting EU AI Act high-risk deadline (Aug 2, 2026).

#### ADR-Ledger Integration (Q2 2026)

**Status**: Planned, 0% implemented.

Intelligent governance for architecture decisions with semantic search over ADRs.

---

## Performance Characteristics

| Metric               | Value            | Notes                                             |
| -------------------- | ---------------- | ------------------------------------------------- |
| TUI Startup          | <50ms            |                                                   |
| Memory (TUI)         | \~15MB           |                                                   |
| Memory (Server)      | \~200MB (idle)   |                                                   |
| Qwen 1.8B Inference  | 5-10 tok/s (CPU) | Candle backend; consider llama.cpp for production |
| Build Time (release) | \~10s            |                                                   |

**Note**: Local inference at 5-10 tok/s CPU is suitable for development/testing. For production workloads, use the ml-offload API backend (GPU-accelerated) or the SecureLLM cloud fallback. SLO targets (500 RPS, p99 <200ms) have not been validated yet.

---

## 🧪 Testing

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test test_grpc_chat_stream

# Checksum validation
cargo check --all-targets
```

---

## Known Issues

1. **Legacy Scripts**: `run-*.sh` still present (marked for removal)
2. **engine.rs has 0 tests**: Core inference engine has no unit test coverage
3. **In-memory Vector Store**: `nlp.rs` uses `Vec<Document>` - data lost on restart. Persistent store (`storage/vector_store.rs`) exists but requires PostgreSQL + pgvector setup
4. **Path Dependencies**: 5 path dependencies in Cargo.toml require sibling projects to build (see Cargo.toml comments for setup)
5. **Local inference performance**: 5-10 tok/s CPU via Candle is below production threshold; use ml-offload or SecureLLM fallback
6. **Lock contention risk**: `Arc<Mutex<>>` on engine/vector store may bottleneck above \~50 req/s
7. **Warnings**: Unused imports in `securellm-core` (external crate)
8. **SLO targets unvalidated**: 500 RPS / p99 <200ms targets have never been load-tested

---

## 📚 Documentation

- : Architecture Decision Records (NEW)
- : System architecture overview
- : Legacy quick reference
- : UI/UX design rationale

---

## Contributing

This is part of a larger research project. External contributions are not currently accepted.

---

## License

Proprietary - Internal Research Project

---

**Maintained by**: VoidNxSEC Team
**Last Updated**: 2026-02-15
