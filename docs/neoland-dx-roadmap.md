# Neoland DX Roadmap

**Last Updated**: 2026-05-17
**Objective**: verify that Neoland's real developer/operator experience matches
what the CLI, TUI, workbench, quickstart, and release docs promise.

---

## Current DX Reading

Neoland DX is now materially better than the older roadmap described:

- `just --list` exposes a clear command surface.
- `server`, `client`, and `doctor` recipes load SOPS secrets through `scripts/neoland-run.sh`.
- `*-raw` recipes exist for direct cargo runs without SOPS.
- frontend helper commands are exported by the Nix dev shell.
- Python agent helpers exist for starting agents and running contract/integration tests.
- `NEOLAND_GATEWAY_URL` is now the canonical gateway env, with `NEOLAND_ML_API_URL` kept as compatibility alias.

The remaining DX work is verification and truthfulness, not command invention.

---

## DX Principle

For each promised flow, track:

```text
promise -> evidence -> status -> next action
```

Status vocabulary:

- `verified`: tested in the current tree and safe to document as working.
- `partial`: implemented but not fully smoked or has caveats.
- `stale`: doc/UI copy no longer matches the code.
- `backlog`: intentionally not required for the first public pre-release.

---

## Active Workstreams

## Stream 1 - Command Surface

**Status**: `mostly_verified`

Evidence:

- `just --list` shows server/client/doctor/test/frontend/agents/secrets/validate commands.
- `justfile` documents SOPS-loaded and raw variants.
- `flake.nix` exposes wrapper commands and frontend helpers.

Next:

- [ ] run `just doctor-raw` or `neoland doctor --json` after the direct DSPy probe lands;
- [ ] keep README command examples aligned with `justfile`.

## Stream 2 - Core Journey Verification

**Status**: `partial`

Target journey:

1. enter dev shell;
2. load or provide secrets;
3. start server;
4. start DSPy agents;
5. run `doctor`;
6. submit a task;
7. inspect live stream;
8. inspect session;
9. inspect ADR/checkpoint;
10. stop and restore.

Next:

- [ ] turn the target journey into one release preflight;
- [ ] record the first full successful output in `docs/neoland-progress.md`.

## Stream 3 - TUI Truthfulness

**Status**: `partial`

Done:

- live SSE stage output;
- breakpoint resolved handling;
- visible steering/breakpoint failures;
- shell-like input history;
- code-task workstation direction.

Next:

- [ ] add long-silent SSE timeout/degraded handling;
- [ ] re-test first-run behavior after diagnostics work;
- [ ] keep non-code orchestration surfaces in the web console.

## Stream 4 - Frontend Truthfulness

**Status**: `partial`

Done:

- `/pipeline` uses real `PipelineRunner`;
- session registry consumes `GET /v1/agents/sessions`;
- live stream relay consumes `GET /v1/agents/events/:session`;
- stats route uses ADR/checkpoint truth instead of Matrix as canonical data.

Next:

- [ ] guard analytics for partial ADR documents;
- [ ] run frontend lint/build;
- [ ] keep Matrix telemetry visibly optional/non-canonical.

## Stream 5 - Environment DX

**Status**: `partial`

Nix/NixOS is the strongest path:

- dev shell;
- SOPS;
- wrappers;
- NixOS modules;
- integrated LLM suite module.

Ubuntu/bare-metal remains supported conceptually through env vars and local files,
but it needs a refreshed smoke path.

Next:

- [ ] document the minimal Ubuntu path without requiring SOPS;
- [ ] document where SOPS is preferred but not mandatory;
- [ ] verify the same command sequence outside Nix or label it as partial.

## Stream 6 - Release Honesty

**Status**: `in_progress`

Done:

- readiness recalibrated to 78/100;
- current project snapshot and roadmap updated;
- older archive/checkpoint docs marked as historical context.

Next:

- [ ] update README/quickstart public claims after the next fixes;
- [ ] remove or soften unverified SLO/performance claims;
- [ ] keep "pre-release beta" language until the full runtime smoke passes.

---

## Current DX Matrix

| Area | Current Promise | Status | Next Action |
|------|-----------------|--------|-------------|
| Dev shell | reproducible Nix environment | verified | keep command list current |
| SOPS secrets | preferred encrypted dotenv flow | verified for wrapper shape | keep env/local file as valid alternative |
| CLI doctor | environment diagnostics | partial | add direct DSPy probe |
| TUI | code-task live workstation | partial | timeout/degraded state and first-run polish |
| Pipeline workbench | real task + live stream | partial | verify with full stack and frontend build |
| Sessions | recent session registry | implemented | verify DB-backed flow in full smoke |
| ADR vault | checkpoint-backed decisions | partial | guard partial checkpoint analytics |
| Services | health/readiness/liveness map | partial | include runtime layers in release preflight |
| Runtime topology | SecureLLM -> ml-ops -> llama | partial | run end-to-end smoke |
| Release docs | honest public pre-release | in_progress | finish README/quickstart pass after fixes |

---

## Completion Criteria

DX is healthy when:

- the main flow can be run from docs without private context;
- diagnostics identify the broken layer;
- frontend and TUI represent real backend state;
- SOPS is preferred but not mandatory;
- Nix/NixOS and one non-Nix path are understandable;
- readiness claims are tied to test or smoke evidence.
