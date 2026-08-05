# Doc-Comment Conventions

Status: proposed pattern, derived from the Doxygen POC (`Doxyfile.poc`,
`rust_filter.py`) run against `src/health.rs` and cross-checked against the
best existing examples already in the codebase (`src/validation.rs`,
`src/audit.rs`, `src/secrets.rs`, `src/auth/jwt.rs`,
`src/agents/checkpoint_store.rs`).

This is not a new invention — it formalizes what those five files already do
well and were doing inconsistently everywhere else. As of this writing, only
**24 of 62** files under `src/` open with a module-level `//!` block; the
other 38 (including `src/health.rs`, the POC subject) skip straight to a
plain `//` note or no header at all. That gap is the concrete thing this
document exists to close over time — not all at once.

## Three levels, three jobs

Doc comments answer three different questions, at three different scopes.
Don't let one level's job leak into another's.

### 1. `//!` — module header: *what is this, and why does it exist in the project*

One block at the very top of the file, before `use`. Answers "what is this
module's role in Neoland" — the project-identity question — not "how is it
implemented." Reference the owning ADR when one exists; that's where the
*why* behind the design lives, so the module header doesn't have to restate
it.

```rust
//! Input Validation Module
//!
//! This module provides input validation and sanitization for all user inputs.
//! It protects against oversized requests, malformed data, and potential
//! security issues.
//!
//! See ADR-014 for validation strategy decisions.
```

(`src/validation.rs`, verbatim — this is the reference example.)

### 2. `///` — item docs: *what this does*

Above every `pub` struct, enum, fn, and field. Brief one-line summary first
(Doxygen's `\brief`, rustdoc's summary line — both tools use "first sentence
of the first paragraph" the same way, which is exactly why this level
survived the Doxygen POC untouched). Expand below the brief line only when
the behavior isn't obvious from the signature — see the POC's
`check_vector_store_health` for the shape to copy: brief line, blank line,
then a short bulleted decision table.

```rust
/// Health checker for vector store.
///
/// Checks `NEOLAND_DATABASE_URL` (or `DATABASE_URL`):
/// - Not set  → Healthy  (in-memory mode)
/// - Set, reachable + pgvector present → Healthy
/// - Set, reachable but no pgvector   → Degraded
/// - Set, unreachable or timeout       → Degraded
pub async fn check_vector_store_health() -> ComponentHealth { ... }
```

This is also the level that showed up cleanest in the generated Doxygen
HTML — struct fields with `///` above them rendered with correct name, type,
and description in the POC output with zero extra work.

### 3. `//` — inline: *why, never what*

Matches the project's existing default (no comments unless the *why* is
genuinely non-obvious: a workaround, a hidden constraint, a subtle
invariant). If an inline `//` is explaining what the next line does, that's
a signal the code should be renamed or extracted instead, not commented.

## What NOT to do

- Don't put implementation detail in the `//!` header — that's what `///`
  on individual items is for. A module header bloated with internals is why
  "separar responsabilidades" became a real complaint in the first place.
- Don't leave a file with no `//!` header at all if it exports a `pub`
  surface — every file with public API needs the project-identity paragraph,
  even a two-line one.
- Don't restate the function signature in prose (`/// Takes a string and
  returns a bool`) — that's what the signature already says. Say what the
  signature *can't* say: preconditions, side effects, error cases.

## Relationship to tooling

This convention is generator-agnostic on purpose. It held up unchanged
across the Doxygen POC (via `rust_filter.py`'s translation) and is exactly
what `rustdoc`/`cargo doc` already consume natively with zero filtering.
Whichever tool ends up doing the full-scale generation later, the source
comments don't need to change — only the tool wrapping them does.
