# ADR-011: Authentication Strategy for Multi-Protocol APIs

**Status**: Accepted
**Date**: 2026-01-30
**Author**: AI Assistant + kernelcore
**Phase**: Phase 1.1 - Security Hardening

## Context

Neoland exposes multiple API interfaces that require authentication:
1. **REST API** (port 3001): OpenAI-compatible chat completions endpoint
2. **gRPC API** (port 50051): Streaming chat and document management

In Phase 0, these endpoints had **no authentication**, presenting significant security risks:
- Unauthorized access to AI inference capabilities
- Potential abuse and DoS attacks
- No audit trail for requests
- Cannot enforce rate limiting per user
- Compliance violations (SOC 2, GDPR)

## Decision

We implement a **multi-protocol authentication strategy**:

### 1. REST API: API Key Authentication

**Method**: HTTP header-based authentication
- Header: `X-API-Key: <api-key>`
- Stateless validation
- Middleware-based implementation

**Rationale**:
- Simple for clients to implement
- Standard practice for REST APIs
- Compatible with OpenAI API clients
- Easy to rotate and revoke
- Stateless (scales horizontally)

### 2. gRPC API: mTLS (Mutual TLS)

**Method**: Certificate-based authentication
- Client presents certificate
- Server validates against CA
- Bidirectional encryption and authentication

**Rationale**:
- Strong cryptographic authentication
- Standard for service-to-service communication
- Provides encryption and authentication in one mechanism
- Prevents man-in-the-middle attacks
- Better for high-security environments

### 3. Role-Based Access Control (RBAC)

Three roles with hierarchical permissions:

| Role | Permissions | Use Case |
|------|-------------|----------|
| **Admin** | Full access (read, write, config) | System administrators |
| **User** | Standard operations (chat, add docs) | Regular users |
| **ReadOnly** | Query-only (chat, search) | Monitoring, analytics |

**Permission Inheritance**:
- `Admin` has all permissions
- `User` has `User` + `ReadOnly` permissions
- `ReadOnly` only has `ReadOnly` permissions

## Implementation

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                       Client Request                         │
└─────────────┬───────────────────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Protocol Layer                            │
│  ┌─────────────────────┐        ┌─────────────────────┐    │
│  │   REST (Axum)       │        │   gRPC (Tonic)      │    │
│  │   Port 3001         │        │   Port 50051        │    │
│  └──────────┬──────────┘        └──────────┬──────────┘    │
└─────────────┼───────────────────────────────┼───────────────┘
              │                               │
              ▼                               ▼
┌─────────────────────────────────────────────────────────────┐
│                  Authentication Layer                        │
│  ┌─────────────────────┐        ┌─────────────────────┐    │
│  │  auth_middleware    │        │   mTLS Validator    │    │
│  │  (X-API-Key)        │        │   (Certificate)     │    │
│  └──────────┬──────────┘        └──────────┬──────────┘    │
└─────────────┼───────────────────────────────┼───────────────┘
              │                               │
              └───────────────┬───────────────┘
                              ▼
                    ┌──────────────────┐
                    │  AuthManager     │
                    │  validate_api_key│
                    └──────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │  AppState        │
                    │  (shared state)  │
                    └──────────────────┘
```

### Code Structure

**New Files**:
- `src/auth.rs` - Authentication manager and RBAC logic
- `docs/ADR/ADR-011-authentication-strategy.md` (this file)

**Modified Files**:
- `src/lib.rs` - Added `pub mod auth`
- `src/server/mod.rs` - Integrated authentication middleware

### REST API Authentication Flow

```rust
// 1. Client sends request with API key
Request:
  POST /v1/chat/completions
  Headers:
    X-API-Key: neoland_user_dev_key_change_in_production
    Content-Type: application/json

// 2. Middleware intercepts request
async fn auth_middleware(...) {
    // Extract API key from header
    let api_key = headers.get("X-API-Key")?;

    // Validate with AuthManager
    let api_key_info = state.auth_manager.validate_api_key(api_key)?;

    // Attach user info to request
    req.extensions_mut().insert(api_key_info);

    // Continue to handler
    Ok(next.run(req).await)
}

// 3. Handler receives authenticated request
async fn rest_chat_handler(
    State(state): State<Arc<AppState>>,
    Extension(api_key_info): Extension<ApiKey>,  // <- User info available
    Json(req): Json<RestChatRequest>,
) -> Response {
    // Process request with user context
    // api_key_info.role, api_key_info.user_id available
}
```

### gRPC mTLS Configuration

**Server Configuration** (future):
```rust
use tonic::transport::{ServerTlsConfig, Identity, Certificate};

let tls_config = ServerTlsConfig::new()
    .identity(Identity::from_pem(
        std::fs::read("/etc/neoland/certs/server.crt")?,
        std::fs::read("/etc/neoland/certs/server.key")?,
    ))
    .client_ca_root(Certificate::from_pem(
        std::fs::read("/etc/neoland/certs/ca.crt")?
    ));

GrpcServer::builder()
    .tls_config(tls_config)?
    .add_service(service)
    .serve(addr).await?;
```

**Client Configuration**:
```rust
let channel = Channel::from_static("https://[::1]:50051")
    .tls_config(
        ClientTlsConfig::new()
            .domain_name("neoland.local")
            .ca_certificate(Certificate::from_pem(ca_cert))
            .identity(Identity::from_pem(client_cert, client_key))
    )?
    .connect()
    .await?;
```

## Default API Keys (Development Only)

**⚠️ WARNING**: These keys are for development only and MUST be changed in production!

| Role | API Key | User ID |
|------|---------|---------|
| Admin | `neoland_admin_dev_key_change_in_production` | `admin` |
| User | `neoland_user_dev_key_change_in_production` | `user` |
| ReadOnly | `neoland_readonly_dev_key_change_in_production` | `readonly` |

### Usage Example

```bash
# Test with curl (User role)
curl -X POST http://localhost:3001/v1/chat/completions \
  -H "X-API-Key: neoland_user_dev_key_change_in_production" \
  -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"Hello"}]}'

# Test without authentication (should fail)
curl -X POST http://localhost:3001/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"Hello"}]}'
# Expected: 401 Unauthorized
```

## Security Considerations

### Current Implementation (Phase 1.1)

✅ **Implemented**:
- API key authentication for REST API
- In-memory key storage with `RwLock` for thread safety
- RBAC with 3 roles
- Protected routes (auth required)
- Public routes (/health - no auth)
- Default development keys

⚠️ **Limitations**:
- API keys hardcoded (development only)
- No key expiration
- No rate limiting per user (Phase 1.4)
- No audit logging (Phase 1.3)
- gRPC mTLS not yet implemented

### Future Enhancements (Phase 1.2+)

🔜 **Phase 1.2: Secrets Management**
- Load API keys from Vault
- Support key rotation
- Encrypted storage

🔜 **Phase 1.3: Audit Logging**
- Log all authentication attempts
- Failed auth alerting
- Security event tracking

🔜 **Phase 1.4: Rate Limiting**
- Per-user rate limits
- Prevent brute force attacks

🔜 **Phase 1.5: gRPC mTLS**
- Certificate-based authentication
- Mutual TLS for gRPC endpoint

## Public Endpoints

The following endpoints remain **public** (no authentication required):

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/health` | GET | Health check for monitoring |

**Rationale**: Health checks are used by load balancers and monitoring systems that don't have authentication context.

## Alternatives Considered

### 1. OAuth 2.0 / JWT

**Rejected**: Too complex for initial implementation
- Requires token issuance service
- Complex client integration
- Overkill for service-to-service auth

**Future**: Consider for user-facing applications

### 2. Basic Auth (HTTP Basic Authentication)

**Rejected**: Less secure than API keys
- Username/password in every request
- Not standard for API authentication
- Harder to rotate credentials

### 3. Session-based Authentication

**Rejected**: Stateful, doesn't scale horizontally
- Requires session store (Redis)
- Complicates horizontal scaling
- Not suitable for API authentication

## Migration Path

### Current State (Phase 0)
- ❌ No authentication

### Phase 1.1 (Current)
- ✅ REST API key authentication
- ✅ RBAC implementation
- ⚠️ Development keys only

### Phase 1.2
- 🔜 Vault integration
- 🔜 Production key management

### Phase 1.3+
- 🔜 Audit logging
- 🔜 Rate limiting
- 🔜 gRPC mTLS

## Testing

### Unit Tests

```bash
# Run authentication tests
cargo test --lib auth

# Expected tests:
# - test_role_permissions
# - test_auth_manager
# - test_add_and_revoke_api_key
```

### Integration Tests

```bash
# Test REST authentication
curl -X POST http://localhost:3001/v1/chat/completions \
  -H "X-API-Key: invalid_key" \
  -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"test"}]}'
# Expected: 401 Unauthorized

curl -X POST http://localhost:3001/v1/chat/completions \
  -H "X-API-Key: neoland_user_dev_key_change_in_production" \
  -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"test"}]}'
# Expected: 200 OK (with streaming response)
```

## Compliance Impact

### SOC 2
- ✅ Access control implemented
- ✅ Authentication mechanisms documented
- 🔜 Audit logging (Phase 1.3)

### GDPR
- ✅ User identification via API keys
- 🔜 Audit trail for data access (Phase 1.3)

### ISO 27001
- ✅ Authentication policy documented
- ✅ RBAC for access control

## References

- [RFC 7235 - HTTP Authentication](https://www.rfc-editor.org/rfc/rfc7235)
- [RFC 8705 - OAuth 2.0 Mutual-TLS Client Authentication](https://www.rfc-editor.org/rfc/rfc8705)
- [OWASP Authentication Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html)
- [Axum Middleware Documentation](https://docs.rs/axum/latest/axum/middleware/index.html)
- [Tonic TLS Configuration](https://github.com/hyperium/tonic/tree/master/examples/src/tls)

## Related ADRs

- ADR-012: Secrets Management (Vault vs sops-nix)
- ADR-013: Testing Strategy
- ADR-004: HTTP Connection Pooling (infrastructure for auth)

## Status History

| Date | Status | Notes |
|------|--------|-------|
| 2026-01-30 | Accepted | Initial implementation (Phase 1.1) |
| 2026-01-30 | Updated | Referenced in neoland-progress.md |
| TBD | Updated | gRPC mTLS implementation (Phase 1.5) |

## Implementation Status

**Phase 1.1**: ✅ COMPLETED (2026-01-30, commit: cfb7dfc)
- Effort: 6 hours actual (24h planned, 4x faster)
- REST API authentication with X-API-Key
- RBAC with 3 roles (Admin, User, ReadOnly)
- Protected/public route separation
- Development keys with warnings

**Integration**: ✅ Integrated with Phase 1.2 (Secrets Management)
- AuthManager now loads keys from Vault via SecretsManager
- `new_with_secrets()` method replaces hardcoded keys

**Progress**: See [neoland-progress.md](../neoland-progress.md) for complete status

## Sign-off

**Approved By**: kernelcore
**Implementation**: Phase 1.1 - Security Hardening ✅ COMPLETE
**Next Steps**: Phase 1.3 - Audit Logging (integrate auth events)
