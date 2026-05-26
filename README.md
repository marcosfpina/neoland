# Neoland - AI Agent Platform

**Status**: v0.1.0 — Integrated Pre-Release Beta | Security-Hardened | Nix/NixOS First

Neoland is an AI control-plane component built in Rust, Python, and TypeScript. It combines a code-task TUI, a Rust REST/gRPC control plane, a DSPy multi-agent pipeline, a Next.js operator workbench, and Nix-first runtime wiring for the larger AI agent ecosystem.

**Version**: 0.1.0
**Responsible Public Pre-Release Readiness**: 78/100

- ✅ Security Hardening — Auth, RBAC, Vault, Audit, Rate Limiting
- ✅ Testing — 226 Rust unit tests passing, 17 ignored + Python contract suite available
- ✅ CI/CD — GitHub Actions + pre-commit hooks (fmt, clippy, test)
- ✅ Multi-agent DSPy pipeline — Junior → Senior → Architect → TechLeader
- ✅ Observability — Prometheus metrics, OpenTelemetry spans, Swagger UI at `/swagger-ui/`
- ✅ IPC — mmap SharedFlags (64 bytes, 1 cache line) + NATS JetStream
- ✅ ADR Ledger — Merkle chain + secp256k1 signatures + at-least-once delivery
- ✅ EDR rules — 3 SIGMA + 6 YARA rules (agent pipeline anomaly detection)
- ✅ TUI — async tokio::select!, Tokyo Night, readline cursor, word-nav, braille spinner
- ⚠️ Release gates still open — full runtime smoke, DSPy doctor probe, session API hardening, frontend analytics guards, backup/restore ritual

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

See: `docs/neoland-authentication.md`, `docs/neoland-vault-setup.md`, `docs/neoland-sops-setup.md`, `docs/ADR/`

---

## Testing & Quality

**Test Suite**: 226 Rust unit tests passing + Python contract tests available

| Suite | Count | Notes |
|-------|-------|-------|
| Rust lib (`cargo test --lib --quiet`) | 226 passing, 17 ignored | includes TUI, auth, agents, metrics, openapi |
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
neoland-client --neoland-gateway-url http://localhost:8080
neoland-doctor --json

# Or run one-shot commands without opening a shell
nix develop --command neoland-server
nix develop --command neoland-test --json

# Build release binary
cargo build --bin neoland --release

# Run TUI client
./target/release/neoland client --neoland-gateway-url http://localhost:8080
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
nix develop --command neoland-client --neoland-gateway-url http://localhost:8080

# Custom endpoints
neoland client \
  --server-url http://[::1]:50051 \
  --neoland-gateway-url http://localhost:8080
```

### SOPS Workflow

For local development, this repo can keep encrypted dotenv secrets in
`secrets/neoland.sops.env` and decrypt them only when launching the process.

Users can also provide secrets through shell environment variables, local
untracked env files, or external secret stores. `SOPS` is the preferred option
when you want stronger handling for versioned secrets, but it is not mandatory.

See: [`docs/neoland-sops-setup.md`](docs/neoland-sops-setup.md)

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

Major architectural decisions are documented in [`docs/ADR/`](docs/ADR/) and summarized in [`docs/neoland-adr.md`](docs/neoland-adr.md).

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

1. **SecureLLM Bridge API** (Port 8080): OpenAI-compatible gateway consumed by Neoland
2. **gRPC Internal**: Local Qwen 1.8B (CPU fallback)
3. **SecureLLM providers / upstreams**: `ml-ops-api`, cloud providers, and local backends behind the gateway

**Configuration**: `--neoland-gateway-url` points to the primary OpenAI-compatible gateway endpoint.

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

### securellm-bridge

Primary LLM gateway consumed by Neoland:

- OpenAI-compatible API for the TUI/runtime
- Audit, rate limiting and provider routing
- Can proxy to `ml-ops-api` for local inference

**Usage**: `--neoland-gateway-url` should point here in the default topology.

### ml-ops-api

Inference bridge behind the gateway:

- Backend routing (`llama.cpp`, `vLLM`, other local engines)
- GPU/local acceleration
- OpenAI-compatible upstream for the gateway
**Typical role**: upstream internal service, not the primary Neoland client endpoint.

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

Current source-of-truth docs:

- [`docs/neoland-project-snapshot.md`](docs/neoland-project-snapshot.md) — codebase map and runtime topology
- [`docs/neoland-progress.md`](docs/neoland-progress.md) — current status, evidence, and active gaps
- [`docs/neoland-roadmap.md`](docs/neoland-roadmap.md) — delivery roadmap to responsible public release
- [`docs/neoland-dx-roadmap.md`](docs/neoland-dx-roadmap.md) — promise/evidence/status verification loop
- [`docs/roadmaps/neoland-llm-runtime-roadmap.md`](docs/roadmaps/neoland-llm-runtime-roadmap.md) — SecureLLM/ml-ops/llama runtime alignment

Next delivery focus:

1. harden session serialization and session DB mapping;
2. add direct DSPy health to `neoland doctor`;
3. guard frontend analytics against partial ADR/checkpoint payloads;
4. run the first complete `Neoland -> SecureLLM Bridge -> ml-ops-api -> llama.cpp` smoke;
5. turn that smoke into the release-candidate preflight.

---

## Performance Characteristics

| Metric               | Value            | Notes                                             |
| -------------------- | ---------------- | ------------------------------------------------- |
| TUI Startup          | <50ms            |                                                   |
| Memory (TUI)         | \~15MB           |                                                   |
| Memory (Server)      | \~200MB (idle)   |                                                   |
| Qwen 1.8B Inference  | 5-10 tok/s (CPU) | Candle backend; consider llama.cpp for production |
| Build Time (release) | \~10s            |                                                   |

**Note**: Local inference at 5-10 tok/s CPU is suitable for development/testing. For production workloads, use the SecureLLM Bridge gateway backed by `ml-ops-api` and `llama.cpp`/`vLLM`. SLO targets (500 RPS, p99 <200ms) have not been validated yet.

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

1. **Session API hardening**: session serialization and DB row mapping need explicit failure handling before release candidate.
2. **Doctor coverage**: direct DSPy `NEOLAND_DSPY_URL` probing still needs to be added.
3. **Runtime smoke**: the official SecureLLM Bridge -> ml-ops-api -> llama.cpp path is documented but still needs one recorded full-stack task run.
4. **Frontend analytics guards**: dashboard analytics need to tolerate partial ADR/checkpoint payloads.
5. **Path Dependencies**: Cargo uses pinned git dependencies and local patching conventions; see `Cargo.toml` and `.cargo/config.toml` if developing with sibling checkouts.
6. **Local inference performance**: CPU Candle inference is a dev fallback, not the production performance path.
7. **SLO targets unvalidated**: 500 RPS / p99 <200ms targets require load-test evidence before being advertised.
8. **Tracked backup files**: `.orig` files should be removed or archived in a dedicated cleanup pass.

---

## 📚 Documentation

- [`docs/neoland-project-snapshot.md`](docs/neoland-project-snapshot.md): current codebase map
- [`docs/neoland-architecture.md`](docs/neoland-architecture.md): architecture overview
- [`docs/neoland-quickstart.md`](docs/neoland-quickstart.md): setup and first run
- [`docs/neoland-roadmap.md`](docs/neoland-roadmap.md): delivery roadmap
- [`docs/neoland-progress.md`](docs/neoland-progress.md): status and gaps
- [`docs/ADR/`](docs/ADR/): architectural decisions
- [`docs/runbooks/`](docs/runbooks/): operations runbooks

---

## Contributing

This is part of a larger research project. External contributions are not currently accepted.

---

## License

Proprietary - Internal Research Project

---

**Maintained by**: VoidNxSEC Team
**Last Updated**: 2026-05-17
