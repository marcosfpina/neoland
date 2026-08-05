# Neoland: Rust Control Plane and UI with DSPy Agent Orchestration

> **Publication status:** draft. Do not publish as a production announcement.
> Neoland remains a release candidate; see [`ROADMAP.md`](../ROADMAP.md) for
> current evidence and open gates.

**We built an AI platform that assists with Architecture Decision Records using
a Rust control plane and operator surfaces plus a Python DSPy pipeline.**

---

Six months ago, we asked: *"What if an AI could autonomously reason about system architecture, propose trade-offs, and generate auditable decision records — without a human in the loop?"*

Today, **Neoland v0.0.1** is a pre-release baseline. It combines a multi-tenant
Rust control plane, TUI, WebAssembly SPA and desktop shell with a 4-stage Python
DSPy pipeline for Architecture Decision Records (ADRs).

The supported browser path uses Leptos WASM without a JavaScript application
framework; agent orchestration remains a separate Python service.

---

## The Architecture

```
┌──────────────────────────────────────────────────────┐
│  TUI (ratatui) · Web Console (Leptos WASM) · Desktop (Tauri) │
└──────────────────────────┬───────────────────────────┘
                           │ REST :3001 / gRPC :50051
              ┌────────────▼──────────────┐
              │   Neoland Control Plane   │
              │   (Rust · axum · tonic)   │
              └────────────┬──────────────┘
                           │ mTLS
              ┌────────────▼──────────────┐
              │   SecureLLM Bridge        │
              │   (PII redaction · audit) │
              └────────────┬──────────────┘
                           │
     ┌─────────────────────┼─────────────────────┐
     ▼                     ▼                     ▼
llama.cpp (local)    vLLM (GPU cluster)    External APIs
                                              (DeepSeek, Gemini, Groq)
```

### Why Rust Across The Operator Surfaces?

- **Zero-cost abstractions**: The control plane handles gRPC streaming, JWT validation, mTLS handshakes, and LLM inference routing at sub-millisecond latency — with no GC pauses.
- **WASM in the browser**: Leptos compiles to WebAssembly. No JavaScript framework, no virtual DOM diffing, no `node_modules`. The entire Web Console is a 600-line CSS file + compiled Rust.
- **Focused control-plane binary**: `cargo build --release` produces the Rust control plane that serves REST, gRPC, SSE and optional static Web assets; the DSPy pipeline remains external.
- **Deterministic builds**: The Nix flake locks every dependency — including the Leptos WASM toolchain and IBM Plex Mono font — to exact hashes. `nix build .#neoland-full` reproduces the entire stack bit-for-bit.

---

## The Agent Pipeline

Neoland's core is a **4-stage DSPy pipeline** that mirrors an engineering team:

| Stage | Role | Responsibility |
|-------|------|---------------|
| 1 | **Junior** | Draft initial ADR from context |
| 2 | **Senior** | Review, refine, add technical depth |
| 3 | **Architect** | Validate against system constraints, flag risks |
| 4 | **Tech-Leader** | Final approval, sign-off, ledger recording |

Each stage produces checkpoint artifacts stored in `$NEOLAND_CHECKPOINT_DIR`. The final ADR is anchored to a **Merkle chain** (secp256k1 signatures) and published to NATS JetStream — making every decision auditable and immutable.

```
Prompt → [Junior] → [Senior] → [Architect] → [Tech-Leader] → Signed ADR
           ↓            ↓            ↓              ↓
        Checkpoint   Checkpoint   Checkpoint    Checkpoint + Merkle anchor
```

---

## What Ships in v0.0.1

### 1. Multi-Backend LLM Routing

Neoland doesn't lock you into one inference provider. The `UnifiedLLMClient` manages **three backends** with independent circuit breakers and latency tracking:

- **Local** (llama.cpp via ml-ops-api) — low latency, privacy-first
- **vLLM** (OpenAI-compatible GPU server) — high throughput, code generation
- **Bridge** (SecureLLM → DeepSeek, Gemini, Groq) — best general knowledge

Four routing strategies:

| Strategy | Behavior |
|----------|----------|
| `LocalFirst` | Local → vLLM → Bridge fallback |
| `ExternalFirst` | Bridge → vLLM → Local fallback |
| `LoadBalanced` | EMA-latency weighted selection |
| **`Adaptive`** | **Workload classification** — Short prompts → local, Code gen → vLLM, General → bridge |

The `Adaptive` strategy uses a heuristic classifier that inspects prompt length, keywords (code, architecture, explain), and structure to route to the optimal backend **automatically**. No YAML config, no manual routing rules.

### 2. Enterprise Auth (OAuth2 + SSO + LDAP)

- **OAuth2**: Google + GitHub login with JWT (HS256, 15-min access + 7-day refresh)
- **SSO/OIDC**: Generic OpenID Connect with auto-discovery — plug in Azure AD, Okta, Keycloak, Auth0, or any compliant IdP
- **LDAP**: Bind + search for Active Directory, OpenLDAP, FreeIPA — configurable attribute mapping per tenant
- **RBAC**: Admin / User / ReadOnly roles scoped to PostgreSQL-backed tenants

All auth flows converge to the same JWT issuance path. A single `require_auth` middleware layer validates Bearer tokens and injects `AuthUser` into request extensions — the handlers don't care *how* you authenticated.

### 3. Kubernetes-Native Deployment

The Helm chart (`deploy/helm/neoland`) deploys a production HA setup:

- 3 replicas default, HPA to 10
- Anti-affinity: pods spread across nodes
- Init container copies WASM bundle from separate image
- Health probes on `/health` (liveness) and `/live` (readiness)
- nginx ingress + cert-manager TLS
- PostgreSQL (pgvector) + NATS JetStream as external dependencies

```bash
helm install neoland ./deploy/helm/neoland \
  --set auth.oidc.issuer=https://login.microsoftonline.com/tenant/v2.0 \
  --set ingress.host=neoland.example.com
```

### 4. End-to-End mTLS

Every hop — TUI → Control Plane → SecureLLM Bridge → LLM Backend — supports mutual TLS. The `scripts/gen-certs.sh` generates a complete PKI:

```
ca.crt                     # 4096-bit RSA CA
server.crt + server.key    # 2048-bit server cert
client.crt + client.key    # 2048-bit client cert (also PKCS#12 for browsers)
bridge.crt + bridge.key    # Bridge-specific mTLS cert
```

The TLS module (`src/tls.rs`) provides `from_env()` constructors that read certificate paths from environment variables — no hardcoded paths, no config files needed. Both listeners serve TLS natively: REST via axum-server (rustls) and gRPC via tonic. When a CA is configured, client certificates are required — plain-HTTP and cert-less connections are rejected at the handshake.

---

## The Web Console (Leptos WASM)

The Web Console is a **Leptos SPA** that compiles to WebAssembly and runs entirely in the browser. No JavaScript framework. No `npm install`.

```
┌──────────┬──────────────────────────────┬───────────┐
│ Sessions │      Conversation            │ Reasoning │
│ Panel    │      Panel                   │ Panel     │
│          │                              │           │
│ · ADR #1 │  User: "Design the auth..."  │ Pipeline  │
│ · ADR #2 │  Agent: "## ADR-014..."      │ tree:     │
│ · ADR #3 │                              │ Junior ─┐ │
│          │  [streaming SSE tokens...]    │ Senior ─┤ │
│          │                              │ Arch  ──┤ │
│          │                              │ TL ─────┘ │
└──────────┴──────────────────────────────┴───────────┘
│                   Composer                        │
│  [________________________________________] [Send] │
└───────────────────────────────────────────────────┘
```

- **Real-time SSE streaming**: Tokens appear in the browser as the agent generates them
- **Pipeline visualization**: Confidence badges (RWA-anchored) at each stage
- **Zero JavaScript**: CSS artesanal, 600 lines, responsive (720px, 980px breakpoints)
- **PWA-ready**: Service worker + manifest + icons

The WASM bundle is built deterministically via the Nix flake and served by the same Rust binary that runs the control plane — a single `cargo build` produces everything.

---

## Performance (historical, not release evidence)

The measurements below were recorded for an earlier draft but do not yet carry
the revision, harness and raw output required by the roadmap claim policy. They
must be reproduced before publication.

Measured on a Hetzner CX32 (8 vCPU, 16 GB RAM) with llama.cpp running locally:

| Metric | Value |
|--------|-------|
| REST endpoint p50 latency | 1.2ms |
| gRPC streaming first-token | 45ms |
| JWT validation | 0.08ms |
| ADR pipeline (4 stages) | 8-45s (varies by model) |
| Memory (idle, control plane) | 42 MB |
| Binary size (release, stripped) | 18 MB |

---

## What's Next: v0.1.0

The v0.1.0 public release requires:

- **Soak period**: 2 weeks of real tasks without P0 incidents
- **SOC2 Type I + GDPR compliance**: Formal docs and audit trails
- **Blog post + Launch**: HN, Reddit, dev.to
- **Community**: GitHub Discussions + Discord

---

## Try It

```bash
git clone https://github.com/VoidNxSEC/neoland
cd neoland

# NixOS / Nix
nix develop
just smoke

# Ubuntu / Debian
docker compose up -d postgres nats
cargo run -- server
```

Open `http://localhost:3001` for the Web Console, or run `neoland chat` for the TUI.

---

**Neoland is open-source (MIT).** Contributions, issues, and ADR proposals welcome.

→ [GitHub](https://github.com/VoidNxSEC/neoland) · [ROADMAP](ROADMAP.md) · [ADRs](docs/ADR/)
