# Project Snapshot: Neoland

**Last Meta-Sync**: 2026-08-02
**Primary Engine**: Rust + Python + Leptos WASM
**Ecosystem Role**: control plane, web console, TUI, and agent-orchestration component
**Current Delivery Readiness**: v0.1.0 release candidate; public-release gates remain open
**Rule**: [`ROADMAP.md`](../ROADMAP.md), code, routes, tests, and runtime wiring win over older copy. Numeric readiness scores are not release evidence.

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
| **TUI** | Rust + ratatui · 4 themes · chat bubbles · pipeline tree | Implemented; authenticated terminal smoke pending |
| **Web Console** | Leptos WASM + CSS artesanal · 3-panel SPA · SSE streaming | CI-validated for unit, WASM, browser and bundle paths |
| **Desktop** | Tauri + Leptos (reuses WASM bundle) | Shell and derivation exist; signed installers pending |
| **CLI / API** | Rust + axum + tonic · REST + gRPC | Unit/TLS validated; external-service smoke pending |
| **Agent Pipeline** | Python · DSPy 3.x · FastAPI · 4-stage | Contract-tested; real-LLM full-stack smoke pending |

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

## Delivery Evidence (v0.1.0 release candidate)

Evidence below is from the PR #10 change set on 2026-08-02. The canonical
post-merge evidence is linked from [`ROADMAP.md`](../ROADMAP.md).

| Gate | Result |
|------|--------|
| Rust workspace library tests | ✅ 301 core passed, 18 ignored; 16 web passed |
| Web Console tests | ✅ 16/16 passed |
| Clippy (all targets) | ✅ 0 warnings |
| Clippy (WASM) | ✅ 0 warnings |
| Python contracts | ✅ 26/26 passed |
| Web workflow | ✅ unit, check, clippy, browser WASM tests and release bundle |
| Docker Compose + Helm | ✅ config, lint, render, probes and secret keys |
| Nix outputs | ✅ flake check, `neoland`, `neoland-web`, `neoland-full`, wrapper help |
| TLS/mTLS smoke | ✅ CI smoke passed |
| Format check | ✅ |
| Full-stack smoke with real LLM | ⏳ Required before v0.1.0 |

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
| Deploy stack | Render/build gates exist; live external stack is not exercised | define one blessed boot/health/task path with a real LLM |
| Release evidence | CI evidence exists for technical jobs | repeat independent quickstart and complete public gates |

---

## Source Of Truth

- **Roadmap**: [`ROADMAP.md`](../ROADMAP.md) (canonical)
- **Release status and gates**: [`ROADMAP.md`](../ROADMAP.md)
- **Architecture**: [`docs/neoland-architecture.md`](neoland-architecture.md)
- **Quickstart**: [`docs/neoland-quickstart.md`](neoland-quickstart.md)
- **ADRs**: [`docs/ADR/`](ADR/)
- **Runbooks**: [`docs/runbooks/`](runbooks/)
