# Neoland

**Autonomous AI Engineering Platform** — Multi-agent ADR pipeline with TUI, Web Console, and Desktop app

**Version**: 0.4.0-beta · **Status**: Enterprise Ready · **Score**: 91/100 · [ROADMAP](ROADMAP.md)

---

## What it is

Neoland is the orchestration layer that wires a local AI stack together.
It exposes a unified REST/gRPC API, runs a four-stage DSPy multi-agent pipeline
(Junior → Senior → Architect → TechLeader), and surfaces everything through three
interfaces: **TUI** (ratatui), **Web Console** (Leptos WASM), and **CLI**.

### Screenshots

**TUI** — 3-column layout (Sessions / Conversation / Reasoning), 4 themes,
chat bubbles, pipeline tree with confidence badges:

```
│  󰽥 Neoland  │ 󰤣  │ 󰍉 local  │ 󰔎 0ms 󰡱 817d82  󰭹 Conversation                     󰀄   󰢻  │
│──────────────────────────────────────────────────────────────────────────────────│
│╭ 󰙯 Sessions ──────────────╮╭ 󰭹 Conversation ──────────────────╮╭ 󰒝 Reasoning ──────╮│
││  ✚ New Chat  ^n          ││  󰚩 Neoland                       ││  Refatorar módulo   ││
││  󰅂 ⠋ Refatorar módulo…   ││  ╭──────────────────────────────╮││    ├─ 󰄬 Root [92%]    ││
││      agora               ││  │ Claro! Sugiro multiprocessing│││    ├─ 󰄬 Echo [88%]    ││
││    ● Revisar pipeline…   ││  │ from multiprocessing import │││    ├─ ⠋ Lyra 0.0s     ││
││      18m                 ││  │ Pool                        │││    ╰─ 󰡱 ADR accepted   ││
││    ● Otimizar query db   ││  ╰──────────────────────────────╯│╰─────────────────────╯│
│╰──────────────────────────╯╰────────────────────────────────╯                     │
│  ╭────────────────────────────────────────────────────────────────────────────╮   │
│  │ L local │  󰢱 balanced │ 󰑮 Aguardando intervenção (Steer)...                  │   │
│  ╰────────────────────────────────────────────────────────────────────────────╯   │
```

**Web Console** — Leptos WASM SPA, same 3-panel layout, CSS artesanal zero deps:

```
┌─ Topbar ──────────────────────────────────────────────────────────────────────────┐
│  [☰ Sessions]  NEOLAND://CORE  [⚙ Reasoning]                                      │
├─ Workspace ───────────────────────────────────────────────────────────────────────┤
│ ┌ Sessions ────────┐ ┌ Conversation ───────────────────────┐ ┌ Reasoning ────────┐ │
│ │ [+ New Chat]     │ │                                    │ │ Pipeline Tree     │ │
│ │                  │ │  󰚩 Neoland                         │ │  ├─ Junior [92%]   │ │
│ │ ● Session 1  5m  │ │  ╭────────────────────────────────╮│ │  ├─ Senior [88%]   │ │
│ │ ● Session 2  1h  │ │  │ Response streaming via SSE...   ││ │  ├─ Architect ⠋    │ │
│ │ ○ Session 3  1d  │ │  ╰────────────────────────────────╯│ │  ╰─ ADR accepted   │ │
│ └──────────────────┘ └────────────────────────────────────┘ └────────────────────┘ │
├─ Composer ────────────────────────────────────────────────────────────────────────┤
│  [Input box]                                                           [Submit ▶]  │
└────────────────────────────────────────────────────────────────────────────────────┘
```

> **Nota**: screenshots reais (PNG) podem ser capturadas com `just screenshot`
> quando o ambiente estiver rodando localmente.

```
┌───────────────┐  ┌──────────────┐  ┌──────────────────┐
│   Terminal    │  │   Browser    │  │   Desktop (Tauri) │
│   TUI (Rust)  │  │   Leptos SPA │  │   (v0.4.0+)      │
└───────┬───────┘  └──────┬───────┘  └────────┬─────────┘
        │                 │                    │
        └────────┬────────┴────────────────────┘
                 │ REST :3001 / gRPC :50051
         ┌───────▼───────────┐
         │   Neoland Control │
         │   Plane  (Rust)   │
         └───────┬───────────┘
                 │ HTTP
         ┌───────▼───────────┐
         │  SecureLLM Bridge │  :8080  (mTLS · PII redact)
         └───────┬───────────┘
                 │ HTTP
         ┌───────▼───────────┐
         │   ml-ops-api      │  :8083  (VRAM-aware routing)
         └───────┬───────────┘
                 │ HTTP
         ┌───────▼───────────┐
         │   Inference       │  llama.cpp :8081 / vLLM (opt)
         └───────────────────┘

         DSPy Pipeline      :8001  (Python · FastAPI)
         ADR Ledger         Merkle chain · secp256k1 · NATS JetStream
```

---

## Interfaces

| Surface | Stack | Status |
|---|---|---|
| **TUI** | Rust + ratatui · 4 themes · chat bubbles · pipeline tree | ✅ Stable |
| **Web Console** | Leptos WASM + CSS artesanal · 3-panel SPA · SSE streaming | ✅ Honest Preview |
| **Desktop** | Tauri + Leptos (reuses WASM bundle) | 🔮 v0.4.0 |
| **CLI / API** | Rust + axum + tonic · 15 REST endpoints + gRPC | ✅ Stable |

---

## Release status

| Gate | Result | Evidence |
|---|---|---|
| Rust unit tests | ✅ 242 passed, 17 ignored | `cargo test --workspace` |
| Clippy (all targets) | ✅ 0 warnings | `cargo clippy --all-targets -- -D warnings` |
| Clippy (WASM) | ✅ 0 warnings | `cargo clippy -p neoland-web --target wasm32-unknown-unknown` |
| Web Console tests | ✅ 16/16 passed | `cargo test -p neoland-web` |
| E2E REST tests | ✅ 22/22 passed | `cargo test --test rest_api_test` |
| Python contracts | ✅ 26/26 passed | `pytest -m contract` |
| WASM compilation | ✅ Clean | `cargo check -p neoland-web --target wasm32-unknown-unknown` |
| Doctor | ✅ `ok: true` | `just doctor` |
| Full-stack smoke | ✅ 9/9 layers | `just smoke` |

---

## Quick start

### Requirements

- Nix with flakes enabled (or NixOS)
- PostgreSQL running locally with a `neoland` database

```bash
# One-time: create the database
psql -c "CREATE DATABASE neoland;"
```

### Control plane + TUI

```bash
git clone <this-repo> && cd neoland
nix develop

# Start server + TUI together
just dev

# Or individual:
just server          # REST :3001 + gRPC :50051
just tui             # TUI client (in another terminal)
```

### Web Console (dev)

```bash
# Single command: server + Web Console on :8080 with Trunk proxy
just dev-web

# Open http://localhost:8080 in your browser
```

### Docker Compose (full stack)

```bash
cd deploy/docker/
cp .env.example .env && $EDITOR .env   # set LLM_API_KEY

eval $(ssh-agent) && ssh-add ~/.ssh/id_ed25519
DOCKER_BUILDKIT=1 docker compose build --ssh default
docker compose up -d

curl http://localhost:3001/health | jq .
```

---

## CLI reference

```
neoland server     # gRPC :50051 + REST :3001
neoland client     # TUI terminal client
neoland doctor     # environment diagnostics (--json for machine output)
neoland test       # health checks
neoland restart    # kill + restart server
```

Aliases set by `nix develop`: `nsrv` (server), `ncli` (client), `nd` (doctor), `nt` (test).

---

## Architecture

### Control plane (Rust)

```
src/
├── bin/neoland.rs          CLI entrypoint
├── server/mod.rs           REST + gRPC handlers · security middleware
├── agents/                 DSPy pipeline client, orchestrator, sessions, escalation
├── mcp/                    Native MCP server (breakpoints + tool routing)
├── tools/                  NativeTool trait + ShellTool
├── auth.rs                 API key auth · RBAC (Admin / User / ReadOnly)
├── secrets.rs              Vault → SOPS → env fallback
├── audit.rs                Structured JSON audit · brute-force detection
├── validation.rs           Input sanitization · path traversal prevention
├── tui/                    ratatui TUI · 4 themes · SSE subscriber
└── storage/                PostgreSQL + pgvector
```

### Agent pipeline (Python · DSPy 3.x)

```
agents/neoland_agents/
├── app.py             FastAPI · /health · /run · /sessions
├── signatures/        DSPy contracts (4 agents)
├── modules/           Junior · Senior · Architect · TechLeader
├── pipeline/          Orchestrator · checkpoint persistence
└── schemas/api.py     Pydantic ↔ Rust mirror types
```

### Web Console (Leptos WASM)

```
web/
├── src/
│   ├── main.rs        App component · signal-based state
│   ├── api.rs         REST + SSE client (gloo-net + EventSource)
│   ├── model.rs       Message data model
│   ├── lib.rs         Crate root + unit tests (16 tests)
│   └── components/    5 Leptos components
│       ├── composer.rs      Input box + submit
│       ├── conversation.rs  Chat messages + streaming
│       ├── reasoning.rs     Pipeline reasoning panel
│       ├── sessions.rs      Session list sidebar
│       └── topbar.rs        Header bar
├── public/neoland.css  ~600 lines · zero dependencies
├── Trunk.toml           Proxy config for dev
└── Cargo.toml           Workspace member
```

---

## Testing

```bash
# Rust — all crates
cargo test --workspace

# Web Console only
cargo test -p neoland-web

# Python — no LLM needed
cd agents && poetry run pytest tests/ -m contract -v

# Python — requires LLM_API_KEY
cd agents && poetry run pytest tests/ -m integration -v

# WASM compilation check
cargo check -p neoland-web --target wasm32-unknown-unknown

# SLO validation (requires running server)
just validate-slo
```

---

## Security

- **RBAC**: 3 roles (Admin / User / ReadOnly) via `X-API-Key` header
- **Secrets**: Three-tier retrieval — Cache → HashiCorp Vault → env fallback
- **Audit**: Structured JSON, 15 action types, brute-force detection (>5 failures/min)
- **Rate limiting**: 100 req/min per user/IP · max 100KB prompt
- **mTLS**: Planned (v0.4.0)
- See: [`docs/ADR/`](docs/ADR/) for full security decisions

---

## Performance

| Metric | Value |
|---|---|
| TUI startup | <50ms |
| Memory (TUI) | ~15MB |
| Memory (server, idle) | ~200MB |
| `/live` | 37,600 RPS · p99 12ms |
| `/health` | 258 RPS · p99 309ms (IO-bound) |
| Qwen 1.8B CPU | 5–10 tok/s (dev fallback) |
| Build time (release) | ~10s |

---

## NixOS Integration

```nix
imports = [ ./modules/applications/neoland.nix ];

services.neoland = {
  enable = true;
  openFirewall = true;
  environmentFile = "/run/secrets/neoland.env";
};
```

Nix flake provides:
- `nix build .#neoland` — control plane binary
- `nix build .#neoland-web` — Web Console WASM bundle (requires network for first build)
- `nix develop` — dev shell with all tooling (Rust, trunk, Python, SOPS, NATS, etc.)
- NixOS modules: `neoland` · `securellmBridgeApi` · `mlOpsApi` · `llmSuite` · `stack`

---

## Known limitations

- **pgvector** — vector store runs in-memory; `CREATE EXTENSION vector` for persistence
- **LLM API key** — full pipeline requires `OPENAI_API_KEY` (or equivalent litellm key)
- **SecureLLM Bridge Redis** — caching disabled without Redis (non-blocking)
- **CPU inference** — Candle/Qwen is dev-only (~5–10 tok/s); use llama.cpp/vLLM for production
- **Nix WASM build** — `nix build .#neoland-web` requires network on first run (cargo deps); cached thereafter
- **gRPC-web** — not yet bridged (v0.3.0); Web Console uses REST/SSE only
- **Non-Nix install** — Ubuntu bare metal path documented but not yet validated end-to-end

---

## Documentation

- [`ROADMAP.md`](ROADMAP.md) — release roadmap and delivery log
- [`docs/neoland-architecture.md`](docs/neoland-architecture.md) — architecture overview
- [`docs/neoland-quickstart.md`](docs/neoland-quickstart.md) — setup and first run
- [`docs/neoland-sops-setup.md`](docs/neoland-sops-setup.md) — secrets workflow
- [`docs/ADR/`](docs/ADR/) — architectural decisions (016+)
- [`docs/runbooks/`](docs/runbooks/) — operations runbooks

---

## License

Proprietary — Internal Research Project

**Maintained by**: VoidNxSEC Team  
**Last validated**: 2026-07-16 · build clean · REST/gRPC/LLM-gateway healthy · Web Console compiling for WASM
