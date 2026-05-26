# Neoland LLM Runtime Roadmap

**Last Updated**: 2026-05-17
**Objective**: make Neoland's inference topology official, diagnosable, and
repeatable for the first responsible public pre-release.

---

## Architectural Decision

Canonical topology:

```text
Neoland
  -> SecureLLM Bridge API
    -> ml-ops-api
      -> llama.cpp
      -> vLLM (optional)
```

Meaning:

- Neoland's primary client-facing endpoint is the SecureLLM Bridge gateway.
- `ml-ops-api` is the inference bridge behind the gateway.
- `llama.cpp` and `vLLM` are backends behind `ml-ops-api`.
- direct `llama.cpp` access can exist for debugging, but it is not the product default.

---

## Current State

Done:

- `src/config.rs` and CLI now use `neoland_gateway_url` / `--neoland-gateway-url`.
- `NEOLAND_GATEWAY_URL` is canonical in dev shell and NixOS LLM suite wiring.
- `NEOLAND_ML_API_URL` remains supported as a compatibility alias.
- `doctor` probes SecureLLM Bridge using `/api/health` before `/health`.
- `doctor` optionally probes `ML_OPS_API_URL` and `LLAMACPP_URL` when declared.
- NixOS modules exist for `services.neoland`, `services.securellm-bridge-api`, `services.ml-ops-api`, and `services.neoland-llm-suite`.

Still open:

- direct DSPy `NEOLAND_DSPY_URL/health` is not yet part of `doctor`;
- a complete `Neoland -> SecureLLM Bridge -> ml-ops-api -> llama.cpp` task run has not been recorded in the current docs;
- frontend/services view does not yet represent every runtime layer as release-gate evidence;
- README/quickstart need a final pass after the smoke succeeds.

---

## Official Local Ports

| Layer | Role | Default |
|-------|------|---------|
| Neoland REST | control plane | `http://127.0.0.1:3001` |
| Neoland gRPC | internal gRPC | `http://[::1]:50051` |
| Frontend workbench | operator UI | `http://127.0.0.1:3006` |
| DSPy agents | agent pipeline | `http://127.0.0.1:8001` |
| SecureLLM Bridge API | LLM gateway | `http://127.0.0.1:8080` |
| ml-ops-api | inference bridge | `http://127.0.0.1:8083` |
| llama.cpp | local backend | `http://127.0.0.1:8081` |
| vLLM | optional backend | `http://127.0.0.1:8000` |

## Official Env Names

| Variable | Owner | Status | Purpose |
|----------|-------|--------|---------|
| `NEOLAND_GATEWAY_URL` | Neoland | canonical | SecureLLM Bridge endpoint consumed by Neoland |
| `NEOLAND_ML_API_URL` | Neoland | compatibility | legacy alias mapped to gateway URL |
| `NEOLAND_DSPY_URL` | Neoland | canonical | Python DSPy pipeline endpoint |
| `ML_OPS_API_URL` | SecureLLM Bridge | canonical | ml-ops upstream URL |
| `LLAMACPP_URL` | ml-ops-api | canonical | llama.cpp backend URL |
| `VLLM_URL` | ml-ops-api | optional | vLLM backend URL |

---

## Roadmap

## Phase 0 - Freeze Topology

**Status**: `done`

- [x] declare SecureLLM Bridge as the primary gateway;
- [x] declare ml-ops-api as the inference bridge;
- [x] keep `llama.cpp` behind ml-ops-api in the default path;
- [x] keep `NEOLAND_ML_API_URL` only as compatibility naming.

## Phase 1 - Naming Alignment

**Status**: `done`

- [x] code config surface renamed to `neoland_gateway_url`;
- [x] CLI arg is `--neoland-gateway-url`;
- [x] dev shell exports `NEOLAND_GATEWAY_URL`;
- [x] NixOS suite exports both canonical and compatibility env vars.

## Phase 2 - Health And Doctor

**Status**: `partial`

- [x] gateway health probes `/api/health` and `/health`;
- [x] ml-ops-api health is diagnosed when `ML_OPS_API_URL` exists;
- [x] llama.cpp health is diagnosed when `LLAMACPP_URL` exists;
- [ ] DSPy health is diagnosed directly from `NEOLAND_DSPY_URL`;
- [ ] doctor output names all layers in a way operators can act on immediately.

Gate:

- `neoland doctor --json` can identify which layer failed without reading logs first.

## Phase 3 - Bridge Bootstrap

**Status**: `next`

- [ ] start `securellm-api-server` with ml-ops provider enabled;
- [ ] pass `ML_OPS_API_URL=http://127.0.0.1:8083`;
- [ ] confirm `/api/health`;
- [ ] confirm OpenAI-compatible request path consumed by Neoland.

## Phase 4 - ml-ops Bootstrap

**Status**: `next`

- [ ] start ml-ops-api on `127.0.0.1:8083`;
- [ ] set `LLAMACPP_URL=http://127.0.0.1:8081`;
- [ ] confirm `/health` and `/api/health` behavior;
- [ ] confirm at least one backend is routable.

## Phase 5 - Backend And Forwarding

**Status**: `next`

- [ ] start llama.cpp on `127.0.0.1:8081`;
- [ ] validate `ml-ops-api -> llama.cpp`;
- [ ] validate `SecureLLM Bridge -> ml-ops-api -> llama.cpp`;
- [ ] decide whether vLLM is release backlog or optional smoke.

## Phase 6 - End-To-End Runtime Smoke

**Status**: `planned`

- [ ] start Neoland server;
- [ ] start DSPy agents;
- [ ] start SecureLLM Bridge;
- [ ] start ml-ops-api;
- [ ] start llama.cpp;
- [ ] run `neoland doctor --json`;
- [ ] submit a real task from TUI or workbench;
- [ ] inspect session and ADR/checkpoint;
- [ ] record the command sequence and output in progress docs.

## Phase 7 - Release Discipline

**Status**: `planned`

- [ ] convert the smoke into a release preflight;
- [ ] document failure modes by layer;
- [ ] update README/quickstart after proof;
- [ ] separate first-release requirements from HA/enterprise backlog.

---

## Tracking

| ID | Item | Status | Criticality |
|----|------|--------|-------------|
| LLM-1 | Freeze canonical topology | Done | High |
| LLM-2 | Rename Neoland gateway naming | Done | High |
| LLM-3 | Export canonical gateway env in Nix | Done | High |
| LLM-4 | Gateway/ml-ops/llama doctor probes | Partial | High |
| LLM-5 | Direct DSPy doctor probe | Next | Critical |
| LLM-6 | Bootstrap SecureLLM Bridge with ml-ops | Next | Critical |
| LLM-7 | Bootstrap ml-ops-api with llama.cpp | Next | Critical |
| LLM-8 | Complete E2E task smoke | Planned | Critical |
| LLM-9 | Release preflight from smoke | Planned | High |

---

## Completion Criteria

This roadmap closes when:

- `NEOLAND_GATEWAY_URL` is the visible default everywhere;
- `doctor` identifies control plane, DSPy, gateway, ml-ops, llama.cpp, DB, Vault, and config;
- SecureLLM Bridge and ml-ops-api run as the official inference path;
- a real task completes through the gateway chain;
- docs show exact ports, env vars, commands, health checks, and known failures.
