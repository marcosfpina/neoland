# Project Snapshot: Neoland

**Last Meta-Sync**: 2026-07-16
**Primary Engine**: Rust + Python + Leptos WASM
**Ecosystem Role**: control plane, web console, TUI, and agent-orchestration component
**Current Delivery Readiness**: 78/100 (v0.2.0 Honest Preview complete)
**Rule**: code, routes, tests, and runtime wiring win over older roadmap copy.

---

## Executive Map

Neoland is a multi-surface AI control-plane component:

```text
Operator
  -> TUI (ratatui) / Web Console (Leptos WASM) / Desktop (Tauri, v0.0.1+)
    -> Rust control plane (:3001 REST, :50051 gRPC)
      -> Python DSPy agent pipeline (:8001)
      -> Session/checkpoint storage (PostgreSQL)
      -> SSE event bus
      -> MCP / NATS / Matrix optional integrations
      -> SecureLLM Bridge gateway (:8080)
        -> ml-ops-api (:8083)
          -> llama.cpp (:8081) / vLLM (optional)
```

---

## Runtime Surfaces

| Surface | Stack | Status |
|---------|-------|--------|
| **TUI** | Rust + ratatui · 4 themes · chat bubbles · pipeline tree | ✅ Stable |
| **Web Console** | Leptos WASM + CSS artesanal · 3-panel SPA · SSE streaming | ✅ Honest Preview (v0.2.0) |
| **Desktop** | Tauri + Leptos (reuses WASM bundle) | ✅ v0.0.1 |
| **CLI / API** | Rust + axum + tonic · 15 REST endpoints + gRPC | ✅ Stable |
| **Agent Pipeline** | Python · DSPy 3.x · FastAPI · 4-stage | ✅ Stable |

---

## Codebase Inventory

| Area | Files | Purpose |
|------|-------|---------|
| `src/` | 50+ Rust files | CLI, server, TUI, auth, audit, health, agents, SSE, MCP, NATS, storage, LLM |
| `agents/neoland_agents/` | 24 Python files | FastAPI DSPy pipeline, schemas, signatures, checkpoints, RAG, mmap flags |
| `web/` | 8 files | Leptos WASM SPA: 5 components, API client, model, CSS, 16 unit tests |
| `tests/`, `agents/tests` | Rust, Python, shell | unit, integration, contract, security, load tests |
| `modules/applications/` | 4 Nix modules | Neoland, SecureLLM Bridge API, ml-ops-api, LLM suite |
| `deploy/` | Docker, Helm, Prometheus, Vector | deployment scaffolding |
| `scripts/` | 8+ scripts | SOPS runner, hooks, validation, backup/restore |
| `docs/` | 71 docs | ADRs, roadmaps, runbooks, specs, architecture |
| `.github/` | 12 workflows | CI/CD: test, lint, build, release, nix, ADR, web |

---

## Endpoint Map

### Rust Control Plane

Public:
- `GET /health`, `GET /ready`, `GET /live`, `GET /metrics`
- `GET /openapi.json`, `GET /swagger-ui/`
- `GET /v1/agents/health`

Protected:
- `POST /v1/chat/completions`
- `POST /v1/agents/task`
- `GET /v1/agents/sessions`
- `GET /v1/agents/session/:id`
- `GET /v1/agents/session/:id/messages`
- `PATCH /v1/agents/session/:id/name`
- `POST /v1/agents/session/:id/steer`
- `POST /v1/agents/session/:id/breakpoint/resolve`
- `GET /v1/agents/tools`
- `POST /v1/agents/tools/call`
- `GET /v1/agents/events`
- `GET /v1/agents/events/:session`

### Python DSPy Pipeline

- `POST /v1/pipeline/run`
- `GET /v1/pipeline/session/{session_id}`
- `GET /health`

### Web Console (client-side)

- REST: `GET /v1/agents/sessions`, `GET /v1/agents/session/:id/messages`, `POST /v1/agents/task`, `PATCH /v1/agents/session/:id/name`, `POST /v1/agents/session/:id/steer`
- SSE: `GET /v1/agents/events/:session` (EventSource with structured SseEvent parsing)

---

## Delivery Evidence (v0.2.0)

| Gate | Result |
|------|--------|
| Rust unit tests | ✅ 275 passed, 18 ignored |
| Web Console tests | ✅ 16/16 passed |
| Clippy (all targets) | ✅ 0 warnings |
| Clippy (WASM) | ✅ 0 warnings |
| E2E REST tests | ✅ 22/22 passed |
| Python contracts | ✅ 26/26 passed |
| WASM compilation | ✅ Clean |
| Format check | ✅ |
| Doctor (`ok: true`) | ✅ |
| Full-stack smoke | ✅ 9/9 layers |

---

## Port And Env Map

| Component | Default | Primary Env |
|-----------|---------|-------------|
| Neoland REST | `127.0.0.1:3001` | `NEOLAND_CONTROL_PLANE_URL`, `NEOLAND_REST_PORT` |
| Neoland gRPC | `[::1]:50051` | `NEOLAND_GRPC_PORT` |
| Web Console dev | `127.0.0.1:8080` | Trunk proxy → REST :3001 |
| DSPy agents | `127.0.0.1:8001` | `NEOLAND_DSPY_URL` |
| SecureLLM Bridge | `127.0.0.1:8080` | `NEOLAND_GATEWAY_URL` |
| ml-ops-api | `127.0.0.1:8083` | `ML_OPS_API_URL` |
| llama.cpp | `127.0.0.1:8081` | `LLAMACPP_URL` |
| vLLM | optional | `VLLM_URL` |
| checkpoints | `~/.local/share/neoland/checkpoints/adr` | `NEOLAND_CHECKPOINT_DIR` |

---

## Main Hotspots

| Hotspot | Why It Matters | Next Action |
|---------|----------------|-------------|
| `src/server/mod.rs` session JSON | `unwrap_or_default()` hides serialization failures | return explicit 500 and log context |
| `src/agents/session.rs` row mapping | tuple-index mapping fragile under schema drift | move to named `FromRow` |
| `src/commands.rs` doctor | gateway/ml-ops/llama probed, DSPy :8001/health missing | add DSPy probe |
| `web/src/main.rs` SSE | stream generation guard works, but no explicit close on old EventSources | add `.close()` via stored handle (v0.3.0) |
| `web/src/api.rs` dead code | API surface includes unused endpoints (steer, get_session) | wire them as UI features ship |
| Deploy stack | Docker/Helm exist, no canonical smoke path | define one blessed boot/health/task path |
| gRPC-web | not yet bridged | tonic-web nativo (v0.3.0) |

---

## Source Of Truth

- **Roadmap**: [`ROADMAP.md`](../ROADMAP.md) (canonical)
- **Release Notes**: [`RELEASE-v0.2.0.md`](../RELEASE-v0.2.0.md)
- **Architecture**: [`docs/neoland-architecture.md`](neoland-architecture.md)
- **Quickstart**: [`docs/neoland-quickstart.md`](neoland-quickstart.md)
- **ADRs**: [`docs/ADR/`](ADR/)
- **Runbooks**: [`docs/runbooks/`](runbooks/)
