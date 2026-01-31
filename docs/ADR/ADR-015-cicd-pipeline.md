# ADR-015: CI/CD Pipeline Architecture

**Date**: 2026-01-31
**Status**: ✅ Accepted
**Priority**: HIGH
**Complexity**: Medium
**Impact**: Build/Deploy

## Context

Phase 3 of the Production Readiness Roadmap requires implementing automated CI/CD pipelines to ensure code quality, prevent regressions, and enable safe, frequent deployments.

**Requirements**:
- Automated testing on every push/PR
- Code quality enforcement (formatting, linting)
- Reproducible builds (Nix)
- Pre-commit hooks for local development
- Fast feedback loops (<5 minutes)

**Constraints**:
- Must work with Nix-based build system
- Support monorepo structure (neoland + dependencies)
- Run in GitHub Actions (free tier)
- Cachix integration for build caching

## Decision

Implement a 3-tier CI/CD architecture:

### 1. Pre-commit Hooks (Local Development)

**Tool**: Custom bash scripts in `.githooks/`
**Execution**: Before git commit

**Checks**:
1. **Format check** (cargo fmt): Ensures consistent code style
2. **Linting** (clippy): Catches common mistakes and anti-patterns
3. **Unit tests** (cargo test --lib): Fast test suite
4. **Compilation** (cargo check): Validates code compiles

**Nix Integration**:
```bash
# Auto-enter nix environment if not already inside
if [ -f "$PROJECT_ROOT/flake.nix" ] && [ -z "$IN_NIX_SHELL" ]; then
    exec nix develop -c .githooks/pre-commit "$@"
fi
```

**Setup**: `./scripts/setup-hooks.sh` (one-time)

### 2. GitHub Actions Workflows

#### Workflow: `test.yml` (Automated Testing)
- **Trigger**: Push to any branch, pull requests
- **Jobs**:
  - **Unit tests**: `cargo test --lib --no-fail-fast`
  - **Integration tests**: `cargo test --tests --test-threads=1`
- **Caching**: Cachix for Nix store
- **Timeout**: 15 minutes

#### Workflow: `lint.yml` (Code Quality)
- **Trigger**: Push to any branch, pull requests
- **Jobs**:
  - **Format check**: `cargo fmt --check` (fail if not formatted)
  - **Clippy**: `cargo clippy -- -D warnings` (fail on warnings)
  - **Compilation check**: `cargo check --all-targets`
- **Timeout**: 10 minutes

#### Workflow: `build.yml` (Release Builds)
- **Trigger**: Push to main, version tags (`v*`)
- **Jobs**:
  - **Build release binary**: `nix build .#neoland`
  - **Build dev binary**: `cargo build --bin neoland`
  - **Binary size check**: Report debug binary size
  - **Upload artifacts**: 7-day retention
- **Cachix**: Both read and write
- **Timeout**: 20 minutes

### 3. Configuration Files

#### `rustfmt.toml` (Code Formatting)
```toml
edition = "2021"
max_width = 100
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
reorder_imports = true
```

**Rationale**:
- 100-char line width (readable on modern displays)
- Crate-level import granularity (cleaner diffs)
- Group stdlib/external/internal imports

**Nightly features**: Disabled (using stable Rust 2021)

#### `clippy.toml` (Linting Thresholds)
```toml
cognitive-complexity-threshold = 30
too-many-arguments-threshold = 7
too-many-lines-threshold = 150
disallowed-names = ["foo", "bar", "baz", "test"]
```

**Rationale**:
- Cognitive complexity: 30 (allow moderate complexity)
- Max 7 arguments (suggest refactoring beyond this)
- Max 150 lines per function (maintainability)
- Ban placeholder names in production code

## Alternatives Considered

### 1. Use `pre-commit` framework (Python-based)
- **Pros**: Rich ecosystem, language-agnostic
- **Cons**: Extra dependency (Python), overkill for Rust-only project
- **Rejected**: Custom bash hooks are simpler

### 2. Travis CI / CircleCI
- **Pros**: Mature CI platforms
- **Cons**: GitHub Actions integrates better, Cachix support
- **Rejected**: GitHub Actions is sufficient and free

### 3. Enforce formatting in CI only (no pre-commit hooks)
- **Pros**: Simpler developer workflow
- **Cons**: Wastes CI time on trivial issues, slower feedback
- **Rejected**: Pre-commit hooks catch issues faster

### 4. Run all checks in single CI job
- **Pros**: Simpler workflow configuration
- **Cons**: Slower feedback (can't parallelize), all-or-nothing
- **Rejected**: Separate workflows allow parallelization

## Implementation Details

### Pre-commit Hook Flow
```
Developer commits code
    ↓
.githooks/pre-commit triggered
    ↓
Check if IN_NIX_SHELL set?
    ↓ No
Exec: nix develop -c .githooks/pre-commit
    ↓ Yes
Run checks: fmt → clippy → tests → check
    ↓
Any failures? → Block commit, show error
    ↓ No
Allow commit to proceed
```

### CI Parallelization
```
Push to GitHub
    ↓
Trigger 3 workflows in parallel:
    ├─ test.yml (5-8 min)
    ├─ lint.yml (3-5 min)
    └─ build.yml (8-12 min)
    ↓
All pass? → PR approved
Any fail? → Block merge
```

### Cachix Integration
```yaml
- uses: cachix/cachix-action@v14
  with:
    name: neoland
    authToken: '${{ secrets.CACHIX_AUTH_TOKEN }}'
```

**Benefits**:
- ~80% faster builds (Nix cache hits)
- Shared cache across all workflows
- Cache persists across PRs

## Consequences

### Positive
✅ **Automated quality gates**: No untested code in main
✅ **Fast feedback**: Pre-commit hooks catch issues in <10s
✅ **Reproducible builds**: Nix ensures consistency
✅ **Parallel CI**: 3 workflows run simultaneously
✅ **Low cost**: GitHub Actions free tier (2000 min/month)
✅ **Developer productivity**: Catch bugs before PR

### Negative
⚠️ **Pre-commit overhead**: ~10s delay per commit
⚠️ **Nix dependency**: Developers must install Nix
⚠️ **CI queue time**: Busy repos may have wait times
⚠️ **Cache invalidation**: Nix cache sometimes rebuilds everything

### Mitigations
- Pre-commit hooks can be bypassed with `git commit --no-verify` (not recommended)
- Nix installation is one-time setup
- Cachix significantly reduces CI time

## Compliance & Security

### SOC 2 Controls
- **CC8.1**: Code changes require automated testing
- **CC7.2**: Version control with automated checks

### OWASP ASVS
- **V14.2.6**: Build pipeline integrity (Nix reproducibility)

### Best Practices
- ✅ Fail-fast: Clippy errors block commits
- ✅ Deterministic builds: Nix flake lock
- ✅ Automated regression testing
- ✅ Code review required (GitHub branch protection)

## Metrics & Monitoring

### Success Criteria
- ✅ CI pass rate: >95%
- ✅ Average CI duration: <10 minutes
- ✅ Pre-commit hook usage: >80% of developers
- ✅ Zero broken main builds

### Dashboards
- GitHub Actions: Built-in workflow analytics
- Cachix: Cache hit rate monitoring

## Testing & Validation

### Pre-commit Hook Test
```bash
# Setup hooks
./scripts/setup-hooks.sh

# Test formatting check
echo "  " >> src/main.rs  # Add trailing whitespace
git add src/main.rs
git commit -m "test"  # Should fail with format error

# Test clippy check
# (Add clippy warning and test)

# Test unit tests
# (Break a unit test and verify hook blocks commit)
```

### CI Workflow Test
```bash
# Trigger test workflow
git push origin feature-branch

# Verify all 3 workflows start
gh workflow view --ref feature-branch

# Check Cachix cache hit
# (Should see "cache hit" in nix build logs)
```

## Rollout Plan

### Phase 1: Initial Setup ✅
- ✅ Create workflow files
- ✅ Create pre-commit hooks
- ✅ Configure rustfmt/clippy
- ✅ Document setup in scripts/setup-hooks.sh

### Phase 2: Developer Onboarding (Week 1)
- [ ] Update README.md with CI/CD section
- [ ] Add "Running CI locally" guide
- [ ] Team training on hook setup

### Phase 3: Enforcement (Week 2)
- [ ] Enable branch protection on main
- [ ] Require CI checks to pass before merge
- [ ] Block force-push to main

## Future Enhancements

### Short-term (4-8 weeks)
- [ ] Code coverage reporting (cargo-tarpaulin)
- [ ] Automated dependency updates (Dependabot/Renovate)
- [ ] Security scanning (cargo-audit in CI)
- [ ] Performance benchmarks (criterion)

### Long-term (3-6 months)
- [ ] Nightly builds for nightly Rust features
- [ ] Multi-platform builds (x86_64, aarch64)
- [ ] Automated releases (semantic versioning)
- [ ] Deployment to staging environment

## References

- Production Readiness Roadmap: Phase 3
- GitHub Actions Docs: https://docs.github.com/en/actions
- Cachix Setup: https://docs.cachix.org/
- Nix Flakes: https://nixos.wiki/wiki/Flakes
- Rust CI Best Practices: https://www.lpalmieri.com/posts/rust-ci/

## Related ADRs

- **ADR-001**: NixOS as deployment platform
- **ADR-013**: Testing strategy (unit + integration)
- **ADR-014**: Rate limiting & validation

## Approval

- **Author**: Claude Sonnet 4.5 + Human
- **Reviewers**: Production Readiness Team
- **Approved**: 2026-01-31
