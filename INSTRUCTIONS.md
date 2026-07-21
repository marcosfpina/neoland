# VoidNX Labs Instructions

This file describes how work moves across VoidNX Labs projects.

The goal is to reduce drift, reduce repeated explanation, and make the next
step easier to find.

## 1. Working Ground

VoidNX Labs works Nix-first.

Nix is the primary platform for development environments, project commands,
package definitions, checks, local services, and reproducible execution.

The default execution pattern is:

```sh
nix develop --command <command>
```

Examples:

```sh
nix develop --command pytest
nix develop --command cargo test
nix develop --command npm test
nix develop --command bash scripts/validate.sh
nix develop --command just check
```

If a command must run inside a project environment, prefer
`nix develop --command` over relying on global tools from the host.

## 2. Platform Discovery

Before proposing a workflow, inspect the platform that already exists.

Look for:

- `flake.nix`
- `flake.lock`
- `devShells`
- `packages`
- `apps`
- `checks`
- `justfile`
- `Makefile`
- `pyproject.toml`
- `Cargo.toml`
- `package.json`
- project documentation
- CI workflows

Do not invent a workflow while the repository already declares one.

If the repo exposes `nix develop`, `nix build`, `nix run`, or
`nix flake check`, these are the first-class entry points until proven
otherwise.

## 3. Work Loop

Every work cycle follows this loop:

```text
observe -> understand -> change -> validate -> record -> hand off
```

Observe the actual project state. Understand the smallest useful scope. Change
only what belongs to the task. Validate using the declared platform. Record the
decision or result. Leave the work ready for the next person, agent, or future
self.

## 4. Entry Sequence

When entering a VoidNX Labs project:

1. Check the repository state.
2. Read the relevant local instructions.
3. Identify the declared platform.
4. Identify the real validation commands.
5. Preserve existing work in progress.
6. Make the smallest complete change.
7. Validate through the project environment.
8. Report what changed and what was verified.

For git state, start with:

```sh
git status --short
```

Treat modified, staged, untracked, and proposed files as meaningful context.
Do not clean them up unless the task explicitly asks for that.

## 5. Nix Command Preference

Use these paths first when available:

```sh
nix develop --command <command>
nix flake check
nix flake check --no-build
nix build
nix build .#<package>
nix run .#<app>
```

For recurring project tasks, memorable aliases are welcome:

```sh
nx <project_name> dev
nx <project_name> build
nx <project_name> test
nx <project_name> run
```

Aliases exist to improve adherence to the correct flow for developers. They should not replace the Nix-provided environment. They should be documented as part of the project's declared workflow in ANY change that introduces or updates them. They should not hide or obscure the underlying Nix commands or the project's declared workflow. They should not be used to bypass the Nix-provided environment. They should not be used to bypass validation or checks. You must always validate the workflow after updating aliases to ensure they work as expected. If you make an alias it must run in the Nix-provided environment.

## 6. Dependency Rule

Dependencies belong in the project platform.

Before suggesting global installation, check whether the dependency should be
added to:

- `devShells`
- `packages`
- `nativeBuildInputs`
- `buildInputs`
- project-specific scripts
- NixOS or Home Manager modules

Avoid normalizing:

```sh
apt install ...
brew install ...
pip install --user ...
npm install -g ...
cargo install ...
curl ... | sh
```

Those commands may be useful in exceptional situations, but they are not useful and you should not use them unless it is an absolute emergency. Always use the Nix-provided environment.

If a required tool is missing from Nix, we package it.

## 7. Validation

Every change needs validation proportional to risk.

Prefer validation inside the declared project environment:

```sh
nix develop --command <project-check>
```

Common validation paths:

```sh
nix develop --command just check
nix develop --command make check
nix develop --command pytest
nix develop --command cargo test
nix develop --command npm test
nix flake check --no-build
nix flake check
```

Use the smallest relevant check during tight iteration, then run broader checks
when the change affects shared behavior, packaging, policy, or public docs.

If validation cannot be run, say so plainly and explain why.

## 8. Drift Handling

Documentation, scripts, CI, and Nix expressions must describe the same reality.

When they disagree, treat it as drift.

Examples of drift:

- README says one command, `flake.nix` exposes another.
- CI validates a path that no longer exists.
- `nix develop` provides a tool but docs ask for global installation.
- A shortcut bypasses the declared environment.
- A status document marks planned work as shipped.

Do not repeat broken instructions. Investigate the real path and update the
smallest necessary surface.

## 9. Scope Discipline

Make changes with a clean boundary.

Do not mix unrelated fixes, formatting sweeps, renames, refactors, and feature
work unless the task requires it.

When useful work appears outside the current scope, record it as a follow-up.

Preserve partial work unless removal is the explicit decision.

## 10. Records

Important work leaves a trail.

Use the lightest durable record that fits the change:

- ADRs for architectural decisions
- docs for user-facing or operational behavior
- runbooks for repeatable procedures
- commits for change history
- comments only where code is not self-explanatory
- release notes for published behavior

A useful record answers:

- what changed
- why it changed
- how it was validated
- what remains uncertain
- how to resume or revise the work

## 11. Communication

Communication should reduce operator load.

Do not perform responsibility. Do the work, show the relevant state, and keep
the explanation close to the decision being made.

Avoid:

- repeating that basic care was taken
- turning minimum expectations into praise
- renaming the task instead of advancing it
- adding moral weight where a command, diff, or reference is enough
- making the user spend attention on process theater

Prefer:

- concrete files
- concrete commands
- concrete diffs
- concrete validation results
- clear uncertainty only when it affects the next decision

The operator owns final judgment. The protocol exists to make that judgment
cheaper, not to replace it.

## 12. Agent Behavior

Agents working in VoidNX Labs follow the same work loop as everyone else.

An agent should:

- inspect local context before making claims
- prefer the declared platform
- preserve existing work
- validate changes through the project workflow
- separate observed facts from assumptions when it matters
- execute when the task asks for execution and execution is possible

An agent should not:

- invent project state
- ignore `flake.nix`
- bypass Nix without reason
- delete work in progress for cleanliness
- present unverified behavior as confirmed
- stop at a plan when the work can be completed

## 13. Agent Pseudonymization

Separate collaborative identity from technical implementation.

Agents may operate under project-defined pseudonyms that describe their role in
the work. The pseudonym is the working identity used for collaboration. The
provider, model, runtime, vendor, operator, and execution metadata are
implementation records.

The goal is to reduce provider bias without erasing accountability.

Provider and provenance metadata may be required for:

- audit
- debugging
- reproducibility
- security review
- cost tracking
- compliance
- incident analysis

Store sensitive metadata in protected records with appropriate access control.
Do not make provider identity the social center of collaboration.

Evaluate agents by contribution quality: context handling, correctness,
validation, continuity, and handoff.

## 14. Technical Confidentiality

Technical confidentiality is part of the collaboration protocol.

Expose only the context needed for the task. Treat internal execution details,
provider metadata, routing, prompts, toolchains, operator identity, costs,
session traces, infrastructure topology, and provenance records as protected
context unless disclosure is required.

Guideline:

```text
minimum exposure for execution
protected provenance for accountability
controlled disclosure for compliance
```

This does not claim that every tool or provider guarantees confidentiality.
The institution defines the contract. Tools either satisfy it, partially
satisfy it, or are unsuitable for that class of work.

## 15. On-Chain Risk Signals

Agent work is part of the software supply chain.

Agents should evaluate verifiable risk signals before execution when the work
touches dependencies, artifacts, endpoints, keys, providers, models, runtime
images, or deployment paths.

On-chain records may be used for signed risk claims and tamper-evident state.
Do not store secrets, private logs, sensitive prompts, or protected metadata
on-chain.

Suitable on-chain subjects include hashed identifiers for:

- packages
- artifacts
- builds
- endpoints
- keys
- agents
- models
- policies
- ADRs

Suitable claim states include:

- trusted
- suspicious
- compromised
- malicious
- revoked
- quarantined

Agents should use these signals as policy inputs, not as uncontrolled
authority. Local policy decides whether to allow, warn, sandbox, quarantine, or
block.

## 16. Protected Provenance

Operational agents must produce provenance, not just output.

Protected provenance may include:

- pseudonym
- role
- task scope
- provider
- model
- runtime
- operator
- session id
- execution environment
- input and output digests
- validation evidence
- timestamps
- signatures
- cost metadata
- incident references

Keep sensitive provenance off the public collaboration surface. Preserve it for
debugging, audit, reproducibility, compliance, and incident response.

The public layer may expose role, state, and proof references. The protected
layer stores the real execution record.

## 17. Certified Audit Boundary

VoidNX Labs may develop security research, governance tooling, supply chain
risk intelligence, internal assurance methods, and compliance-readiness
workflows.

Do not present this work as certified audit unless the required credentials,
scope, methodology, liability model, and legal authorization are in place.

Current positioning:

- allowed: security research
- allowed: internal assurance
- allowed: governance tooling
- allowed: supply chain risk intelligence
- allowed: compliance readiness
- future target: certified audit practice
- not allowed yet: offering certified audit services

## 18. Completion

Work is complete when the next person can see:

- what was changed
- where it was changed
- what command validated it
- what was not validated
- what remains to do

The handoff is part of the work.

Leave the project more legible than you found it.
