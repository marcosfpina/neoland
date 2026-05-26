# Neoland Roadmap To Responsible Public Release

**Last Updated**: 2026-05-17
**Current Reading**: **78/100 toward responsible public pre-release**
**Definition of prod for Neoland**: public access and public sharing with honest scope, safe defaults, visible limitations, and a recoverable operating path.
**Rule**: when roadmap text and code diverge, code wins and this file must be updated.

---

## Executive Summary

Neoland now has the major product pieces in place:

- Rust control plane with agent task/session APIs, SSE, health, metrics, OpenAPI, auth, audit, rate limiting, MCP, NATS, and storage hooks.
- Python DSPy pipeline with typed contracts, checkpoints, RAG hooks, mmap flags, and session checkpoint listing.
- TUI workstation for code-task execution with live stage output, breakpoints, steering, and history.
- Next.js workbench for pipeline execution, sessions, ADR inspection, services, and operational views.
- Nix-first DX with SOPS-aware `just` commands, frontend helpers, and NixOS modules for the Neoland + SecureLLM + ml-ops stack.

The roadmap is no longer about proving that the architecture can exist. It is
about making the integrated system honest, repeatable, and shareable.

---

## Current Release Blockers

| Priority | Blocker | Why It Blocks |
|----------|---------|---------------|
| P0 | Session API fragility | session serialization can hide failures; DB row mapping is brittle |
| P0 | Diagnostics still incomplete | `doctor` does not directly probe the DSPy pipeline layer |
| P0 | Full runtime stack not smoked | gateway -> ml-ops -> llama.cpp path is documented but not proven end-to-end |
| P1 | Frontend analytics can assume complete ADR payloads | partial checkpoint files can crash dashboard analytics |
| P1 | Release ritual is not canonical | deploy, health, task, session, ADR, backup, restore, and rollback are not one short routine |
| P2 | Historical docs still overstate readiness | older docs/runbooks remain useful but can mislead if read as current truth |

---

## Roadmap Phases

## Phase 0 - Source Of Truth And Codebase Map

**Status**: `done` for the current sync; keep updated continuously.

Done:

- [x] map Rust, Python, frontend, Nix, deploy, tests, ADRs, runbooks, and runtime ports;
- [x] update project snapshot;
- [x] update progress and roadmap around the live tree;
- [x] mark older archived docs as historical context;
- [x] align gateway naming with `NEOLAND_GATEWAY_URL`.

Ongoing rule:

- every delivery slice must update `docs/neoland-progress.md` when it changes status, gates, or release confidence.

## Phase 1 - Reliability Fixes For Core Contracts

**Status**: `next`
**Goal**: make the core session/diagnostic paths fail visibly and predictably.

- [ ] replace session JSON `unwrap_or_default()` with explicit 500/logging;
- [ ] replace tuple-index session row mapping with named mapping;
- [ ] add direct DSPy pipeline probe to `doctor`;
- [ ] add tests for session serialization failure and named session mapping if practical;
- [ ] update CLI/doctor docs after behavior lands.

Gate:

- `neoland doctor --json` distinguishes control plane, DSPy, gateway, ml-ops, llama.cpp, DB, Vault, and config status.
- session APIs never return silent `{}` on serialization failure.

## Phase 2 - Full Runtime Integration

**Status**: `in_progress`
**Goal**: prove the official LLM topology in one repeatable local/staging run.

Canonical topology:

```text
Neoland -> SecureLLM Bridge API -> ml-ops-api -> llama.cpp / vLLM
```

Work:

- [x] rename the client/gateway config surface to `neoland_gateway_url`;
- [x] preserve `NEOLAND_ML_API_URL` as compatibility alias;
- [x] export `NEOLAND_GATEWAY_URL` from dev shell and NixOS suite;
- [x] probe gateway health using `/api/health` before `/health`;
- [ ] boot SecureLLM Bridge with ml-ops provider enabled;
- [ ] boot ml-ops-api with `LLAMACPP_URL`;
- [ ] boot llama.cpp on the official local port;
- [ ] run a real task through the complete chain;
- [ ] record logs, health output, and failure behavior.

Gate:

- one command sequence can show `boot -> doctor -> task -> session -> ADR/checkpoint` with the official runtime topology.

## Phase 3 - Workbench And TUI Truthfulness

**Status**: `in_progress`
**Goal**: ensure the operator sees real state, not legacy or optimistic state.

Done:

- [x] frontend pipeline uses `PipelineRunner`;
- [x] frontend opens an SSE relay at `/api/neoland/events/[sessionId]`;
- [x] sessions page uses `GET /v1/agents/sessions`;
- [x] stats route uses ADR/checkpoint truth instead of Matrix as canonical source;
- [x] TUI parses `pipeline_started`, `stage_output`, `breakpoint_resolved`, and final buffered SSE events.

Work:

- [ ] guard frontend analytics against partial ADR payloads;
- [ ] run frontend lint/build and classify failures;
- [ ] add TUI SSE timeout/degraded-state handling;
- [ ] keep Matrix as optional telemetry, not core product truth;
- [ ] keep TUI focused on code tasks while the web console owns sessions, ADRs, services, and orchestration visibility.

Gate:

- a user can run a task and inspect live progress, final session, and ADR/checkpoint without mock data or misleading copy.

## Phase 4 - Single-Host Operations

**Status**: `planned`
**Goal**: turn the current stack into an operator routine instead of a set of ingredients.

Work:

- [ ] create a single release preflight script or checklist;
- [ ] validate restore of session/checkpoint artifacts;
- [ ] validate rollback from a failed deployment;
- [ ] document env/secrets for NixOS and Ubuntu bare metal;
- [ ] ensure health/readiness/liveness reflect true layer status;
- [ ] run Python contract tests, Rust targeted tests, and frontend build/lint as separate gates;
- [ ] decide what Docker/Helm must prove for first public release versus later HA work.

Gate:

- the stack can be started, checked, used, stopped, restored, and rolled back with a short documented path.

## Phase 5 - Release Candidate

**Status**: `planned`
**Goal**: freeze the first public-shareable shape.

Work:

- [ ] README and quickstart describe only verified flows;
- [ ] known limitations are visible;
- [ ] screenshots or demo path match the real workbench;
- [ ] security defaults are safe enough for first public exposure;
- [ ] release notes distinguish pre-release beta from enterprise/compliance backlog;
- [ ] all P0/P1 blockers are closed or explicitly accepted as non-release blockers.

Gate:

- a reviewer can follow the docs, run the product, see real progress, recover from common failure, and understand the limits without private tribal context.

## Phase 6 - Public Release

**Status**: `planned`
**Goal**: publish responsibly, observe behavior, and re-plan from actual use.

Work:

- [ ] tag and publish first public pre-release;
- [ ] run soak period with real tasks;
- [ ] review incidents and onboarding friction;
- [ ] re-open roadmap for multi-host, HA, richer analytics, ledger automation, and compliance depth.

---

## Tracking Board

| ID | Item | Status | Criticality |
|----|------|--------|-------------|
| MAP-1 | Full codebase map and source-of-truth docs | Done | High |
| CFG-1 | Canonical `NEOLAND_GATEWAY_URL` exported in dev shell/Nix suite | Done | High |
| SESS-1 | Explicit session serialization errors | Next | Critical |
| SESS-2 | Named session DB row mapping | Next | Critical |
| DOC-1 | Direct DSPy health in `doctor` | Next | Critical |
| RT-1 | SecureLLM Bridge + ml-ops + llama full smoke | Next | Critical |
| UI-1 | Frontend analytics optional guards | Next | High |
| TUI-1 | SSE timeout/degraded handling | Planned | Medium |
| OPS-1 | Canonical release preflight | Planned | High |
| OPS-2 | Backup/restore/rollback smoke | Planned | High |
| REL-1 | README/quickstart release honesty pass | Planned | High |
| CLEAN-1 | Remove/archive tracked backup `.orig` files | Planned | Medium |
| LOAD-1 | Load/SLO validation | Planned | Medium |

---

## Backlog After First Public Pre-Release

These should not block the first responsible release unless the target audience changes:

- multi-tenant auth and tenancy isolation;
- full SOC 2/GDPR/ISO documentation set;
- mTLS end-to-end across every plane;
- HA Kubernetes deployment with 3+ replicas;
- advanced ranking analytics;
- deeper ADR-ledger automation;
- mobile or desktop app packaging;
- vLLM as a required backend rather than optional acceleration.

---

## Production Criteria

Neoland is ready for the first public cut when all are true:

- P0 blockers are closed;
- `doctor` reports every runtime layer honestly;
- frontend and TUI show real pipeline/session/ADR state;
- one full runtime smoke proves the official gateway path;
- docs do not claim more than the system can demonstrate;
- installation and first run are clear for Nix/NixOS and one non-Nix path;
- backup/restore and rollback have a tested minimum routine;
- known limitations are explicit rather than explained verbally after failure.

---

## Next Recommended Work

1. Fix session serialization and session DB mapping.
2. Add direct DSPy health to `doctor`.
3. Guard frontend analytics for partial ADRs.
4. Run Rust/Python targeted tests.
5. Prove the official LLM runtime path.
6. Write the release preflight from that smoke.
