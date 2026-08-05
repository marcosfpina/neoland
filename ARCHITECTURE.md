# Architecture

## System Purpose

NEOLAND is the control plane for the VoidNX Labs platform. It orchestrates agents, mediates
LLM access, holds identity and authorization, and offloads machine-learning work. It is the
component that knows what is running and decides what runs next.

At 246 commits and the highest security posture score of the backbone (87/100), it is the most
actively developed service in the ecosystem.

## High-Level Overview

```
┌─────────────────────────────────────────────────────────┐
│ server/            HTTP surface                         │
│ auth/              identity, authorization (Ory Kratos)  │
└──────────────┬──────────────────────────────────────────┘
               ▼
┌─────────────────────────────────────────────────────────┐
│ agents/   runtime: checkpointing, escalation, flags,    │
│           mmap-backed state, NATS events                │
└───┬──────────────┬────────────────┬─────────────────────┘
    ▼              ▼                ▼
  llm/          mcp/            ml_offload/
  provider      MCP client      heavy compute
  access        integration     dispatch
    │              │                │
    ▼              ▼                ▼
 storage/       tools/           matrix/
 persistence    capability       matrix integration
                surface
               │
               ▼
        tui/  operator terminal UI
```

## Components

| Module | Files | Responsibility |
|---|---|---|
| `src/agents/` | 10 | Agent runtime: `checkpoint_store`, `client`, `escalation`, `events`, `flags`, `mmap`, `nats` |
| `src/tui/` | 9 | Operator terminal interface |
| `src/auth/` | 7 | Identity and authorization, Ory Kratos OIDC |
| `src/mcp/` | 5 | Model Context Protocol client integration |
| `src/llm/` | 4 | LLM provider access |
| `src/ml_offload/` | 3 | Offloading heavy ML work |
| `src/tools/` | 2 | Capability surface |
| `src/storage/` | 2 | Persistence |
| `src/matrix/` | 2 | Matrix integration |
| `src/server/` | 1 | HTTP entry point |

`adr/` holds in-repo architecture decisions; `.chain/` holds signature provenance; `benches/`
holds benchmarks; `agents/` holds agent definitions.

### Agent runtime

The agent subsystem is the substance of the control plane:

- `checkpoint_store.rs` — durable agent state so a restart resumes rather than restarts.
- `escalation.rs` — defined escalation paths when an agent cannot proceed.
- `flags.rs` — feature and behaviour gating per agent.
- `mmap.rs` — memory-mapped state for low-latency access.
- `nats.rs` / `events.rs` — publication to the SPECTRE event mesh.

## Data Flow

1. A request arrives at `server/`, authenticated through `auth/` against Ory Kratos.
2. The agent runtime selects or resumes an agent, loading state from `checkpoint_store`.
3. The agent acts through `llm/` (provider access), `mcp/` (tool invocation) or `ml_offload/`
   (heavy compute).
4. State changes are checkpointed; events are published to NATS for SPECTRE.
5. Escalation triggers when an agent exhausts its options.

## Trust Boundaries

| Boundary | Control |
|---|---|
| External → server | Ory Kratos OIDC authentication |
| Agent → tools | capability surface in `tools/`, gated by `flags/` |
| Agent → LLM | mediated through `llm/`, not direct provider calls |
| Control plane → mesh | one-way event publication |
| Secrets | SOPS-managed, never in the repository |

## Runtime Model

Async Rust on Tokio. Agents are long-lived and checkpointed, so the process can restart
without losing agent progress. `mmap` is used where state access is hot enough that
serialization overhead matters.

## Configuration

Environment and file-based. `build.rs` participates in the build. Secrets via SOPS.

## Storage

- Checkpoint store for agent state.
- PostgreSQL (`neoland-postgres` in the platform compose).
- Memory-mapped regions for hot agent state.

## External Integrations

| Target | Purpose |
|---|---|
| Ory Kratos | identity and OIDC |
| NATS / SPECTRE | event publication |
| securellm-bridge | LLM provider access |
| securellm-mcp | tool invocation |
| PostgreSQL | persistence |
| Matrix | messaging integration |

## Security Model

- Ory Kratos OIDC for identity; no bespoke credential handling.
- SOPS for secrets; `.sops.yaml` committed, secrets are not.
- `.chain/` provenance records signed artifacts.
- Full observability: structured logging, metrics endpoint, tracing, graceful shutdown, health
  probe — the most complete instrumentation among first-party services.
- Security posture 87/100, highest of the backbone.

## Testing Model

29 test files, plus `benches/` for performance. `cargo test` and `cargo bench`. The flake
exposes `packages`, `overlays` and `nixosModules`.

## Operational Notes

- Runs as `neoland-control-plane` in `deploy/docker-compose.master.yml`, with its own
  PostgreSQL and a DSPy pipeline sidecar.
- `azure-pipelines.yml` present alongside GitHub Actions.
- Build: `nix develop` then `cargo build --release`.

## Known Architectural Risks

1. **Architecture depth scores 28/100** — 10 modules, an `adr/` directory, but no recorded
   contracts between the agent runtime and the modules it dispatches to.
2. **Release discipline 48/100** — 1 tag against 246 commits. The most actively developed
   service in the ecosystem has essentially no release history, so dependents cannot pin.
3. **No operational runbook**, despite the strongest telemetry in the platform.
4. **`matrix/` is 2 files** and the migration status of the matrix integration is unresolved
   (see the ecosystem migration queue).
5. **`securellm-bridge` is vendored inside this repository** (`neoland/securellm-bridge`),
   which duplicates a top-level repo and makes the dependency direction ambiguous.
6. **Agent escalation paths are code, not policy.** There is no declarative record of what
   escalates where, which matters for a control plane.
