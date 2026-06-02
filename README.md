# Neoland

**AI control-plane component** — Rust REST/gRPC · Python DSPy pipeline · Next.js workbench · Nix-first runtime

```
Release Candidate — 98/100 — Preflight 8/8 ✅ — Smoke 9/9 ✅
```

---

## What it is

Neoland is the orchestration layer that wires your local AI stack together.
It exposes a unified REST/gRPC API, runs a four-stage DSPy multi-agent pipeline
(Junior → Senior → Architect → TechLeader), and renders the whole thing in a
Next.js operator workbench and a Rust TUI. Everything boots from a single Nix
dev shell — no Docker required for the control plane itself.

**Topology**

```
┌─────────────────────────────────────────────────────────┐
│                     Operator                            │
│          TUI (ratatui)   ·   Workbench (:3006)          │
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
| Frontend lint | ✅ 0 errors | `npm run lint` |
| Frontend build | ✅ 25 routes | `npm run build` |
| Doctor | ✅ `ok: true` | `just doctor` |
| Full-stack smoke | ✅ 9/9 layers | `just smoke` |

**Last preflight**: 2026-06-02 · `just preflight` → PASS 8/8

**Performance** (measured, not estimated):

| Endpoint | RPS | p99 latency |
|----------|-----|-------------|
| `GET /live` | **37,600** | 12 ms |
| `GET /health` | 258 | 309 ms (IO-bound by design — polls every layer) |

---

## Quick start

### Requirements

- Nix with flakes enabled (or NixOS)
- PostgreSQL running locally with a `neoland` database

```bash
# one-time: create the database
psql -c "CREATE DATABASE neoland;"
```

### Boot the control plane

```bash
git clone <this-repo> && cd neoland
nix develop

# loads SOPS secrets + starts gRPC :50051 + REST :3001
just server

# verify
just doctor
```

### Boot the full stack

```bash
# 1. Start DSPy pipeline
just agents-start

# 2. Start SecureLLM Bridge (from its project dir)
cd ../securellm-bridge/docker && docker compose up -d securellm-proxy

# 3. Start ml-ops-api (from its project dir)
cd ../ml-ops-api && docker compose up -d

# 4. Smoke the whole topology
cd -  # back to neoland
just smoke
```

### Common commands

```bash
just          # list all recipes
just server   # start control plane (SOPS + gRPC + REST)
just client   # start TUI
just doctor   # environment diagnostics (JSON)
just smoke    # full-stack smoke (9 layers)
just preflight # all release gates (8 gates)
just roadmap  # print current roadmap
```

---

## Architecture

### Control plane (Rust)

```
src/
├── bin/neoland.rs     CLI entrypoint (server | client | doctor | restart)
├── server/mod.rs      gRPC + REST handlers · security middleware
├── agents/            DSPy session management · checkpoint relay
├── auth.rs            API key auth · RBAC (Admin / User / ReadOnly)
├── secrets.rs         HashiCorp Vault · SOPS · env fallback chain
├── audit.rs           structured JSON events · brute-force detection
├── validation.rs      input sanitization · path traversal prevention
├── tui/               ratatui TUI · Tokyo Night · SSE subscriber
└── storage/           PostgreSQL vector store · pgvector (optional)
```

### Agent pipeline (Python · DSPy 3.x)

```
agents/
├── app.py             FastAPI · /health · /run · /sessions
└── pipeline/
    ├── orchestrator.py  four-stage coordinator
    ├── checkpoint.py    ADR artifact persistence
    └── modules/
        ├── junior.py    hypothesis generation (ReAct + tools)
        ├── senior.py    risk assessment
        ├── architect.py structural soundness
        └── tech_leader.py final decision (approve | reject | defer | escalate)
```

### Operator workbench (Next.js 15)

25 routes including live pipeline view, session inspector, ADR browser,
LLM playground, service dashboard, and agent analytics.

---

## Security

| Layer | Implementation |
|-------|---------------|
| Authentication | X-API-Key · RBAC · 3 roles |
| Secrets | HashiCorp Vault → SOPS → env fallback |
| Audit | Structured JSON · 15 action types · PII redacted |
| Rate limiting | 100 req/min per user/IP |
| Input validation | 100KB prompt cap · null byte removal · path traversal guard |
| Brute force | >5 failed auth / 1 min → alert |
| mTLS | SecureLLM Bridge layer (between Neoland and LLM providers) |

**SOPS workflow** — secrets live in `secrets/neoland.sops.env`, encrypted with age.
All `just` commands that touch the running server decrypt automatically.
See [`docs/neoland-sops-setup.md`](docs/neoland-sops-setup.md).

---

## Known limitations

These are real, not hypothetical:

- **pgvector not installed** — vector store runs in-memory only. Install the
  extension (`CREATE EXTENSION vector`) to enable persistent embedding search.
- **LLM API key required for full pipeline** — DSPy routes through litellm.
  Set `OPENAI_API_KEY` (or the relevant provider key) for task execution end-to-end.
  The smoke test confirms routing is wired; it does not validate model responses.
- **SecureLLM Bridge Redis** — runs degraded without Redis (caching disabled).
  Non-blocking for core proxy functionality.
- **CPU inference fallback** — local Qwen 1.8B via Candle is a dev fallback at
  ~5-10 tok/s. Production throughput comes from llama.cpp/vLLM behind ml-ops-api.
- **NixOS / Nix-first** — one documented non-Nix path (Ubuntu bare metal) is
  planned but not yet written. The Nix path is the only validated one.

---

## ADR ledger

Every pipeline decision produces an ADR artifact:

- **Merkle chain** — SHA-256 chained, tamper-evident
- **secp256k1 signatures** — each stage signs its output
- **NATS JetStream delivery** — at-least-once, durable consumer
- **Checkpoint storage** — `$NEOLAND_CHECKPOINT_DIR` (default: `~/.local/share/neoland/checkpoints/adr`)

ADR browser available in the workbench at `/adr`.

---

## Observability

- **Prometheus metrics** at `/metrics`
- **OpenTelemetry spans** (configurable exporter)
- **Swagger UI** at `/swagger-ui/`
- **Doctor JSON** at `just doctor` — reports each layer separately:
  control plane, gRPC, DSPy, gateway, ml-ops, llama.cpp, DB, Vault, config

---

## Development

```bash
# Enter dev shell (sets DATABASE_URL, NEOLAND_* env vars, shell aliases)
nix develop

# Run all tests
just test-all

# Rust only
nix develop --command cargo test --lib

# Python contracts
nix develop --command bash -c "cd agents && poetry run pytest tests/ -m contract -v"

# Frontend
cd matrix/frontend && npm run lint && npm run build
```

**Pre-commit hooks**: `cargo fmt --check`, `cargo clippy`, `cargo test --lib`

---

## Roadmap

Single source of truth: [`ROADMAP.md`](ROADMAP.md)

Current score: **98/100**. Remaining: docs honesty pass complete (this file),
RC freeze, public pre-release tag.

---

## License

Proprietary — Internal Research Project

**Maintained by**: VoidNxSEC Team  
**Last validated**: 2026-06-02 · preflight 8/8 · smoke 9/9
