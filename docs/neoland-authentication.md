# Neoland Authentication Guide

## Overview

Neoland implements multi-protocol authentication for secure API access:
- **REST API**: JWT (OAuth2/SSO) or API key authentication (X-API-Key header), with native TLS/mTLS via `NEOLAND_TLS_*` (v0.0.1)
- **gRPC API**: native TLS with client-certificate verification when a CA is configured (v0.0.1)

See [ADR-011](ADR/ADR-011-authentication-strategy.md) for detailed architecture.

## Quick Start

### 1. Start the Server

```bash
nix develop -c cargo run --bin neoland -- server
```

You should see:
```
🚀 Inicializando Neoland Server...
🔐 Authentication manager initialized
⚠️  Using development API keys (change in production)
📡 gRPC endpoint: [::]:50051
🌐 REST endpoint: 0.0.0.0:3001
```

### 2. Test Authentication

#### ✅ Successful Request (with valid API key)

```bash
curl -X POST http://localhost:3001/v1/chat/completions \
  -H "X-API-Key: neoland_user_dev_key_change_in_production" \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [
      {"role": "user", "content": "Hello, how are you?"}
    ]
  }'
```

#### ❌ Failed Request (no API key)

```bash
curl -X POST http://localhost:3001/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [
      {"role": "user", "content": "Hello"}
    ]
  }'
```

**Expected Response**: `401 Unauthorized`

#### ❌ Failed Request (invalid API key)

```bash
curl -X POST http://localhost:3001/v1/chat/completions \
  -H "X-API-Key: invalid_key_12345" \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [
      {"role": "user", "content": "Hello"}
    ]
  }'
```

**Expected Response**: `401 Unauthorized`

### 3. Check Public Endpoints

The `/health` endpoint is public (no authentication required):

```bash
curl http://localhost:3001/health
```

**Expected Response**: `OK`

## Default API Keys (Development)

⚠️ **WARNING**: These keys are for development only. Change them in production!

| Role | API Key | Permissions |
|------|---------|-------------|
| **Admin** | `neoland_admin_dev_key_change_in_production` | Full access |
| **User** | `neoland_user_dev_key_change_in_production` | Chat, add documents |
| **ReadOnly** | `neoland_readonly_dev_key_change_in_production` | Chat, search only |

### Role Permissions

| Permission | Admin | User | ReadOnly |
|------------|-------|------|----------|
| Chat (inference) | ✅ | ✅ | ✅ |
| Add documents | ✅ | ✅ | ❌ |
| Search | ✅ | ✅ | ✅ |
| System config | ✅ | ❌ | ❌ |
| User management | ✅ | ❌ | ❌ |

## Production Deployment

### ⚠️ CRITICAL: Change Default Keys

**Before deploying to production**, you MUST:

1. **Generate strong API keys** (minimum 32 characters, random)
```bash
# Generate a secure API key
openssl rand -base64 32
```

2. **Store keys in Vault** (Phase 1.2 implementation)
```bash
# Store in Vault (coming in Phase 1.2)
vault kv put secret/neoland/api-keys \
  admin_key=<generated-key> \
  user_key=<generated-key> \
  readonly_key=<generated-key>
```

3. **Distribute keys securely**
- Use environment variables or secrets management
- Never commit keys to version control
- Rotate keys regularly (quarterly recommended)

### Security Best Practices

1. **API Key Management**
   - Use strong, random keys (32+ characters)
   - Rotate keys regularly
   - Revoke compromised keys immediately
   - Different keys per environment (dev/staging/prod)

2. **Key Storage**
   - Never hardcode in source code
   - Use secrets management (Vault, AWS Secrets Manager)
   - Encrypt at rest
   - Audit access logs

3. **Network Security**
   - Use HTTPS/TLS in production — native via `NEOLAND_TLS_*` env vars, or terminate at load balancer
   - Consider IP whitelisting
   - Implement rate limiting (Phase 1.4)

4. **Monitoring**
   - Log all authentication failures (Phase 1.3)
   - Alert on >5 failed attempts in 1 minute
   - Monitor for suspicious patterns

## Integration Examples

### Python

```python
import requests

API_KEY = "neoland_user_dev_key_change_in_production"
BASE_URL = "http://localhost:3001"

headers = {
    "X-API-Key": API_KEY,
    "Content-Type": "application/json"
}

data = {
    "messages": [
        {"role": "user", "content": "Hello, how are you?"}
    ]
}

response = requests.post(
    f"{BASE_URL}/v1/chat/completions",
    headers=headers,
    json=data,
    stream=True
)

if response.status_code == 200:
    for chunk in response.iter_content(chunk_size=None):
        print(chunk.decode('utf-8'), end='', flush=True)
elif response.status_code == 401:
    print("Authentication failed!")
else:
    print(f"Error: {response.status_code}")
```

### JavaScript/TypeScript

```typescript
const API_KEY = "neoland_user_dev_key_change_in_production";
const BASE_URL = "http://localhost:3001";

async function chat(message: string) {
  const response = await fetch(`${BASE_URL}/v1/chat/completions`, {
    method: "POST",
    headers: {
      "X-API-Key": API_KEY,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      messages: [
        { role: "user", content: message }
      ]
    })
  });

  if (response.status === 401) {
    throw new Error("Authentication failed");
  }

  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }

  // Handle streaming response
  const reader = response.body?.getReader();
  if (!reader) throw new Error("No response body");

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    const chunk = new TextDecoder().decode(value);
    process.stdout.write(chunk);
  }
}

chat("Hello, how are you?");
```

### Rust

```rust
use reqwest::header::{HeaderMap, HeaderValue};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(
        "X-API-Key",
        HeaderValue::from_static("neoland_user_dev_key_change_in_production")
    );

    let request = serde_json::json!({
        "messages": [
            {"role": "user", "content": "Hello, how are you?"}
        ]
    });

    let response = client
        .post("http://localhost:3001/v1/chat/completions")
        .headers(headers)
        .json(&request)
        .send()
        .await?;

    if response.status() == 401 {
        eprintln!("Authentication failed!");
        return Ok(());
    }

    println!("Response: {}", response.text().await?);

    Ok(())
}
```

## Troubleshooting

### Issue: 401 Unauthorized

**Causes**:
1. Missing `X-API-Key` header
2. Invalid API key
3. Key typo (check for extra spaces)

**Solution**:
```bash
# Check that the header is correctly formatted
curl -v http://localhost:3001/v1/chat/completions \
  -H "X-API-Key: neoland_user_dev_key_change_in_production" \
  -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"test"}]}'

# Look for the header in the request:
> X-API-Key: neoland_user_dev_key_change_in_production
```

### Issue: Server starts but authentication doesn't work

**Causes**:
1. Server running old version without authentication
2. Wrong port (connecting to different instance)

**Solution**:
```bash
# Ensure you're running the latest version
cargo clean
nix develop -c cargo run --bin neoland -- server

# Check logs for:
# 🔐 Authentication manager initialized
```

### Issue: Need to test without authentication

**Solution**: Use the public `/health` endpoint:
```bash
curl http://localhost:3001/health  # No auth required
```

For testing protected endpoints without auth, you'll need to temporarily modify the code (not recommended for production).

## Roadmap

### Phase 1.1 (Current) ✅
- ✅ REST API key authentication
- ✅ RBAC (admin, user, read-only)
- ✅ Protected/public routes
- ✅ In-memory key storage

### Phase 1.2 (Next) 🔜
- 🔜 Vault integration for secret storage
- 🔜 Load API keys from Vault
- 🔜 Key rotation support

### Phase 1.3 🔜
- 🔜 Audit logging for auth events
- 🔜 Failed auth alerting

### Phase 1.4 🔜
- 🔜 Rate limiting per user
- 🔜 Prevent brute force attacks

### Phase 1.5 ✅ (delivered in v0.0.1)
- ✅ gRPC mTLS authentication — tonic `ServerTlsConfig` with client CA root
- ✅ Certificate-based auth — client certs required when `NEOLAND_TLS_CA_CERT` is set

## References

- [ADR-011: Authentication Strategy](ADR/ADR-011-authentication-strategy.md)
- [Production Readiness Roadmap](../README.md)
- [API Documentation](API.md) (coming in Phase 6)

## Support

**Issues**: Report at [github.com/org/neoland/issues](https://github.com/org/neoland/issues)

**Security**: For security issues, email security@neoland.example.com

## Pattern: mTLS Certificate Generation Across OpenSSL Versions

**Rule: every certificate must be signed with an explicit extensions file — never rely on OpenSSL defaults.**

### The failure mode

`openssl x509 -req` without `-extfile` behaves differently across versions:

| OpenSSL | Result without `-extfile` |
|---------|---------------------------|
| ≤ 3.0.x (Ubuntu 24.04 = 3.0.13) | **X.509 v1** certificate — no extensions at all |
| ≥ 3.3 (nixpkgs) | v3 with SKID/AKID added by default |

rustls/webpki rejects v1 certificates for client authentication. The handshake completes up to the client `Certificate` + `CertificateVerify`, then the server answers `TLS alert: certificate unknown` and the client sees a connection reset (`curl: (56)`). The same code, script and curl pass wherever a newer OpenSSL generated the certs — an environment-dependent bug invisible in local testing.

### The fix (see `scripts/gen-certs.sh`)

Client certificates are signed with an explicit extfile:

```ini
basicConstraints = CA:FALSE
keyUsage = critical, digitalSignature, keyEncipherment
extendedKeyUsage = clientAuth
subjectKeyIdentifier = hash
authorityKeyIdentifier = keyid,issuer
```

Server certs use `extendedKeyUsage = serverAuth` plus SANs; the same rule applies.

### How it was diagnosed (reusable debug pattern)

1. **Smoke script self-diagnosis** (`scripts/tls-smoke.sh`): on failure, before killing the
   server, print — server liveness (`kill -0`) + listeners (`ss -tlnp`), `curl -v` output,
   the **full** server log, and `curl --version`. "Server alive + handshake alert" vs
   "server dead + connection refused" points to entirely different root causes.
2. **Reproduce the CI environment locally** with docker instead of iterating on CI:
   `ubuntu:24.04 --network=host` provides the runner's exact curl (8.5.0/OpenSSL 3.0.13)
   and openssl (3.0.13) to both generate certs and attack a locally-running server.
   Iteration time drops from ~7 min per CI round to seconds.

Verify a client cert is CI-safe:

```bash
openssl x509 -in secrets/tls/client/client.crt -noout -text | grep -E "Version|Extended Key"
# Must show: Version: 3 (0x2) and TLS Web Client Authentication
```
