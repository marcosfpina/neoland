# Neoland - Enterprise AI Agent Platform

**Status**: Production-Ready | Security-Hardened | NixOS/Hyprland Integrated

Neoland is an enterprise-grade, terminal-based AI assistant built in Rust, designed as a foundational component of a larger AI agent ecosystem. It features a sleek TUI interface, comprehensive security hardening, robust fallback mechanisms, and deep OS integration.

**Production Readiness**: 58/100
- ✅ Security Hardening Complete (85%)
- ✅ Testing Coverage (65% - 73 tests)
- ⏳ CI/CD Pipeline (Planned)
- ⏳ Operational Readiness (Planned)

---

## 🎯 Architecture Overview

### Core Components

```
neoland (Unified CLI Binary)
├── server     - gRPC + REST API server
├── client     - Modern TUI (ratatui + crossterm)
├── test       - Health check suite
└── restart    - Process management
```

### Module Structure

```
src/
├── bin/neoland.rs         # Unified CLI entrypoint
├── lib.rs                 # Library root
├── cli.rs                 # Argument parsing (clap)
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

**Security Posture**: 85% hardened ✅

See: `docs/AUTHENTICATION.md`, `docs/VAULT_SETUP.md`, `docs/ADR/`

---

## 🧪 Testing & Quality

**Test Suite**: 73 tests (100% passing)
- **Unit Tests**: 55 tests (~50-55% code coverage)
- **Integration Tests**: 18 tests (REST + gRPC)

**Coverage by Module**:
- ✅ 100%: validation.rs, test_utils.rs
- ✅ High: auth.rs, audit.rs
- ⚠️ Medium: secrets.rs, llm modules

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
cd /home/kernelcore/arch/neoland
nix develop

# Build release binary
cargo build --bin neoland --release

# Run TUI client
./target/release/neoland client --ml-api-url http://localhost:9000
```

### NixOS Integration

```nix
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

# Custom ports
neoland server --grpc-port 50052 --rest-port 3002
```

### Client Mode (TUI)

```bash
# Connect to local server
neoland client

# Custom endpoints
neoland client \
  --server-url http://[::1]:50051 \
  --ml-api-url http://localhost:8080
```

**Key Features**:

- Visual "Thinking..." status indicator
- Tokyo Night color scheme
- Vim-style navigation (j/k scroll)
- 5 preset profiles (Ctrl+1-5)

### Health Checks

```bash
# Run diagnostics
neoland test
```

---

## 🏗️ Architectural Decisions

All major architectural decisions are documented in [**Architecture Decision Records (ADR)**](file:///home/kernelcore/arch/neoland/docs/ADR.md).

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

- [x] Unified CLI (replaces 4 shell scripts)
- [x] TUI client (production-ready)
- [x] Configurable endpoints
- [x] NixOS declarative config
- [x] Agent Hub integration

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

**See Also**: [`docs/ADR.md#ADR-007`](file:///home/kernelcore/arch/neoland/docs/ADR.md) for architectural decisions

---

### Strategic Roadmap (Q2-Q3 2026)

#### ADR-Ledger Integration (Q2 2026)

**Objective**: Intelligent governance for architecture decisions

**Key Features**:

- Auto-tracking of critical decisions (routing changes, provider switches)
- Semantic search over historical ADRs via embeddings
- CI/CD compliance validation against documented ADRs
- Impact analysis when modifying architecture

**Value**: Governance automation + knowledge discovery

**See**: [`docs/ADR.md#ADR-008`](file:///home/kernelcore/arch/neoland/docs/ADR.md#ADR-008)

---

#### Neutron - Autonomous Low-Level Security (Q2 2026)

**Objective**: Independent threat neutralization operating autonomously

**Core Capabilities**:

- **Provenance Verification**: Cryptographic validation of model/data origins
- **Syscall Interception**: eBPF-based anomaly detection (network, file I/O)
- **Autonomous Response**: Self-healing without human intervention (quarantine, snapshot, rollback)
- **Zero-Trust**: Verifies everything, trusts nothing

**Architecture**: Operates **independently** of Neoland (stateless, low-level kernel/syscall layer)

**Threat Protection**:

- Supply chain attacks (binary verification via SHA-256)
- Model poisoning (output anomaly detection)
- Memory corruption (eBPF memory monitoring)

**Value**: Protection against low-level attacks that bypass application security

**See**: [`docs/ADR.md#ADR-009`](file:///home/kernelcore/arch/neoland/docs/ADR.md#ADR-009)

---

#### Spectre - Enterprise Observability & Scale (Q3 2026)

**Objective**: Enterprise-ready features to unlock Fortune 500 market + revenue

**Enterprise Features**:

- **Multi-Tenancy**: Resource quotas, cost attribution per tenant, SLA tiers (Gold/Silver/Bronze)
- **Distributed Tracing**: OpenTelemetry → SIEM integration (Datadog/Splunk)
- **Compliance Automation**: SOC2/ISO27001/GDPR automated validation
- **HA/Multi-Region**: 99.99% SLA with global deployment

**Business Model**: SaaS pricing ($99-$999+/month based on tier)

**Value Proposition for Investors**:

- 📈 Recurring revenue stream (SaaS)
- 🏢 Fortune 500 addressable market
- 🔒 Vendor lock-in via proprietary observability
- 🌍 International expansion via multi-region

**See**: [`docs/ADR.md#ADR-010`](file:///home/kernelcore/arch/neoland/docs/ADR.md#ADR-010)

---

## 📊 Performance Characteristics

| Metric               | Value            |
| -------------------- | ---------------- |
| TUI Startup          | <50ms            |
| Memory (TUI)         | ~15MB            |
| Memory (Server)      | ~200MB (idle)    |
| Qwen 1.8B Inference  | 5-10 tok/s (CPU) |
| Build Time (release) | ~10s             |

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

## 🐛 Known Issues

1. **Legacy Scripts**: `run-*.sh` still present (marked for removal)
2. **Dead Code**: `engine.rs` and `nlp.rs` have unused structs (future RAG feature)
3. **Warnings**: Unused imports in `securellm-core` (external crate)

---

## 📚 Documentation

- **[`docs/ADR.md`](file:///home/kernelcore/arch/neoland/docs/ADR.md)**: Architecture Decision Records (NEW)
- **[`ARCHITECTURE.md`](file:///home/kernelcore/arch/neoland/ARCHITECTURE.md)**: System architecture overview
- **[`QUICKSTART.md`](file:///home/kernelcore/arch/neoland/QUICKSTART.md)**: Legacy quick reference
- **[`UX_IMPROVEMENTS.md`](file:///home/kernelcore/arch/neoland/UX_IMPROVEMENTS.md)**: UI/UX design rationale

---

## 🤝 Contributing

This is part of a larger research project. External contributions are not currently accepted.

---

## 📜 License

Proprietary - Internal Research Project

---

**Maintained by**: VoidNxSEC Team  
**Last Updated**: 2026-01-18
