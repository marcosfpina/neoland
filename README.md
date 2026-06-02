# Neoland

**AI control-plane component** — Rust REST/gRPC · Python DSPy pipeline · Next.js workbench · Nix-first runtime

**Version**: 0.1.0-rc.1 · **Status**: Release Candidate · **Score**: 99/100

---

## What it is

Neoland is the orchestration layer that wires a local AI stack together.
It exposes a unified REST/gRPC API, runs a four-stage DSPy multi-agent pipeline
(Junior → Senior → Architect → TechLeader), and surfaces everything through a
Rust TUI. Everything boots from a single Nix dev shell — no Docker required for
the control plane itself.

**Topology**

```
┌─────────────────────────────────────────────────────────┐
│                     Operator                            │
│                     TUI (ratatui)                        │
└────────────────────┬────────────────────────────────────┘
                     │ REST :3001 / gRPC :50051
         ┌───────────▼───────────┐
         │   Neoland Control     │
         │   Plane  (Rust)       │
         └───────────┬───────────┘
                     │ HTTP
         ┌───────────▼───────────┐
         │  SecureLLM Bridge     │  :8080
         │  (mTLS · PII redact)  │
         └───────────┬───────────┘
                     │ HTTP
         ┌───────────▼───────────┐
         │   ml-ops-api          │  :8083  (VRAM-aware routing)
         └───────────┬───────────┘
                     │ HTTP
         ┌───────────▼───────────┐
         │   llama.cpp server    │  :8081  (GPU inference)
         └───────────────────────┘

         DSPy Pipeline      :8001  (Python · FastAPI)
         ADR Ledger         Merkle chain · secp256k1 · NATS JetStream
```

---

## Release status

| Gate | Result | Evidence |
|------|--------|----------|
| Rust unit tests | ✅ 226 passed, 17 ignored | `cargo test --lib` |
| Clippy | ✅ 0 warnings/errors | `cargo clippy --all-targets -- -D warnings` |
| E2E REST tests | ✅ 22/22 passed | `cargo test --test rest_api_test` |
| Python contracts | ✅ 26/26 passed | `pytest -m contract` |
| Doctor | ✅ `ok: true` | `just doctor` |
| Full-stack smoke | ✅ 9/9 layers | `just smoke` |

**Last preflight**: 2026-06-02 · `just preflight` → PASS 6/6

---

## Quick start

### Requirements

- Nix with flakes enabled (or NixOS)
- PostgreSQL running locally with a `neoland` database

```bash
# one-time: create the database
psql -c "CREATE DATABASE neoland;"
```

### Installation

```bash
git clone <this-repo> && cd neoland
nix develop

# loads SOPS secrets + starts gRPC :50051 + REST :3001
just server

# or via shell aliases set by the dev shell:
neoland-server
neoland-client --neoland-gateway-url http://localhost:8080
neoland-doctor --json
```

### NixOS Integration

```nix
# configuration.nix
imports = [ ./modules/applications/neoland.nix ];

services.neoland = {
  enable = true;
  openFirewall = true;
  environmentFile = "/run/secrets/neoland.env";
};
```

The module configures `systemd.services.neoland` with state/cache/runtime/log directories,
`AUDIT_LOG_PATH`, DSPy URL, and server port wiring. Use `environmentFile` for
`DATABASE_URL` and API keys in production.

### Boot the full stack

```bash
# 1. Start DSPy pipeline
just agents-start

# 2. Start SecureLLM Bridge
cd ../securellm-bridge/docker && docker compose up -d securellm-proxy

# 3. Start ml-ops-api
cd ../ml-ops-api && docker compose up -d

# 4. Smoke all 9 layers
cd -
just smoke
```

---

## CLI reference

### Server mode

```bash
neoland server                        # gRPC :50051 + REST :3001
neoland server --grpc-port 50052 --rest-port 3002
nix develop --command neoland-server  # via dev shell alias
```

### Client mode (TUI)

```bash
neoland client
neoland client --neoland-gateway-url http://localhost:8080
neoland client --server-url http://[::1]:50051
```

TUI features: braille spinner · Tokyo Night · readline cursor (Ctrl+←/→, Ctrl+W/U/K) ·
auto-scroll · 5 preset profiles (Ctrl+1-5)

### Diagnostics

```bash
neoland doctor --json          # full layer-by-layer check
neoland test --json            # alias for health check
just smoke                     # full-stack 9-layer smoke
just preflight                 # all 6 release gates
just validate-slo              # load test against SLO targets (requires hey)
```

### Bootstrap order

```
1.  just server          # control plane  :3001 / :50051
2.  just agents-start    # DSPy pipeline  :8001
3.  (docker) securellm-bridge  :8080
    cd ../securellm-bridge/docker && docker compose up -d securellm-proxy
4.  (docker) ml-ops-api        :8083
    cd ../ml-ops-api && docker compose up -d
5.  just doctor          # confirm all layers
6.  just smoke           # full 9-layer verification
```

### SOPS secrets

```bash
# Dev workflow: secrets live in secrets/neoland.sops.env (age-encrypted)
# All 'just' commands that touch the running server decrypt automatically.
# Manual:
sops secrets/neoland.sops.env

# Without SOPS: set env vars manually, then use -raw variants
export NEOLAND_ADMIN_API_KEY="..."
just server-raw
```

See [`docs/neoland-sops-setup.md`](docs/neoland-sops-setup.md) for key setup.

---

## Architecture

### Control plane (Rust)

```
src/
├── bin/neoland.rs         CLI entrypoint (server | client | doctor | restart)
├── server/mod.rs          gRPC + REST handlers · security middleware
├── agents/
│   ├── client.rs          HTTP client — mirrors Python pipeline schemas
│   ├── orchestrator.rs    Orchestration core + live steering + breakpoints
│   ├── session.rs         Session state via PostgreSQL
│   ├── escalation.rs      Inter-agent escalation policy
│   └── checkpoint_store.rs
├── mcp/                   Native MCP server (breakpoints + tool routing)
├── tools/                 NativeTool trait + ShellTool
├── matrix/client.rs       Pipeline metrics reporting
├── ml_offload/client.rs   Inference bridge client (OpenAI-compatible)
├── auth.rs                API key auth · RBAC (Admin / User / ReadOnly)
├── secrets.rs             Vault → SOPS → env fallback chain
├── audit.rs               Structured JSON events · brute-force detection
├── validation.rs          Input sanitization · path traversal prevention
├── storage/vector_store.rs PostgreSQL + pgvector (optional)
├── tui/                   ratatui TUI · Tokyo Night · SSE subscriber
└── hyprland_ops.rs        Hyprland window manager IPC
```

Contracts between Rust and Python: types in `src/agents/client.rs` mirror
`agents/neoland_agents/schemas/api.py`. Any change on one side requires the other.

### Agent pipeline (Python · DSPy 3.x)

```
agents/neoland_agents/
├── app.py             FastAPI · /health · /run · /sessions
├── signatures/        DSPy contracts (4 agents)
├── modules/
│   ├── junior.py      Hypothesis generation (ReAct + tools)
│   ├── senior.py      Risk assessment and refinement
│   ├── architect.py   Structural soundness (conditional — escalate_to_architect)
│   └── tech_leader.py Final decision: approve | reject | defer | escalate
├── pipeline/
│   ├── orchestrator.py  Four-stage coordinator
│   └── checkpoint.py    ADR artifact persistence
├── schemas/api.py     Pydantic ↔ Rust mirror types
└── ipc/flags.py       SharedFlags mmap reader/writer
```

---

## Security

### Authentication & Authorization (ADR-011)

- REST API Key via `X-API-Key` header
- RBAC: 3 roles — Admin > User > ReadOnly
- Development keys pre-configured; production keys via SOPS or Vault
- gRPC mTLS: planned

### Secrets Management (ADR-012)

- Three-tier retrieval: Cache (30s TTL) → HashiCorp Vault → environment variables
- Graceful degradation when Vault is unavailable
- Secret types: LLM API keys, NEOLAND API keys, DB credentials, TLS certs
- Cache hit: <1ms · Vault read: 50–100ms

### Audit Logging (ADR-013)

- Structured JSON, immutable append-only
- 15 action types: auth, secrets, API, config, admin operations
- PII/credentials auto-redacted
- Brute-force detection: >5 failed auth in 1 min → alert

### Rate Limiting & Validation (ADR-014)

- 100 req/min per user/IP
- Max 100KB prompt · 100 messages · 1MB request
- Null byte removal · control character filtering · path traversal prevention

See: `docs/neoland-authentication.md` · `docs/neoland-vault-setup.md` · `docs/ADR/`

---

## Testing

| Suite | Count | Notes |
|-------|-------|-------|
| Rust lib | 226 passing, 17 ignored | TUI, auth, agents, metrics, OpenAPI |
| E2E REST | 22/22 | Real server on :3004, no mocks |
| Python contracts | 26/26 | Pydantic schemas + IPC — no LLM required |
| adr-ledger | 15 | Merkle chain, JetStream, signers |

```bash
# Rust
nix develop --command cargo test --lib
nix develop --command cargo test --test rest_api_test

# Python — no LLM needed
cd agents && poetry run pytest tests/ -m contract -v

# Python — requires LLM_API_KEY
cd agents && poetry run pytest tests/ -m integration -v

# SLO validation (requires hey + running server)
nix develop --command just validate-slo
```

---

## Architectural decisions

Full ADR log: [`docs/ADR/`](docs/ADR/) · summary: [`docs/neoland-adr.md`](docs/neoland-adr.md)

**Key decisions**:
- **ADR-001**: 3-layer architecture (Infra / Security / Compliance)
- **ADR-002**: LocalFirst LLM routing strategy
- **ADR-003**: SecureLLM Proxy with Factory Pattern
- **ADR-004**: Connection pooling for low latency
- **ADR-011–014**: Security hardening (auth, secrets, audit, validation)

**TUI over GTK4**: 50ms startup vs 2–3s · 15MB vs 300MB · native tmux/zellij integration.
Replaced legacy GTK client with `ratatui` + `crossterm`.

**LLM fallback chain**:
1. SecureLLM Bridge (:8080) — primary OpenAI-compatible gateway
2. gRPC internal — local Qwen 1.8B CPU fallback
3. SecureLLM providers/upstreams — ml-ops-api, cloud, local backends behind gateway

---

## Integrations

**securellm-bridge** — primary LLM gateway. OpenAI-compatible · audit · rate limiting ·
provider routing · can proxy to ml-ops-api for local inference.
`--neoland-gateway-url` should point here.

**ml-ops-api** — inference bridge behind the gateway. VRAM-aware routing to
llama.cpp / vLLM / other local engines. OpenAI-compatible upstream.
Typical role: internal upstream, not direct client endpoint.

**intelagent-core (Phantom)** — task orchestration framework.
Multi-step reasoning · tool execution · memory management.
Path dep: `../phantom/intelagent/crates/core`

**Hyprland IPC** — window management via `hyprland-ipc` crate.
Scratchpad toggle · floating window rules · opacity control.
Path dep: `../ai-agent-os/crates/hyprland-ipc`

---

## ADR ledger

Every pipeline decision produces an ADR artifact:

- **Merkle chain** — SHA-256 chained, tamper-evident
- **secp256k1 signatures** — each stage signs its output
- **NATS JetStream** — at-least-once delivery, durable consumer
- **Checkpoint storage** — `$NEOLAND_CHECKPOINT_DIR` (default: `~/.local/share/neoland/checkpoints/adr`)

ADR browser in the workbench at `/adr`.

---

## Performance

Characteristics are measured, not estimated. Run `just validate-slo` to verify
against defined targets on your hardware.

| Metric | Value | Notes |
|--------|-------|-------|
| TUI startup | <50ms | |
| Memory (TUI) | ~15MB | |
| Memory (server, idle) | ~200MB | |
| `/live` — measured | 37,600 RPS · p99 12ms | SLO target: ≥10K RPS · p99 ≤50ms |
| `/health` — measured | 258 RPS · p99 309ms | IO-bound by design (polls all layers) · SLO target: p99 ≤1000ms |
| Qwen 1.8B inference | 5–10 tok/s (CPU) | Dev fallback; use llama.cpp/vLLM for production |
| Build time (release) | ~10s | |

Numbers above were recorded on this machine. Re-run `just validate-slo` after any
infrastructure change to confirm they still hold.

---

## Observability

- **Prometheus metrics** at `/metrics`
- **OpenTelemetry spans** (configurable exporter)
- **Swagger UI** at `/swagger-ui/`
- **Doctor JSON** at `just doctor` — reports control plane, gRPC, DSPy, gateway,
  ml-ops, llama.cpp, DB, Vault, config as separate fields

---

## Known limitations

- **pgvector not installed** — vector store runs in-memory only.
  `CREATE EXTENSION vector` in the `neoland` database to enable persistent search.
- **LLM API key required for full pipeline** — DSPy routes through litellm.
  Set `OPENAI_API_KEY` (or the relevant provider key). Smoke confirms routing;
  it does not validate model responses.
- **SecureLLM Bridge Redis** — caching disabled without Redis. Non-blocking.
- **CPU inference** — Candle/Qwen is a dev fallback (~5–10 tok/s).
  Production throughput comes from llama.cpp/vLLM behind ml-ops-api.
- **Nix-first** — one documented non-Nix path (Ubuntu bare metal) is planned
  but not yet validated.
- **Path dependencies** — Cargo uses pinned git deps and local patches.
  See `Cargo.toml` and `.cargo/config.toml` when developing with sibling checkouts.
- **gRPC mTLS** — planned, not yet implemented.

---

## Documentation

- [`ROADMAP.md`](ROADMAP.md) — current planning document and delivery log
- [`docs/neoland-project-snapshot.md`](docs/neoland-project-snapshot.md) — codebase map
- [`docs/neoland-architecture.md`](docs/neoland-architecture.md) — architecture overview
- [`docs/neoland-quickstart.md`](docs/neoland-quickstart.md) — setup and first run
- [`docs/neoland-sops-setup.md`](docs/neoland-sops-setup.md) — secrets workflow
- [`docs/neoland-vault-setup.md`](docs/neoland-vault-setup.md) — Vault integration
- [`docs/ADR/`](docs/ADR/) — architectural decisions
- [`docs/runbooks/`](docs/runbooks/) — operations runbooks

---

## Contributing

Part of a larger research project. External contributions not currently accepted.

---

## License

Proprietary — Internal Research Project

**Maintained by**: VoidNxSEC Team  
**Last validated**: 2026-06-02 · preflight 8/8 · smoke 9/9
