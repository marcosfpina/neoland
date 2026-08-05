> **Source-of-truth movida para `ROADMAP.md` na raiz do projeto (2026-05-31). Este arquivo é histórico.**
>
> Todo o conteúdo abaixo é um snapshot de 2026-05-17 e não descreve o estado
> atual. Para o release candidate e suas evidências, consulte:
> - [`ROADMAP.md`](../ROADMAP.md)
> - [`docs/neoland-project-snapshot.md`](neoland-project-snapshot.md)

# Neoland Delivery Progress (HISTÓRICO)

**Last Updated**: 2026-05-17
**Historical Status At Snapshot Date**: integrated pre-release beta
**Historical Readiness Estimate**: **78/100** (not a current release claim)
**Rust baseline**: `cargo test --lib --quiet` -> **226 passed, 17 ignored**
**Source-of-truth rule**: code, routes, tests, and runtime wiring win over older status docs.

---

## Executive Status

Neoland has crossed the line from "collection of promising parts" into a real
control-plane component. The core stack now includes:

- Rust control plane with REST/gRPC, auth, health, metrics, OpenAPI, SSE, sessions, MCP, NATS, and matrix hooks.
- Python DSPy pipeline with typed contracts, checkpoints, session listing, RAG hooks, and mmap IPC flags.
- Next.js workbench with real pipeline execution, session registry, ADR vault, services view, and SSE relay.
- TUI workstation with live stage output, breakpoints, steering, history, and code-task orientation.
- Nix-first DX with `just`, SOPS-loaded wrappers, frontend commands, and NixOS modules for Neoland, SecureLLM Bridge, and ml-ops-api.

The remaining work is mostly integration hardening and release discipline:

- fix fragile session serialization and DB row mapping;
- add direct DSPy health to `doctor`;
- guard frontend analytics against partial ADRs;
- validate one full runtime path: Neoland -> SecureLLM Bridge -> ml-ops-api -> llama.cpp;
- turn deploy/backup/restore into one repeatable public-release ritual.

---

## Current Codebase Map

| Stream | Status | Evidence | Current Risk |
|--------|--------|----------|--------------|
| Rust control plane | 86% | `/v1/agents/*`, `/health`, `/ready`, `/live`, `/metrics`, OpenAPI, SSE, auth, sessions | fragile session serialization, tuple row mapping |
| Agent pipeline | 84% | FastAPI `/v1/pipeline/run`, `/v1/pipeline/session/{session_id}`, checkpoint store, contract tests | full E2E needs live LLM/DB smoke |
| TUI workstation | 82% | stage output SSE, breakpoints, steering, history, code-task flow | SSE timeout/polish and recurrent UX bugs still need closure |
| Frontend workbench | 72% | pipeline runner, SSE relay, session registry, ADR source-of-truth stats | analytics guards and full build/lint still unverified |
| LLM runtime topology | 76% | `neoland_gateway_url`, gateway health candidates, ml-ops/llama optional probes, Nix suite | stack boot with SecureLLM + ml-ops + llama not proven in one run |
| Ops and deployment | 67% | NixOS modules, Docker/Helm, Prometheus, Vector, backup scripts, runbooks | no single blessed release smoke/rollback gate |
| Documentation truth | 70% | this sync updates snapshot, progress, roadmap, DX/runtime roadmaps | older archived/runbook claims still exist as history |

---

## Completed Delivery History

### Cycle 0 - Core Pipeline

Done:

- Rust control plane connected to Python DSPy pipeline.
- Agent task/session contracts landed.
- Python 3.13 dev shell and contract tests established.
- ADR-019 documented the multi-agent pipeline.

### Cycle 1 - IPC, Events, And Ledger Edges

Done:

- mmap IPC flags and Rust/Python layout.
- NATS publisher and event subjects.
- ADR-ledger integration path and JetStream semantics.
- Phantom scan event path documented as cross-stack integration.

### Cycle 2 - Operational Stack

Done:

- Prometheus metrics.
- cargo-audit in dev shell.
- NKey/SOPS direction for NATS ACL.
- EDR rule set and cross-stack test references.

### Cycle 3 - Quality And Observability

Done:

- TUI async rewrite.
- TUI cursor/history/render tests.
- OpenTelemetry instrumentation on agent flow.
- Python contract suite for schemas and IPC flags.

### Cycle 4 - Agent Workstation

Done:

- TUI task queue, live pipeline panel, keybindings, and transparent stage output.
- MCP stdio registry under `src/mcp/`.
- Matrix metrics integration as optional telemetry.
- Pipeline history and workbench surfaces.

### Cycle 5 - Communication Gaps

Done:

- `BreakpointResolved` now reaches the TUI and moves tasks out of `WaitingForBreakpoint`.
- `pipeline_started` and trailing SSE buffer are handled instead of silently lost.
- Steering/breakpoint failures route to visible pipeline error output.
- LLM gateway health check probes real HTTP status.
- Frontend `/pipeline` uses `PipelineRunner`, and `/sessions` uses `GET /v1/agents/sessions`.
- Stats/pipeline routes moved to ADR/control-plane truth instead of Matrix as canonical source.
- llama.cpp port alignment moved to `8081` in Nix modules.

### Cycle 6 - DX And Topology Alignment

Done or newly reconciled in this sync:

- `justfile` command map confirms SOPS-loaded `server`, `client`, `doctor`, and raw variants.
- `src/config.rs` and CLI use `neoland_gateway_url` / `--neoland-gateway-url`.
- `flake.nix` and `modules/applications/neoland-llm-suite.nix` now export canonical `NEOLAND_GATEWAY_URL` while preserving `NEOLAND_ML_API_URL`.
- Project snapshot, progress, roadmap, improvement backlog, DX roadmap, and runtime roadmap are realigned to the current tree.

---

## Active Gap Register

| ID | Severity | Area | Status | Action |
|----|----------|------|--------|--------|
| GAP-001 | Critical | `src/commands.rs` | Open | Add direct `NEOLAND_DSPY_URL/health` probe to `neoland doctor` and render it separately from control-plane health. |
| GAP-002 | Critical | `src/server/mod.rs` | Open | Replace `serde_json::to_value(...).unwrap_or_default()` in session handlers with explicit error handling. |
| GAP-003 | Critical | `src/agents/session.rs` | Open | Replace tuple-index `sqlx::query_as` mapping with named `FromRow` mapping or equivalent. |
| GAP-004 | High | `matrix/frontend/lib/neoland/server.ts` | Open | Add optional guards for partial ADR/full-pipeline documents in dashboard overview and analytics. |
| GAP-005 | High | Runtime stack | Open | Run and document full `Neoland -> SecureLLM Bridge -> ml-ops-api -> llama.cpp` smoke. |
| GAP-006 | High | Release discipline | Open | Create one canonical boot/health/task/session/ADR/restore checklist. |
| GAP-007 | Medium | TUI SSE | Open | Add timeout/degraded-state handling for long-dead SSE sockets. |
| GAP-008 | Medium | Cleanup | Open | Remove or archive tracked `.orig` backup files in a dedicated cleanup pass. |
| GAP-009 | Medium | Load/SLO | Open | Run `ghz`/REST load tests before claiming 500 RPS or p99 targets. |
| GAP-010 | Medium | Frontend | Open | Run full frontend lint/build and separate current code issues from legacy debt. |

---

## Current Validation

Verified:

```text
git status --short --branch
## main...github/main

just --list
Available recipes include server, client, doctor, test, frontend-dev, frontend-build,
frontend-lint, agents-start, test-agents-contract, secrets-view, validate.

cargo test --lib --quiet
test result: ok. 226 passed; 0 failed; 17 ignored; 0 measured; 0 filtered out
```

Still needed before calling this release-candidate:

- `cd agents && poetry run pytest tests/ -m contract -q`
- `nix-instantiate --parse < flake.nix` or equivalent Nix parse/flake check
- `nix develop --command frontend-lint`
- `nix develop --command frontend-build`
- full stack smoke with SecureLLM Bridge + ml-ops-api + llama.cpp
- backup/restore smoke for session/checkpoint artifacts

---

## Next Delivery Slice

Recommended order for the next integration pass:

1. Close GAP-002 and GAP-003 together: session API must fail honestly and survive schema drift.
2. Close GAP-001: make `doctor` diagnose control plane, DSPy, gateway, ml-ops, and llama.cpp as separate layers.
3. Close GAP-004: frontend analytics must tolerate incomplete ADR/checkpoint files.
4. Run Python contract tests and targeted Rust tests around the changed areas.
5. Run the first complete LLM runtime smoke and write the result into the runtime roadmap.
6. Convert the smoke into the release-candidate checklist.

---

## Score Policy

Do not raise the readiness score unless there is evidence:

- code path is connected to the real product flow;
- command/test/smoke output exists;
- docs and UI copy match behavior;
- failure mode is visible and recoverable;
- no stale Matrix/legacy claim is being treated as canonical product truth.

The score can go above 85 only after the full runtime stack and release smoke are repeatable.
