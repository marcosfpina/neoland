# AGENTS.md

This file defines the working guidelines for `~/master/neoland`.

## Mission

`neoland` is the primary repo.

It currently contains:
- the Rust control plane
- the Python DSPy multi-agent pipeline
- the existing TUI
- a moved-in `matrix/` frontend project that now serves as the base for Neoland's web surface

The current frontend goal is **not** a full rewrite.

The goal is:
- reposition the moved `matrix` frontend as the Neoland operational interface
- keep strong parts of the existing frontend infrastructure
- reorient the product around the Neoland control plane, sessions, ADRs, services, and pipeline execution

It is not:
- a generic chatbot project
- a marketing-first dashboard with shallow product depth
- a frontend that invents backend behavior not present in Neoland

## Current Direction

Treat `matrix/` as the frontend base that was moved into this repo.

Working assumption:
- we redesign and adapt it
- we do a brand repositioning first, not a deep technical rewrite first
- we do not fully refactor it up front
- we preserve useful infra, app structure, and UI components where possible
- we change narrative, navigation, modules, and information architecture to fit Neoland
- we implement against real Neoland backend and filesystem sources whenever possible, not mock data

The center of the product is:
- pipeline execution
- session inspection
- ADR/checkpoint rendering
- ecosystem/service health

The centerpiece is `Pipeline Live View`, not a chat box.

## Source Of Truth

Before implementing frontend behavior, verify backend truth in:
- `src/server/mod.rs`
- `src/agents/orchestrator.rs`
- `src/agents/client.rs`
- `src/agents/session.rs`
- `src/health.rs`
- `agents/neoland_agents/app.py`
- `agents/neoland_agents/schemas/api.py`
- `docs/ADR/ADR-019-multi-agent-dspy-pipeline.md`
- `docs/ADR/ADR-020-mmap-ipc.md`

If UI plans and backend contracts disagree:
- prefer the backend contract
- adapt the UI
- document the gap instead of guessing

## Repo Map

### Core Neoland

Backend/control-plane truth lives at the repo root:
- `src/` for Rust server, agents, config, health, TUI
- `agents/neoland_agents/` for the Python DSPy pipeline
- `docs/ADR/` for architecture decisions and future evolution

### Frontend Base

Frontend work currently starts from:
- `matrix/apps/frontend/`

Treat it as:
- the active redesign base
- the current source of reusable Next.js/shadcn/ui/frontend infrastructure

Do not treat `matrix/apps/backend/` as Neoland backend truth.

## Ownership Rules

Use this decision rule:
- if it is about orchestration truth, contracts, persistence, auth, health payloads, ADR generation, or agent behavior, root `neoland` owns it
- if it is about layout, component composition, navigation, operator flow, visual hierarchy, charts, and rendering, `matrix/apps/frontend/` owns it

## Frontend Product Positioning

The frontend should become a Neoland control-plane workbench.

Prioritize:
- running a task
- watching agent progression
- reading confidence, risk, escalation, and final decision
- inspecting session state
- rendering ADRs/checkpoints as readable operational artifacts
- seeing control-plane and ecosystem health clearly

Avoid:
- generic AI assistant framing
- vague “toolbox” navigation without operational meaning
- vanity dashboards disconnected from the control plane

## Current Frontend Tasks

When working on the frontend, prioritize in this order:

1. Reposition the brand and product language of `matrix/apps/frontend/` for Neoland
2. Replace generic or Matrix-specific navigation with Neoland operational navigation
3. Build or adapt the first strong vertical slice:
   - `Pipeline`
   - `Sessions`
   - `ADR Vault`
   - `Services`
4. Mirror real backend contracts in frontend types and adapters
5. Add degraded states and placeholders where the backend is not ready yet
6. Prefer empty, unavailable, or stub states over simulated data when the backend does not expose a contract yet

## Preferred UX Direction

The UI should feel like a modern operational console:
- technical
- readable under pressure
- fast to scan
- confident, not playful

Visual semantics:
- green for `approve` and healthy states
- red for `reject` and failure states
- amber/orange for `defer`, warnings, and degraded states
- violet for `escalate` and human-required states

Do not copy the TUI literally, but stay aligned with Neoland's technical identity.

## Integration Rules

Known verified backend routes:
- `POST /v1/agents/task`
- `GET /v1/agents/session/:id`
- `GET /v1/agents/health`
- `GET /health`
- `GET /ready`
- `GET /live`

Important rules:
- do not invent extra Neoland endpoints unless they are added in the backend
- mirror backend payloads in frontend types before using them in UI code
- prefer adapter layers over ad hoc reshaping inside React components

### ADRs

ADR/checkpoint rendering should prefer structured views over raw JSON dumps.

Prefer showing:
- title
- status
- context
- decision
- action items
- session summary
- meaningful excerpts from `full_pipeline`

## Redesign Guidance

When adapting the moved frontend base:
- keep useful infrastructure
- keep useful `components/ui`
- keep useful App Router and shared utility patterns
- remove or rewrite generic mission-control/productivity-tool language
- reposition naming, messaging, page framing, and visual semantics around Neoland
- remove or rewrite modules that do not map to Neoland's real product narrative

Bias toward selective brand repositioning and redesign, not wholesale replacement.

## Practical Default

Unless explicitly stated otherwise:
- work from the existing `matrix/apps/frontend/` base
- redesign instead of rewriting
- treat Neoland as the primary product identity
- validate every backend-facing assumption against the root repo first
