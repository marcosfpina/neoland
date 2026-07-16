# Neoland v0.4.0-beta — "Enterprise Ready"

**Data**: 2026-07-16 · **Codename**: Enterprise Ready  
**Tipo**: Beta Release · **Status**: Pre-Release

---

## 🎯 Overview

Neoland v0.4.0-beta is the **enterprise-ready milestone** — the first release that can be deployed in production environments with authentication, encryption, observability, and high availability. Every control plane now requires OAuth2 or API key authentication, all traffic supports mTLS, and the platform ships with a production-grade Helm chart.

### What's Neoland?

An **autonomous AI engineering platform** that runs a 4-stage DSPy agent pipeline (Junior → Senior → Architect → Tech-Leader) to generate Architecture Decision Records (ADRs). Ships with a TUI, Web Console (Leptos WASM), and Desktop app (Tauri). **100% Rust** from the control plane to the browser.

---

## 🚀 Highlights

| Feature | Description |
|---|---|
| **OAuth2 Authentication** | Google & GitHub login → JWT access/refresh tokens → RBAC |
| **Multi-tenant RBAC** | Users, tenants, roles (admin/user/readonly) backed by PostgreSQL |
| **Kubernetes Helm Chart** | HA deployment: 3-10 replicas, HPA, ingress TLS, PostgreSQL + NATS |
| **mTLS Infrastructure** | PKI generator + rustls TLS module for end-to-end encryption |
| **OpenAPI 3.0 Docs** | Swagger UI at `/swagger-ui/`, 16 endpoints, Bearer JWT + API Key auth |
| **Tauri Desktop App** | Native shell (Linux/macOS/Windows) with system tray + notifications |
| **Web Console** | Leptos WASM SPA, 3-panel layout, SSE streaming, 28 tests |
| **CI/CD Pipeline** | 15 GitHub Actions workflows, `act`-validated, Nix flake |

---

## 📦 What's New Since v0.3.0

### Authentication & Authorization (#2)
- **OAuth2 providers**: Google (OpenID Connect) + GitHub OAuth2
- **JWT tokens**: HS256-signed access tokens (15 min) + refresh tokens (7 days)
- **RBAC**: Admin / User / ReadOnly roles scoped per tenant
- **Database schema**: `tenants`, `users`, `oauth_accounts`, `user_roles`, `sessions`
- **Endpoints**: `/auth/login/google`, `/auth/login/github`, `/auth/callback/*`, `/auth/refresh`, `/auth/me`, `/auth/logout`
- **Backward compatible**: Legacy `X-API-Key` header auth preserved

### Kubernetes Helm Chart (#4)
- **HA deployment**: 3 replicas default, HPA up to 10
- **Rolling updates**: `maxUnavailable: 1`, `maxSurge: 1`
- **Health checks**: Liveness (`/health`) + Readiness (`/live`) probes
- **Ingress**: nginx + cert-manager TLS with Let's Encrypt
- **Resources**: 500m CPU / 512Mi request, 2000m CPU / 2Gi limit
- **Anti-affinity**: Pods spread across nodes
- **Init container**: Copies Web Console WASM bundle from separate image
- **External deps**: PostgreSQL (pgvector/pgvector:pg17) + NATS (2.10 + JetStream)

### mTLS End-to-End (#5)
- **Certificate generator**: `bash scripts/gen-certs.sh` — CA + server + client + bridge
- **PKCS#12 bundle**: Import client cert into browsers/TUI
- **TLS module**: `src/tls.rs` — rustls ServerConfig + ClientConfig builders
- **Production pattern**: Reverse proxy (nginx/Caddy) terminates TLS, internal mTLS between services
- **Environment**: `NEOLAND_TLS_CA_CERT`, `NEOLAND_TLS_SERVER_CERT`, `NEOLAND_TLS_SERVER_KEY`

### OpenAPI Documentation (#7)
- **Swagger UI**: Interactive docs at `/swagger-ui/`
- **16 endpoints** across 4 tags: system, chat, agents, auth
- **14 schemas**: Full type definitions for all request/response bodies
- **Dual auth**: `bearer_auth` (JWT) + `api_key` (legacy) security schemes
- **Spec**: `/openapi.json` — importable into Postman, Insomnia, etc.

### Desktop App (#1)
- **Tauri v2 shell**: Native window with Web Console embedded
- **System tray**: Show/Hide/Quit menu
- **Notifications**: Native OS notifications via `tauri-plugin-notification`
- **Nix derivation**: `nix build .#neoland-desktop`
- **Icons**: RGBA icons for all platforms (32x32, 128x128, tray, .ico)

---

## 🧪 Testing

| Suite | Count | Result |
|---|---|---|
| Core unit tests (`cargo test --lib`) | 280 | ✅ All pass |
| Web Console unit tests | 16 | ✅ All pass |
| WASM integration tests (headless Chrome) | 12 | ✅ All pass |
| OpenAPI spec tests | 3 | ✅ All pass |
| JWT token tests | 4 | ✅ All pass |
| Integration tests (gRPC + REST) | 7/8 | ⚠️ 1 requires SecureLLM Bridge |
| CI/CD (`act` simulation) | fmt + clippy + test + wasm | ✅ All pass |
| Nix flake check | 5 derivations | ✅ All pass |

---

## 📋 Migration from v0.3.0

### Database
```sql
-- Run migration 005 for multi-tenant auth:
psql -d neoland -f migrations/005_multi_tenant_auth.sql
```

### Configuration
```toml
# Add to neoland.toml:
[auth]
[auth.oauth]
google_client_id = ""      # Or set NEOLAND_GOOGLE_CLIENT_ID
google_client_secret = ""  # Or set NEOLAND_GOOGLE_CLIENT_SECRET
github_client_id = ""      # Or set NEOLAND_GITHUB_CLIENT_ID
github_client_secret = ""  # Or set NEOLAND_GITHUB_CLIENT_SECRET
base_url = "https://neoland.example.com"

[auth.jwt]
secret = ""                # Or set NEOLAND_JWT_SECRET (openssl rand -hex 32)
```

### Breaking Changes
- **API key auth still works** but is deprecated in favor of JWT
- **`AppState`** has 3 new fields: `db_pool`, `jwt_secret`, `oauth_base_url`
- **`.github/` CI workflows** moved to `.github/workflows/` (GitHub Actions spec)
- **`stdenvNoCC.lib`** removed — use `lib` directly in Nix derivations

---

## 🚢 Deployment

### Quickstart (Nix)
```bash
nix develop
just gen-certs          # Generate mTLS certificates
just server             # Start the control plane
# Open http://localhost:3001/swagger-ui/
```

### Kubernetes
```bash
helm upgrade --install neoland ./deploy/helm/neoland \
  --namespace neoland --create-namespace \
  --set config.auth.jwtSecret="$(openssl rand -hex 32)" \
  --set config.auth.oauth.googleClientId="..." \
  --set config.auth.oauth.googleClientSecret="..."
```

### Docker Compose
```bash
docker compose up -d
```

---

## 🔜 What's Next (v1.0.0)

- [ ] SSO / LDAP enterprise auth
- [ ] vLLM as optional LLM backend
- [ ] Multi-backend routing (auto-select by workload)
- [ ] SOC2 Type I + GDPR compliance
- [ ] Soak period — 2 weeks of real tasks, zero P0 incidents
- [ ] Public launch: landing page, blog post, community

---

## 👥 Team

**VoidNxSEC Team** — `sec@voidnxlabs.com`  
GitHub: [VoidNxSEC/neoland](https://github.com/VoidNxSEC/neoland)

---

**Changelog**: [Full commit history](https://github.com/VoidNxSEC/neoland/compare/v0.3.0...main)  
**License**: MIT
