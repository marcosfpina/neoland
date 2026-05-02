# Contributing to Neoland

**Last Updated**: 2026-04-26

Thank you for your interest in contributing to Neoland! This guide outlines the development workflow, code standards, and best practices for contributing effectively.

---

## Table of Contents

1. [Project Overview & Stack](#project-overview--stack)
2. [Getting Started](#getting-started)
3. [Development Workflow](#development-workflow)
4. [Commit Message Conventions](#commit-message-conventions)
5. [Code Style](#code-style)
6. [Testing Requirements](#testing-requirements)
7. [Pre-commit Hooks](#pre-commit-hooks)
8. [Architecture Overview](#architecture-overview)
9. [How to Get Help](#how-to-get-help)

---

## Project Overview & Stack

Neoland is a terminal-based AI assistant built in Rust, designed as a foundational component of a larger AI agent ecosystem. It features a modern async TUI, multi-agent DSPy pipeline, comprehensive security hardening, and deep NixOS integration.

### Technology Stack

| Layer               | Technology                          | Purpose                     |
| ------------------- | ----------------------------------- | --------------------------- |
| **CLI**             | `clap` 4.5                          | Argument parsing            |
| **TUI**             | `ratatui` 0.28                      | Terminal rendering          |
| **Input**           | `crossterm` 0.28                    | Cross-platform terminal I/O |
| **RPC**             | `tonic` 0.12 + `prost` 0.13         | gRPC server/client          |
| **REST**            | `axum` 0.7                          | HTTP API                    |
| **Async Runtime**   | `tokio` 1.40                        | async/await executor        |
| **Inference**       | `candle` 0.8                        | Pure Rust ML framework      |
| **Build**           | Nix Flakes                          | Reproducible builds         |
| **Secrets**         | SOPS + HashiCorp Vault              | Encrypted secrets           |
| **Python Agents**   | Python 3.13+ / DSPy / FastAPI       | Multi-agent pipeline        |
| **Event Bus**       | NATS JetStream                      | Inter-service messaging     |

---

## Getting Started

### Prerequisites

- [Nix](https://nixos.org/download.html) with flakes enabled
- 2GB+ free RAM (for local inference)
- A terminal emulator (Alacritty, Kitty, or similar)

### 1. Clone the Repository

```bash
git clone git@github.com:voidnxlabs/neoland.git
cd neoland
```

### 2. Enter the Development Shell

**Option A — Nix develop (recommended):**

```bash
nix develop
```

This provides all dependencies: Rust toolchain, Python environment, protoc, and system libraries.

**Option B — Direnv (automatic shell activation):**

```bash
# One-time setup
cp .envrc.example .envrc   # or just ensure use flake is in .envrc
direnv allow

# From now on, entering the project directory automatically loads the dev shell
```

### 3. Build the Project

```bash
# Release build
cargo build --release

# Debug build (faster iteration)
cargo build
```

### 4. Run the Server

```bash
# Start gRPC (50051) and REST (3001) server
neoland server

# Or via cargo
cargo run -- server
```

### 5. Launch the TUI Client

```bash
# In another terminal
neoland client

# Or via cargo
cargo run -- client --ml-api-url http://localhost:9000
```

### 6. Run Diagnostics

```bash
neoland doctor --json
neoland test
```

---

## Development Workflow

### Branch Strategy

We follow a **trunk-based development** model with short-lived feature branches:

```text
main  ──── feat/xxx ── PR ──► main
         └─ fix/xxx ── PR ──► main
         └─ refactor/xxx ─ PR ──► main
```

| Branch Prefix   | Purpose                          |
| --------------- | -------------------------------- |
| `feat/`         | New feature or enhancement       |
| `fix/`          | Bug fix                          |
| `refactor/`     | Code restructuring (no behavior change) |
| `docs/`         | Documentation only               |
| `chore/`        | Maintenance, CI, dependencies    |
| `security/`     | Security fixes or hardening      |

### Workflow Steps

1. **Sync with main:**

   ```bash
   git checkout main
   git pull --rebase
   ```

2. **Create a feature branch:**

   ```bash
   git checkout -b feat/my-feature
   ```

3. **Make changes and commit** (see [Commit Message Conventions](#commit-message-conventions)):

   ```bash
   git add <files>
   git commit
   ```

4. **Keep your branch up to date:**

   ```bash
   git fetch origin
   git rebase origin/main
   ```

5. **Run the full validation suite:**

   ```bash
   cargo clippy --all-targets -- -D warnings
   cargo test --lib
   cargo fmt -- --check
   ```

6. **Push and open a Pull Request:**

   ```bash
   git push -u origin feat/my-feature
   ```

   Then open a PR on GitHub against `main`.

### Pull Request Guidelines

- **Title**: Brief description of the change (max 72 chars).
- **Description**: What, why, and how. Link related issues and ADRs.
- **Size**: Keep PRs focused on a single concern. If a change touches multiple areas, consider splitting it.
- **Checklist**:
  - [ ] Code compiles without warnings
  - [ ] `cargo clippy` passes with `-D warnings`
  - [ ] `cargo fmt` is applied
  - [ ] All existing tests pass
  - [ ] New tests are added for new functionality
  - [ ] Documentation is updated (if applicable)
  - [ ] ADR is created for architectural decisions (see `docs/ADR.md`)

---

## Commit Message Conventions

We use [Conventional Commits](https://www.conventionalcommits.org/) to enable automatic changelog generation and semantic versioning.

### Format

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Types

| Type       | Description                                     |
| ---------- | ----------------------------------------------- |
| `feat`     | A new feature                                   |
| `fix`      | A bug fix                                       |
| `refactor` | Code restructuring without feature/bug change   |
| `docs`     | Documentation only                              |
| `style`    | Formatting, missing semicolons, etc. (no logic change) |
| `test`     | Adding or improving tests                       |
| `chore`    | Build process, CI, dependency updates           |
| `perf`     | Performance improvement                         |
| `security` | Security hardening                              |
| `ci`       | CI/CD pipeline changes                          |

### Scopes

| Scope         | Area                  |
| ------------- | --------------------- |
| `tui`         | Terminal UI module    |
| `server`      | gRPC/REST server      |
| `cli`         | CLI argument parsing  |
| `auth`        | Authentication/RBAC   |
| `secrets`     | Secrets management    |
| `audit`       | Audit logging         |
| `llm`         | LLM integration       |
| `ml-offload`  | ML offload client     |
| `agents`      | Python/DSPy agents    |
| `nix`         | Nix build & modules   |
| `docs`        | Documentation         |
| `ci`          | CI/CD pipeline        |

### Examples

```text
feat(tui): add Ctrl+W word-delete shortcut

Implement readline-style backward word deletion using Alt+Backspace
mapping in the events handler.

Closes #142
```

```text
fix(auth): validate API key length before HMAC comparison

Prevent timing side-channel when key lengths differ.

Fixes SEC-202
```

```text
docs: add CONTRIBUTING.md

Detailed contribution guide covering workflow, conventions,
testing, and architecture.
```

---

## Code Style

### Rust

We enforce consistent formatting and linting with:

- **`rustfmt`** — Formatting (see `rustfmt.toml`)
- **`clippy`** — Linting (see `clippy.toml`)

Both are checked by the pre-commit hook and CI.

#### Key Formatting Rules

```toml
# rustfmt.toml — excerpt
max_width = 100
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
use_small_heuristics = "Default"
```

#### Key Linting Rules

```toml
# clippy.toml — excerpt
cognitive-complexity-threshold = 30
too-many-arguments-threshold = 7
disallowed-names = ["foo", "bar", "baz", "test"]
```

#### Style Guidelines

1. **Imports**: Group in order: `std` → external crates → `crate` (enforced by `group_imports`).
2. **Error handling**: Use `anyhow::Result` for application-level errors, `thiserror` for library errors.
3. **Naming**: `snake_case` for functions/variables, `PascalCase` for types/enums, `SCREAMING_SNAKE_CASE` for constants.
4. **Unsafe code**: Avoid unless absolutely necessary. Document every `unsafe` block with a safety comment.
5. **Comments**: Prefer self-documenting code over comments. When commenting, explain *why*, not *what*.
6. **Documentation**: Public API items must have doc comments (`///`). Use `//` for internal comments.

### Python (Agents)

The Python agents codebase in `agents/` follows:

- **Line length**: 100 chars (configured in `pyproject.toml` via `[tool.ruff]`)
- **Formatting**: [Ruff](https://docs.astral.sh/ruff/) formatter
- **Type hints**: Required for all function signatures
- **Imports**: Standard library → third-party → local (alphabetical within groups)

```bash
# Format Python code
cd agents
ruff format .

# Lint Python code
ruff check .
```

---

## Testing Requirements

All tests must pass before merging. We maintain a strict zero-mock policy for contract tests.

### Test Suites

| Suite              | Command                                              | Count | Notes                     |
| ------------------ | ---------------------------------------------------- | ----- | ------------------------- |
| Rust unit tests    | `cargo test --lib`                                   | 198   | Core + security + TUI     |
| Rust all tests     | `cargo test`                                         | —     | Includes integration      |
| CLI tests          | `cargo test cli::tests`                              | —     | CLI-specific unit tests   |
| Python contract    | `cd agents && pytest tests/ -m contract -v`          | 24    | Schemas, no LLM required  |
| Python integration | `cd agents && pytest tests/ -m integration -v`       | —     | Requires `LLM_API_KEY`    |
| Code coverage      | `cargo tarpaulin --lib`                              | —     | Target: 80%               |

### Running Tests

```bash
# Quick check — unit tests only
just test

# Full Rust suite
just test-all

# CLI-specific tests
just test-cli

# Python contract tests (no LLM needed)
just agents-test-contract

# All Python tests
just agents-test-integration

# Coverage report
just coverage
```

### Testing Guidelines

1. **Unit tests**: Test individual functions and modules in isolation. Place in a `mod tests` block at the bottom of the source file.
2. **Integration tests**: Test end-to-end flows across modules. Place in `tests/` directory.
3. **Contract tests** (Python): Validate Pydantic schemas and IPC data structures. Must not require an LLM.
4. **No mocks in contract tests**: Use real data structures and validate against the actual schemas.
5. **New code must include tests**: A PR without tests for new functionality will not be merged.
6. **Coverage target**: Aim for 80%+ overall coverage, 90%+ for security-critical modules.

---

## Pre-commit Hooks

The repository includes a pre-commit hook at `.githooks/pre-commit` that runs automatically:

```bash
# The hook runs on every `git commit` and executes:
# 1. cargo fmt --all -- --check      — Format check
# 2. cargo clippy --all-targets ...   — Lint check
# 3. cargo test --lib                 — Unit tests
# 4. cargo check --all-targets        — Compilation check
```

If you're outside a Nix shell, the hook automatically wraps itself in `nix develop`.

### Manual Setup

```bash
# The hook path is already configured by setup-hooks.sh
just setup-hooks
```

### Bypassing Hooks (Not Recommended)

```bash
git commit --no-verify
```

Only use this for emergency fixes or when you're certain the changes are safe. CI will catch any issues.

---

## Architecture Overview

For a full technical deep-dive, see [ARCHITECTURE.md](ARCHITECTURE.md).

### High-Level Structure

```text
neoland (Unified CLI Binary)
├── server       — gRPC + REST API server (tonic, axum)
├── client       — Modern TUI (ratatui + crossterm)
├── test         — Health check suite
├── restart      — Process management
└── doctor       — Environment diagnostics
```

### Module Layout

```text
src/
├── bin/neoland.rs       # Unified CLI entrypoint (clap)
├── lib.rs               # Library root
├── cli.rs               # Argument parsing
├── config.rs            # Config loading (env → CLI)
├── server/              # gRPC/REST server with security middleware
├── tui/                 # Terminal UI (MVC pattern)
├── ml_offload/          # External ML API client (OpenAI-compatible)
├── llm/                 # SecureLLM integration
├── agents/              # DSPy multi-agent pipeline (Python)
├── storage/             # Persistent storage (SQLite + pgvector)
├── mcp/                 # Model Context Protocol
├── matrix/              # Matrix protocol integration
├── tools/               # Tool execution
├── auth.rs              # Authentication & RBAC
├── secrets.rs           # HashiCorp Vault + SOPS
├── audit.rs             # Structured audit logging
├── validation.rs        # Input validation & sanitization
├── engine.rs            # Local inference (Candle)
├── metrics.rs           # Prometheus metrics
├── health.rs            # Health check endpoints
├── nlp.rs               # Vector store (RAG)
└── hyprland_ops.rs      # Window manager IPC
```

### Design Patterns

| Pattern           | Location                        | Purpose                         |
| ----------------- | ------------------------------- | ------------------------------- |
| Builder           | `cli.rs`                        | CLI argument construction       |
| State Machine     | `tui/events.rs`                 | Keyboard event routing          |
| Repository        | `server/mod.rs`                 | Thread-safe state isolation     |
| Adapter           | `ml_offload/`                   | OpenAI-compatible API adaptor   |
| Proxy             | `llm/proxy.rs`                  | SecureLLM security layer        |
| Fallback Chain    | `tui/mod.rs`                    | LLM provider failover           |

### Key Architectural Decisions

All major decisions are recorded as Architecture Decision Records (ADRs) in `docs/ADR.md`. Notable ADRs:

- **ADR-001**: 3-Layer Architecture (Infra/Security/Compliance)
- **ADR-002**: LocalFirst LLM Routing Strategy
- **ADR-003**: SecureLLM Proxy with Factory Pattern
- **ADR-007**: Neutron Integration (Planned Q2 2026)

---

## How to Get Help

### Internal Channels

- **GitHub Issues**: Use the [issue tracker](https://github.com/voidnxlabs/neoland/issues) for bugs, feature requests, and questions.
- **VoidNxSEC Team**: Contact the architecture team for design discussions.

### Documentation

| Resource                   | Location                   |
| -------------------------- | -------------------------- |
| Architecture Overview      | `ARCHITECTURE.md`          |
| Quick Start Guide          | `QUICKSTART.md`            |
| ADR Ledger                 | `docs/ADR.md`              |
| Security Documentation     | `docs/AUTHENTICATION.md`   |
| Vault Setup                | `docs/VAULT_SETUP.md`      |
| SOPS Setup                 | `docs/SOPS_SETUP.md`       |
| API Documentation          | Swagger UI at `/swagger-ui/` |
| ROADMAP                    | `ROADMAP.md`               |
| PROGRESS                   | `docs/runbooks/PROGRESS.md` |

### Troubleshooting

```bash
# Run environment diagnostics
neoland doctor --json

# Check server health
neoland test --json

# Restart the server
neoland restart
```

For common issues, see the [Troubleshooting](QUICKSTART.md#troubleshooting) section in the Quick Start guide.

---

## License

This project is proprietary — internal research project of VoidNxSEC.

---

**Maintained by**: VoidNxSEC Architecture Team
**Review Cycle**: Quarterly
```
````

Now let me create the remaining files.