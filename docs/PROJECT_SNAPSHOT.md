# Project Snapshot: Neoland

**Last Meta-Sync**: 2026-04-16  
**Primary Engine**: Rust + Python + TypeScript  
**Ecosystem Role**: Control Plane & Agent Orchestrator

---

## Technical Health (via Cerebro)

- **Overall Health Score**: 95.5 / 100
- **Total LoC**: 105,023
- **Primary Language**: TypeScript (Frontend: 25k) | Rust (Core: 13k) | Python (Agents: 6k)
- **Security Score**: 100.0 (SOPS Integrated)
- **Test Coverage**: High (21 core test files)

## Operational Architecture

- **Control Plane**: Port 3001 (Axum/Tonic)
- **Pipeline DSPy**: Port 8001 (FastAPI/mmap IPC)
- **Messaging**: NATS via Spectre (:4222)
- **Ledger**: ADR Vault (`docs/ADR/`)
- **Infrastructure**: Nix-first (Flake: `flake.nix`), Multi-env (Docker/Bare-metal)

## Active ADRs (Core)

1. [ADR-019: Multi-Agent DSPy Pipeline](file:///home/kernelcore/master/neoland/docs/ADR/ADR-019-multi-agent-dspy-pipeline.md)
2. [ADR-020: mmap IPC — Zero-Copy](file:///home/kernelcore/master/neoland/docs/ADR/ADR-020-mmap-ipc.md)
3. [ADR-021: Core Architecture Baseline](file:///home/kernelcore/master/neoland/docs/ADR/ADR-021-core-architecture-baseline.md)

## Active Milestones (ROADMAP.md)

- **Phase A**: IPC & Security (90% Complete)
- **Phase B**: Integration & Ledger (In Progress)
- **Phase C**: Real-time Forensics & Monitoring (Planned)

---

> [!NOTE]
> This snapshot is generated for context-aware agents and operational dashboards. For architectural details, consult the `docs/ADR/` vault.
