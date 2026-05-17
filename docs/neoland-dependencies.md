# Neoland Dependencies

## Overview

Neoland currently uses path-based dependencies for several local crates. This document explains the current state and the roadmap for production-ready dependency management.

## Current Path Dependencies

The following dependencies are currently referenced via local paths in `Cargo.toml`:

```toml
securellm-core = { path = "../securellm-bridge/crates/core" }
securellm-providers = { path = "../securellm-bridge/crates/providers" }
securellm-security = { path = "../securellm-bridge/crates/security" }
intelagent-core = { path = "../phantom-ray/phantom/intelagent/crates/core" }
hyprland-ipc = { path = "../ai-agent-os/crates/hyprland-ipc" }
```

## Current State (Phase 0)

| Dependency | Repository | Git Remote | Status |
|------------|-----------|------------|--------|
| securellm-* | securellm-bridge | ✓ (GitHub/GitLab) | Git repo available |
| intelagent-core | phantom-ray | ✗ | **Not a git repository** |
| hyprland-ipc | ai-agent-os | ✓ (GitHub) | Git repo available |

## Issues with Path Dependencies

1. **Build Reproducibility**: Path dependencies break reproducible builds because:
   - They require a specific directory structure on the build machine
   - No version pinning - changes in dependency crates can break builds unexpectedly
   - Cannot be cached or shared with remote build systems

2. **CI/CD**: Path dependencies complicate CI/CD pipelines:
   - Requires cloning multiple repositories
   - Difficult to manage dependency versions across repos
   - No automated dependency update mechanisms

3. **Collaboration**: Makes it harder for new developers:
   - Must clone multiple repositories in specific directory structure
   - Unclear which versions of dependencies are compatible

## Production Roadmap

### Phase 0: Foundation (Current)
- ✅ Document dependency structure
- ✅ Identify blockers (phantom-ray not a git repo)

### Phase 1: Git Migration (2-3 weeks)
1. **Convert phantom-ray to git repository**
   ```bash
   cd /home/kernelcore/arch/phantom-ray
   git init
   git add .
   git commit -m "Initial commit"
   git remote add origin <repo-url>
   git push -u origin main
   ```

2. **Tag releases for all dependencies**
   - securellm-bridge: Create v0.1.0 tag
   - phantom-ray: Create v0.1.0 tag
   - ai-agent-os: Create v0.1.0 tag

3. **Update Cargo.toml to use git dependencies**
   ```toml
   # Option A: Git dependencies
   securellm-core = { git = "https://github.com/org/securellm-bridge", tag = "v0.1.0" }
   intelagent-core = { git = "https://github.com/org/phantom-ray", tag = "v0.1.0" }
   hyprland-ipc = { git = "https://github.com/org/ai-agent-os", tag = "v0.1.0" }
   ```

### Phase 2: Private Registry (Optional, 4-6 weeks)
For better performance and reliability, consider publishing to a private Cargo registry:

1. **Set up private registry** (using Cloudsmith, Artifactory, or self-hosted)
2. **Publish crates to registry**
   ```bash
   cargo publish --registry neoland-private
   ```
3. **Update Cargo.toml**
   ```toml
   securellm-core = { version = "0.1.0", registry = "neoland-private" }
   ```

### Phase 3: Workspace Alternative (Optional)
Another approach is to create a Cargo workspace that includes all dependencies:

```toml
[workspace]
members = [
    "neoland",
    "securellm-bridge/crates/core",
    "securellm-bridge/crates/providers",
    "securellm-bridge/crates/security",
    "phantom-ray/phantom/intelagent/crates/core",
    "ai-agent-os/crates/hyprland-ipc",
]
```

This maintains monorepo benefits while keeping repositories separate.

## Development Setup

### Current Requirements

To build Neoland, you must have the following directory structure:

```
/home/kernelcore/arch/
├── neoland/                  # This repository
├── securellm-bridge/
│   └── crates/
│       ├── core/
│       ├── providers/
│       └── security/
├── phantom-ray/
│   └── phantom/
│       └── intelagent/
│           └── crates/
│               └── core/
└── ai-agent-os/
    └── crates/
        └── hyprland-ipc/
```

### Cloning All Dependencies

```bash
cd /home/kernelcore/arch

# Clone main repository
git clone <neoland-repo-url> neoland

# Clone dependencies
git clone <securellm-bridge-url> securellm-bridge
git clone <ai-agent-os-url> ai-agent-os

# Note: phantom-ray must be cloned manually or converted to git first
```

## Verification

To verify your dependency setup:

```bash
cd /home/kernelcore/arch/neoland
cargo check
```

If you see errors about missing crates, verify:
1. All dependency repositories are cloned
2. Directory structure matches the paths in Cargo.toml
3. Each dependency builds successfully on its own

## References

- [Cargo Book: Specifying Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)
- [RFC 2906: Cargo Workspaces](https://github.com/rust-lang/rfcs/blob/master/text/2906-cargo-workspace-deduplicate.md)
- [Production Readiness Roadmap](docs/ADR.md) - See ADR-012: "Dependency Management Strategy"

## Status

**Current Status**: Path dependencies (temporary for development)
**Target Status**: Git dependencies or private registry
**Blocker**: phantom-ray needs to be converted to a git repository
**Priority**: HIGH (blocks Phase 1: Security Hardening)
