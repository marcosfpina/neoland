# Neoland Active Improvement Backlog

**Last Updated**: 2026-05-17
**Purpose**: keep the next implementation queue aligned with the mapped codebase,
not older production-readiness phases.

---

## Critical: Contract And Diagnostics

- [ ] **SESS-1: explicit session serialization errors**
  - Area: `src/server/mod.rs`
  - Problem: `serde_json::to_value(&session).unwrap_or_default()` can turn a serialization failure into a misleading `{}` response.
  - Acceptance: session handlers return a clear 500 with log context when serialization fails.

- [ ] **SESS-2: robust session row mapping**
  - Area: `src/agents/session.rs`
  - Problem: tuple-index mapping is fragile if the DB projection changes.
  - Acceptance: session metadata is mapped by named fields or an equivalent resilient pattern.

- [ ] **DOC-1: direct DSPy doctor probe**
  - Area: `src/commands.rs`
  - Problem: `doctor` diagnoses control plane, gateway, ml-ops, llama, DB, and Vault, but not the Python DSPy service directly.
  - Acceptance: `neoland doctor --json` includes a distinct DSPy pipeline result using `NEOLAND_DSPY_URL`.

- [ ] **RT-1: full LLM runtime smoke**
  - Area: `flake.nix`, `modules/applications/*`, external services
  - Problem: the official topology is documented but not proven in one boot/task run.
  - Acceptance: documented output for `Neoland -> SecureLLM Bridge -> ml-ops-api -> llama.cpp`.

---

## High: Operator Surface

- [ ] **UI-1: partial ADR guards**
  - Area: `matrix/frontend/lib/neoland/server.ts`
  - Problem: analytics assumes complete `full_pipeline` payloads.
  - Acceptance: dashboard overview and analytics render neutral values for partial checkpoints.

- [ ] **UI-2: frontend verification pass**
  - Area: `matrix/frontend`
  - Problem: current source has not been validated with full lint/build during this sync.
  - Acceptance: `frontend-lint` and `frontend-build` results are recorded, with legacy debt separated from newly touched code.

- [ ] **TUI-1: SSE degraded timeout**
  - Area: `src/tui/mod.rs`
  - Problem: a dead-but-open SSE socket can leave the TUI looking connected.
  - Acceptance: long-silent streams degrade visibly and recovery guidance is shown.

---

## High: Release Operations

- [ ] **OPS-1: canonical preflight**
  - Area: `scripts/`, `docs/runbooks/`, `docs/neoland-quickstart.md`
  - Problem: validation exists in pieces.
  - Acceptance: one short preflight covers boot, health, task, session, ADR, and service visibility.

- [ ] **OPS-2: backup/restore smoke**
  - Area: `scripts/backup/`, checkpoint storage
  - Problem: backup scripts exist, but restore is not part of the normal release gate.
  - Acceptance: restore of at least one checkpoint/session artifact is demonstrated and documented.

- [ ] **OPS-3: release honesty pass**
  - Area: `README.md`, quickstart, docs index
  - Problem: older docs still contain inflated readiness language.
  - Acceptance: public-facing docs describe pre-release beta status, verified flows, and limitations.

---

## Medium: Cleanup And Hardening

- [ ] **CLEAN-1: tracked backup files**
  - Area: `src/tui/mod.rs.orig`, `agents/neoland_agents/tools.py.orig`
  - Problem: tracked `.orig` files make the active map noisier.
  - Acceptance: remove or archive in a dedicated cleanup commit after confirming they are not needed.

- [ ] **LOAD-1: load/SLO validation**
  - Area: `tests/load/`, `benches/`
  - Problem: 500 RPS and p99 latency targets are not validated for the current stack.
  - Acceptance: record REST/gRPC load results and adjust README claims.

- [ ] **SEC-1: edge hardening cut**
  - Area: auth, deployment docs
  - Problem: enterprise backlog and first-release security minimum are mixed.
  - Acceptance: first-release security minimum is explicit; enterprise items stay backlog.

---

## Recently Closed Or Reclassified

- [x] `VectorStore` unbounded-memory concern is no longer the main active blocker; persistent storage exists and release work should focus on integration gates.
- [x] `neoland doctor` now probes the SecureLLM gateway with `/api/health` before `/health`.
- [x] frontend pipeline is no longer an "awaiting backend contract" stub.
- [x] session browsing is no longer lookup-only; `/v1/agents/sessions` exists and the workbench consumes it.
- [x] `ml_api_url` naming was re-scoped to `neoland_gateway_url`; legacy env remains as an alias.

---

## Working Rule

When a backlog item closes, update:

1. [`docs/neoland-progress.md`](neoland-progress.md)
2. [`docs/neoland-roadmap.md`](neoland-roadmap.md)
3. any specific side roadmap that owns the area

No readiness score increase without test, smoke, or runtime evidence.
