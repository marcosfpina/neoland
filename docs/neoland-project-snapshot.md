# Project Snapshot: Neoland

**Last Meta-Sync**: 2026-05-17
**Primary Engine**: Rust + Python + TypeScript
**Ecosystem Role**: control plane, operator workbench, and agent-orchestration component
**Current Delivery Readiness**: 78/100 for responsible public pre-release
**Rule**: code, routes, tests, and runtime wiring win over older roadmap copy.

---

## Executive Map

Neoland is not a single narrow TUI anymore. The current codebase is a multi-surface
control-plane component:

```text
Operator
  -> TUI / Next.js workbench
    -> Rust control plane (:3001 REST, :50051 gRPC)
      -> Python DSPy agent pipeline (:8001)
      -> Session/checkpoint storage
      -> SSE event bus
      -> MCP / NATS / Matrix optional integrations
      -> SecureLLM Bridge gateway (:8080)
        -> ml-ops-api (:8083)
          -> llama.cpp (:8081) / vLLM (optional)
```

The project is strongest in Rust control-plane foundations, agent contracts, TUI
events, Nix-first DX, and operational documentation. The remaining delivery risk
is integration correctness: session serialization, full-stack smoke, frontend
analytics guards, direct DSPy diagnostics, and release-candidate discipline.

---

## Focused Codebase Inventory

The active implementation map, excluding generated caches and build output, is
approximately:

| Area | Files | Purpose |
|------|-------|---------|
| `src/` | 50 Rust files | CLI, server, TUI, auth, audit, health, agents, SSE, MCP, NATS, storage, LLM clients |
| `agents/neoland_agents/` | 24 Python package files | FastAPI DSPy pipeline, schemas, signatures, checkpoints, RAG, mmap flags |
| `matrix/frontend/app`, `components`, `lib` | 154 TypeScript/TSX files | Next.js operator workbench, pipeline runner, sessions, ADR, services, API relays |
| `matrix/backend/src` | 11 Python files | optional ranking/feedback/observability backend |
| `tests/`, `agents/tests`, `matrix/backend/tests` | Rust, Python, shell, expect | unit, integration, contract, security, load, and TUI automation coverage |
| `modules/applications/` | 4 Nix modules | Neoland, SecureLLM Bridge API, ml-ops-api, integrated LLM suite |
| `deploy/` | Docker, Helm, Prometheus, Vector | deployment scaffolding and observability |
| `scripts/` | 8 scripts | SOPS runner, hooks, validation, backup/restore |
| `docs/` | 71 docs total | ADRs, roadmaps, runbooks, specs, architecture, quickstart |
| `docs/ADR/` | 11 ADRs | active architectural decision vault |
| `docs/runbooks/` | 24 runbooks | incident and operations procedures |

Focused source/doc files counted for the active map: 322.

---

## Runtime Surfaces

| Surface | Entry Point | Current State |
|---------|-------------|---------------|
| CLI | `src/bin/neoland.rs`, `src/cli.rs`, `src/commands.rs` | `server`, `client`, `test`, `restart`, `doctor`; canonical gateway arg is `--neoland-gateway-url` |
| REST/gRPC server | `src/server/mod.rs` | chat, health, metrics, OpenAPI, agent task/session/tool/SSE endpoints |
| TUI | `src/tui/` | code-task workstation with SSE events, stage output, breakpoints, steering, history |
| Agent pipeline | `agents/neoland_agents/app.py` | `/v1/pipeline/run`, `/v1/pipeline/session/{session_id}`, `/health` |
| Frontend workbench | `matrix/frontend/app/*` | pipeline runner, session registry, ADR vault, services map, settings, dashboards |
| Frontend API relay | `matrix/frontend/app/api/neoland/*` | task run, sessions, session SSE relay, ADR-backed stats |
| Nix DX | `flake.nix`, `justfile`, `scripts/neoland-run.sh` | dev shell wrappers, SOPS-loaded just commands, frontend commands, agent helpers |
| NixOS modules | `modules/applications/*.nix` | control plane plus integrated SecureLLM Bridge + ml-ops suite |
| Deployment | `deploy/docker`, `deploy/helm`, `deploy/prometheus`, `deploy/vector` | present, but not yet the public-release gate |

---

## Endpoint Map

### Rust Control Plane

Public:

- `GET /health`
- `GET /ready`
- `GET /live`
- `GET /metrics`
- `GET /v1/agents/health`
- `GET /openapi.json`
- `GET /swagger-ui/`

Protected:

- `POST /v1/chat/completions`
- `POST /v1/agents/task`
- `GET /v1/agents/sessions`
- `GET /v1/agents/session/:id`
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

### Frontend API Relays

- `POST /api/neoland/pipeline/run`
- `GET /api/neoland/events/[sessionId]`
- `GET /api/neoland/sessions`
- `GET /api/neoland/sessions/[sessionId]`
- `GET /api/neoland/stats`
- ADR, service, health, and system relay routes under `matrix/frontend/app/api/`

---

## Port And Env Map

| Component | Default | Primary Env |
|-----------|---------|-------------|
| Neoland REST | `127.0.0.1:3001` | `NEOLAND_CONTROL_PLANE_URL`, `NEOLAND_REST_PORT` |
| Neoland gRPC | `[::1]:50051` | `NEOLAND_GRPC_PORT` |
| Frontend dev | `127.0.0.1:3006` | `NEOLAND_FRONTEND_HOST`, `NEOLAND_FRONTEND_PORT` |
| DSPy agents | `127.0.0.1:8001` | `NEOLAND_DSPY_URL` |
| SecureLLM Bridge gateway | `127.0.0.1:8080` | `NEOLAND_GATEWAY_URL` |
| Legacy gateway alias | `127.0.0.1:8080` | `NEOLAND_ML_API_URL` |
| ml-ops-api | `127.0.0.1:8083` | `ML_OPS_API_URL` |
| llama.cpp | `127.0.0.1:8081` | `LLAMACPP_URL` |
| vLLM | optional | `VLLM_URL` |
| checkpoints | `/var/lib/neoland/checkpoints/adr` | `NEOLAND_CHECKPOINT_DIR` |
| mmap IPC | `/run/neoland/agent-flags.shm` | `NEOLAND_SHM_PATH` |

`NEOLAND_GATEWAY_URL` is the canonical gateway variable. `NEOLAND_ML_API_URL`
remains accepted for compatibility.

---

## Delivery Evidence

Verified during this sync:

- Git state: clean `main...github/main` before edits.
- Command surface: `just --list` exposes server/client/doctor/test/frontend/agents/secrets flows.
- Rust unit baseline: `cargo test --lib --quiet` passed with `226 passed; 17 ignored`.
- Frontend session and pipeline surfaces consume real control-plane routes and SSE relay code.
- Python agent API now exposes session checkpoint listing; it is no longer a pure stub.
- Nix suite and dev shell now export the canonical `NEOLAND_GATEWAY_URL` alongside the legacy alias.

Not verified in this sync:

- Full frontend production build.
- Python contract suite.
- End-to-end stack with SecureLLM Bridge + ml-ops-api + llama.cpp.
- Load test/SLO targets.

---

## Main Hotspots

| Hotspot | Why It Matters | Next Action |
|---------|----------------|-------------|
| `src/server/mod.rs` session JSON conversion | `unwrap_or_default()` can hide serialization failures as `{}` | return explicit 500 and log context |
| `src/agents/session.rs` row mapping | tuple-index mapping is fragile under schema drift | move to named `FromRow` mapping |
| `src/commands.rs` doctor | gateway/ml-ops/llama are probed, but direct DSPy `:8001/health` is still missing | add DSPy probe and output field |
| `matrix/frontend/lib/neoland/server.ts` analytics | ADR documents can have partial `full_pipeline` payloads | add optional guards and neutral defaults |
| `src/tui/mod.rs.orig`, `agents/neoland_agents/tools.py.orig` | tracked backup files blur the active implementation map | archive or remove in a dedicated cleanup commit |
| deploy stack | Docker/Helm exist, but the release smoke is not yet canonical | define one blessed boot/health/task/restore path |

---

## Source Of Truth

Use these files for planning:

- Progress: [`docs/neoland-progress.md`](neoland-progress.md)
- Roadmap: [`docs/neoland-roadmap.md`](neoland-roadmap.md)
- DX verification: [`docs/neoland-dx-roadmap.md`](neoland-dx-roadmap.md)
- Runtime topology: [`docs/roadmaps/neoland-llm-runtime-roadmap.md`](roadmaps/neoland-llm-runtime-roadmap.md)
- Improvement backlog: [`docs/neoland-improvements-todo.md`](neoland-improvements-todo.md)

Older checkpoint files under `docs/archive/` and older runbooks are historical
context, not current readiness truth.
