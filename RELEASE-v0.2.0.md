# Neoland v0.2.0 — Honest Preview

**Web Console funcional + Nix WASM build + testes completos.**

> **Historical preview draft:** v0.2.0 is not the current release status. This
> file records a development snapshot; canonical status and gates live in
> [`ROADMAP.md`](ROADMAP.md).

Partial validation recorded on 2026-07-16:

| Gate | Result |
|---|---|
| Rust unit tests (275) | ✅ |
| Web Console tests (16) | ✅ |
| Clippy (all targets, 0 warnings) | ✅ |
| Clippy (WASM target, 0 warnings) | ✅ |
| E2E REST tests (22/22) | ✅ |
| Python contracts (26/26) | ✅ |
| WASM compilation | ✅ |
| Format check | ✅ |
| Doctor (`ok: true`) | ✅ |
| Full-stack smoke (9/9 layers) | Historical claim; current real-LLM smoke remains pending |

## What's new in v0.2.0

### Web Console (Leptos WASM)

- **Backend integration**: REST + SSE connected to the Neoland control plane.
  Sessions load from PostgreSQL, messages stream in real time via EventSource.
- **SSE streaming**: structured event parsing with `SseEvent` type. Stage output
  prefixed with `[junior]`, `[senior]`, etc. Auto-closes on `pipeline_done`.
- **Stream guard**: generation-based concurrency control — old SSE connections
  are silently discarded when a new task starts. No race conditions.
- **16 unit tests**: model, API types, SSE event deserialization. Full coverage of
  all data types.
- **Nix flake**: `nix build .#neoland-web` builds the WASM bundle via `trunk`.
  Dev shell includes `trunk` + WASM target pre-configured.
- **CSS artesanal**: ~600 lines, zero dependencies, responsive (720px / 980px).

### Developer Experience

- `just test-web` — run Web Console unit tests
- `just screenshot` — capture TUI + Web Console screenshots
- `just ci` — updated to include Web crate tests
- `just dev-web` — single-command server + Web Console on :8080

### CI/CD

- All pipelines include `cargo test -p neoland-web`
- Release pipeline gates on full test suite before building artifacts
- Nix CI verifies `nix build .#neoland-web`
- Web Console workflow includes unit test job

### Documentation

- README rewritten: Leptos WASM, TUI, CLI as first-class interfaces.
  No mention of Next.js.
- Screenshots embedded (TUI ASCII dump + Web Console wireframe).
- Release notes reflect real stack (TUI, not Next.js).

## What's next (v0.3.0 — Full Stack Beta)

- gRPC-web bridge (tonic-web nativo)
- IBM Plex Mono font via Nix derivation
- Deploy estático: Neoland server serves `web/dist/`
- Smoke E2E with real LLM
- PWA service worker + manifesto
- Quickstart <5 min (NixOS + Ubuntu)
- Leptos integration tests (`wasm-bindgen-test`)
- Metrics dashboard (latency, tokens, confidence)

## Upgrade notes

No breaking changes from v0.1.0-rc.1. The Web Console is a new crate
(`neoland-web`) in the Cargo workspace. It compiles to WASM and requires
`trunk` for development builds (`nix develop` provides it).

To serve the Web Console in production, either:
- `just build-web` → copy `web/dist/` to your static file server
- `nix build .#neoland-web` → copy the Nix store path

The Neoland server can serve `web/dist/` directly when configured with
the static file path (see `server/mod.rs` fallback service).

---

Full changelog: [`ROADMAP.md`](ROADMAP.md)
