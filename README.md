# Neoland - AI Agent Platform

**Status**: v0.1.0 — Beta | Security-Hardened | NixOS/Hyprland Integrated

Neoland is a terminal-based AI assistant built in Rust, designed as a foundational component of a larger AI agent ecosystem. It features a modern async TUI, multi-agent DSPy pipeline, comprehensive security hardening, and deep NixOS integration.

**Version**: 0.1.0
**Production Readiness**: 96/100

- ✅ Security Hardening — Auth, RBAC, Vault, Audit, Rate Limiting
- ✅ Testing — 198 Rust unit tests + 24 Python contract tests (0 mocks)
- ✅ CI/CD — GitHub Actions + pre-commit hooks (fmt, clippy, test)
- ✅ Multi-agent DSPy pipeline — Junior → Senior → Architect → TechLeader
- ✅ Observability — Prometheus metrics, OpenTelemetry spans, Swagger UI at `/swagger-ui/`
- ✅ IPC — mmap SharedFlags (64 bytes, 1 cache line) + NATS JetStream
- ✅ ADR Ledger — Merkle chain + secp256k1 signatures + at-least-once delivery
- ✅ EDR rules — 3 SIGMA + 6 YARA rules (agent pipeline anomaly detection)
- ✅ TUI — async tokio::select!, Tokyo Night, readline cursor, word-nav, braille spinner

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
- ✅ **SOPS Dev Workflow**: Preferred for encrypted, versioned local secrets, while env vars and local untracked files remain valid alternatives
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

**Test Suite**: 198 Rust unit tests + 24 Python contract tests — zero mocks

| Suite | Count | Notes |
|-------|-------|-------|
| Rust lib (`cargo test --lib`) | 198 | includes TUI, auth, agents, metrics, openapi |
| Python contract (`pytest -m contract`) | 24 | AgentFlags IPC + all Pydantic schemas, no LLM required |
| adr-ledger | 15 | Merkle chain, JetStream, signers |

**Coverage**: ~75% (target 80%)

**Run Tests**:

```bash
# Rust — full suite
nix develop --command cargo test --lib

# Python — contract tests (no LLM needed)
cd agents && pytest tests/ -m contract -v

# Python — integration tests (requires LLM_API_KEY)
cd agents && pytest tests/ -m integration -v
```

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
neoland-doctor --json

# Or run one-shot commands without opening a shell
nix develop --command neoland-server
nix develop --command neoland-test --json

# Build release binary
cargo build --bin neoland --release

# Run TUI client
./target/release/neoland client --ml-api-url http://localhost:9000
```

### NixOS Integration

```nix
# configuration.nix
imports = [ /home/kernelcore/master/neoland/modules/applications/neoland.nix ];

services.neoland = {
  enable = true;
  openFirewall = true;
  environmentFile = "/run/secrets/neoland.env";
};
```

The module configures:

- `systemd.services.neoland` with `mkIf cfg.enable`
- system-managed state, cache, runtime, and log directories
- `AUDIT_LOG_PATH`, DSPy URL, and server port wiring via Nix config
- `environmentFile` for secrets like `DATABASE_URL` and API keys

If you vendor the module into your own system config repo, switch the import to a relative path.

---

## 🔧 CLI Usage

### Server Mode

```bash
# Start gRPC (50051) + REST (3001) server
neoland server

# In nix develop, the shortcut is a real executable too
neoland-server
nix develop --command neoland-server

# Custom ports
neoland server --grpc-port 50052 --rest-port 3002
```

### Client Mode (TUI)

```bash
# Connect to local server
neoland client

# In nix develop, the shortcut is a real executable too
neoland-client
nix develop --command neoland-client --ml-api-url http://localhost:8080

# Custom endpoints
neoland client \
  --server-url http://[::1]:50051 \
  --ml-api-url http://localhost:8080
```

### SOPS Workflow

For local development, this repo can keep encrypted dotenv secrets in
`secrets/neoland.sops.env` and decrypt them only when launching the process.

Users can also provide secrets through shell environment variables, local
untracked env files, or external secret stores. `SOPS` is the preferred option
when you want stronger handling for versioned secrets, but it is not mandatory.

See: [`docs/SOPS_SETUP.md`](docs/SOPS_SETUP.md)

**Key Features**:

- Braille spinner animation while thinking
- Tokyo Night color scheme
- readline-style cursor (Ctrl+←/→ word nav, Ctrl+W/U/K kill)
- Auto-scroll + manual scroll ↑/↓/PgUp/PgDn
- 5 preset profiles (Ctrl+1-5)

### Health Checks

```bash
# Run diagnostics
neoland test

# Machine-readable output for scripts/CI
neoland test --json
neoland doctor --json
nix develop --command neoland-test --json
nix develop --command neoland-doctor --json
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
**Last Updated**: 2026-04-26
