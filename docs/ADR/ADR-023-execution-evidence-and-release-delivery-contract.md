# ADR-023: Execution Evidence and Release Delivery Contract

**Status**: Accepted
**Date**: 2026-08-23
**Decision Makers**: VoidNxSEC Team
**Scope**: Runtime execution, CI evidence, release claims, and published artifacts
**Related ADRs**: ADR-015, ADR-017, ADR-019

---

## Context

Neoland has strong component-level engineering evidence, but the repository has used the same
words for materially different levels of proof. A unit test, an HTTP contract test, a server
booted with disabled dependencies, and a complete operator workflow have all been described as
"E2E", "smoke", or "release ready" at different times.

This ambiguity creates several failure modes:

- a CI job can be green while its main command is `continue-on-error`;
- a test backed by a DSPy stub can be reported as pipeline E2E;
- a binary booted with `NEOLAND_SKIP_EMBEDDINGS` and no database can be reported as a complete
  standalone runtime;
- evidence from an older revision can remain in README or roadmap claims after the code changes;
- an artifact can be published without the exact published bytes having passed the complete
  runtime path;
- implemented schemas or event producers can be presented as an operational external service;
- a client can be described as functional without completing an authenticated workflow.

The result is not primarily a testing problem. It is a contract problem: the project lacks a
normative definition of what execution was proved, which claims that evidence permits, and what
must be true before an artifact is delivered.

## Decision

Neoland adopts a single execution-evidence and release-delivery contract. Every CI job,
verification record, roadmap claim, and release must identify the evidence level it satisfies.
Evidence may support claims at its own level or below, never above it.

This ADR is effective immediately. Existing tests remain valuable, but their names and claims
must be interpreted according to this taxonomy until the CI and documentation are aligned.

### 1. Evidence levels

| Level | Name | Required boundary | Test doubles | Claims permitted |
|---|---|---|---|---|
| E0 | Static | Source, schemas, manifests | Not applicable | Source is formatted, linted, and structurally valid |
| E1 | Unit | One process or module | Allowed when explicit and deterministic | Internal behavior of the tested unit |
| E2 | Contract | One protocol or serialization boundary | Allowed and must be named | Compatibility of the exercised contract |
| E3 | Integration | Real implementations on every boundary named by the test | Not allowed on those boundaries | The named components interoperate |
| E4 | End-to-end | Supported client through control plane, Python pipeline, PostgreSQL, real LLM path, and persisted result | Not allowed | The tested user workflow functions on the recorded topology |
| E5 | Release acceptance | Exact publishable artifact in a clean environment, including E4 and operational gates | Not allowed | The exact revision and artifact are eligible for release |

Rules:

1. A test that uses `DspyStub`, a fake LLM, a fake database, or a disabled dependency is E1 or E2.
   It must not be named E2E.
2. A real PostgreSQL service does not promote a test to E3 if another boundary named by the test
   is stubbed. The evidence description must name the real and substituted boundaries.
3. A skipped mandatory step does not satisfy a level. Conditional skips are acceptable only for
   jobs that are explicitly informational for that execution.
4. `continue-on-error` results are informational and cannot satisfy a mandatory gate.
5. Historical results cannot be reused after the source revision, lockfile, build recipe,
   migration set, or release configuration changes.

### 2. Runtime profiles

Execution evidence must also identify the runtime profile:

| Profile | Required services | Intended use |
|---|---|---|
| `degraded` | Neoland process only; optional dependencies may be absent | Diagnostics, liveness, static UI serving, and failure-mode validation |
| `control-plane` | Neoland, production-equivalent auth configuration, PostgreSQL, migrations | REST/gRPC/SSE, RBAC, persistence, and operational integration |
| `pipeline` | `control-plane` plus the actual FastAPI/DSPy service and supported LLM gateway/model | Multi-agent execution and checkpoint validation |
| `full` | `pipeline` plus every service claimed by the supported deployment topology and at least one supported client | Release acceptance and product-level claims |

`NEOLAND_SKIP_EMBEDDINGS`, development API keys, missing `DATABASE_URL`, a DSPy stub, or a fake
LLM must be recorded in evidence. Their presence prevents `pipeline` or `full` classification.

The `degraded` profile is a supported diagnostic mode, not a release topology. Its health and
doctor output must state which capabilities are unavailable. A degraded boot can prove that the
process starts and fails safely; it cannot prove that Neoland performs its primary workflow.

### 3. Canonical end-to-end workflow

E4 evidence must exercise, at minimum, this sequence without substituting any named boundary:

1. Start from a documented, clean installation path.
2. Apply the supported database migrations.
3. Start the publishable Neoland control-plane artifact.
4. Start the actual Python FastAPI/DSPy pipeline.
5. Start the supported SecureLLM/inference path with a real model.
6. Authenticate through the client contract being claimed.
7. Submit a task through a supported client surface.
8. Observe pipeline stages and a terminal event through the supported event path.
9. Exercise steering or breakpoint resolution when that capability is claimed.
10. Verify the persisted session, task count, requester identity, decision, and checkpoint.
11. Verify any ledger or signature claim cryptographically, not only the presence of a schema or
    event.
12. Stop the topology cleanly and retain the evidence bundle.

The TUI and Web Console are separate client claims. Successful TUI evidence does not prove the
Web Console, and vice versa. A client is operational only after its authenticated happy path and
one authorization-denied path pass.

### 4. CI contract

The canonical release workflow must:

- run against the exact commit proposed for the tag;
- execute every mandatory E0-E5 gate on that revision;
- fail when a mandatory command fails or a required dependency is unavailable;
- label jobs according to the evidence taxonomy;
- state every test double or disabled capability in the job name or summary;
- build publishable artifacts once and promote those same bytes through acceptance and release;
- publish a machine-readable evidence manifest even when the workflow fails;
- prevent the release job from running unless the release-acceptance gate succeeds.

The evidence manifest must record:

- Git revision, tag, dirty-tree state, and timestamp;
- toolchain and lockfile identities;
- commands and exit status;
- runtime profile and topology;
- test doubles, skips, and `continue-on-error` steps;
- versions or immutable references for external services and models;
- migration version;
- artifact names, sizes, and SHA-256 checksums;
- links or identifiers for logs, test reports, and operator sign-off.

Supply-chain scanning is a mandatory release gate. An advisory may be accepted only through a
documented, scoped, time-bounded exception that states reachability, owner, expiry, and removal
condition. A permanently green wrapper around a failing scanner is not a gate.

### 5. Delivery contract

`Cargo.toml` package version is the canonical product version. A release tag must be
`v<Cargo.toml version>`, and package metadata, CLI output, README, release notes, and artifact
names must match it. `LICENSE` is the canonical license source.

For every supported release artifact:

1. The artifact must be produced from a clean tree at the recorded revision.
2. The exact artifact must pass its smoke and E5 workflow outside the source tree.
3. Checksums must be generated after acceptance from the exact files to be published.
4. Database migration, backup, restore, and rollback requirements must be documented.
5. External services and models required by the supported topology must be pinned or recorded by
   immutable identity.
6. Known limitations must describe degraded behavior and unsupported paths explicitly.
7. Release notes must link the evidence manifest and state the highest evidence level achieved.

Changing source after acceptance invalidates the evidence. The new revision must rebuild and
repeat the mandatory gates; a documentation-only exception is not implicit.

### 6. Product maturity claims

The public maturity label follows evidence, not implementation volume or test count:

| Label | Minimum contract |
|---|---|
| Prototype | E0-E2 are repeatable; primary workflow may require operator knowledge |
| Alpha | E4 passes on one documented installation path; no open P0 security or data-integrity defects |
| Beta | E4 passes on every advertised deployment path; authenticated supported clients, backup/restore, and rollback are verified |
| Release Candidate | E5 passes on the exact candidate artifacts; documentation and known limitations match the same revision |
| General Availability | RC requirements plus completed soak, signed distribution requirements, and named operational ownership |

Scores, test counts, performance measurements, and phrases such as "production ready",
"enterprise ready", "cryptographically signed", or "fully operational" require evidence from
the same revision. Performance evidence must additionally record workload, environment, command,
sample size, and date.

Until Alpha requirements pass, Neoland's correct maturity label is **Prototype**, even when its
component-level engineering is more mature.

### 7. Pull-request delivery rules

A delivery PR must have one reviewable objective and one evidence story. Mechanical movement,
security behavior, runtime features, dependency changes, and release-policy changes should be
separate unless their atomicity is necessary and explained.

Every delivery PR must state:

- intended behavior and explicit non-goals;
- affected trust and data boundaries;
- evidence levels executed;
- test doubles and unavailable gates;
- migration and compatibility impact;
- risk and rollback path;
- documentation and ADR impact.

Green component checks do not make a PR mergeable when the claimed behavior is unverified or the
required documentation and architectural decisions are absent.

## Alternatives Considered

### Keep independent checklists in README, ROADMAP, scripts, and workflows

Rejected. The current drift demonstrates that duplicated prose is not a reliable contract.

### Use one boolean `release-ready` gate

Rejected. It hides which topology and boundaries were actually exercised and encourages
degraded smoke tests to inherit full-product claims.

### Require external services for every test

Rejected. Unit and contract feedback must remain fast and deterministic. The decision restricts
the claims supported by test doubles instead of banning them from all lower evidence levels.

### Treat any green CI run as release evidence

Rejected. CI can contain optional jobs, silent skips, test doubles, stale revisions, or
`continue-on-error` commands. Only the canonical E5 workflow and evidence manifest can authorize
release claims.

## Consequences

### Positive

- Product claims become traceable to reproducible evidence.
- Fast unit and contract tests remain available without being confused with E2E coverage.
- Degraded operation becomes explicit and testable instead of silently promoted.
- Release artifacts, source revision, documentation, and checksums share one identity.
- Security and supply-chain failures cannot be hidden behind green job status.
- Operators and reviewers can determine exactly what a passing workflow proves.

### Negative

- Current CI job names, release scripts, roadmap claims, and skills require alignment.
- Full E4/E5 execution is slower and requires real infrastructure and model capacity.
- Some existing green checks will be reclassified as E1/E2 rather than E2E.
- Release promotion will remain blocked until authenticated client and full-topology evidence
  exist.

### Risks

- The contract can itself drift if enforcement remains prose-only.
- External model availability can make E4 evidence nondeterministic.
- A single full-stack gate can become expensive or flaky if topology ownership is unclear.

Mitigations are to generate the evidence manifest in CI, pin the test model and service
revisions, retain lower-level deterministic gates, and keep one named owner for the release
workflow.

## Adoption Plan

1. Rename current stub-backed jobs from E2E to contract or component integration.
2. Make supply-chain gate failures blocking or encode approved exceptions with expiry.
3. Preserve the current degraded binary smoke, but label it `degraded` and remove release-ready
   claims from it.
4. Add a real E4 workflow for Rust → FastAPI/DSPy → PostgreSQL → supported LLM → checkpoint.
5. Add authenticated E4 workflows for each client surface advertised as operational.
6. Generate and publish the evidence manifest from the canonical release workflow.
7. Align README, ROADMAP, STATE, release notes, inventory, and agent skills with the contract.
8. Re-evaluate maturity only after the corresponding acceptance criteria pass on one revision.

## Verification

This ADR is implemented as a governance contract when all of the following are true:

- CI job names and summaries identify evidence level and runtime profile;
- no mandatory release gate uses untracked skips or `continue-on-error`;
- the canonical workflow emits a complete evidence manifest;
- E4 covers the actual Python pipeline, PostgreSQL, supported LLM path, persistence, and events;
- each advertised client completes an authenticated E4 workflow;
- release version, tag, documentation, license, artifact, and checksum identities agree;
- ROADMAP and release notes link evidence from the exact candidate revision.

Until then, this ADR is accepted but its automated enforcement is incomplete.
