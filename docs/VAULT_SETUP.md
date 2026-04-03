# HashiCorp Vault Setup Guide

## Overview

This guide explains how to set up HashiCorp Vault for Neoland secrets management.

**Vault provides**:
- Secure storage for API keys and credentials
- Encryption at rest (AES-256-GCM)
- Audit logging for compliance
- Fine-grained access control
- Automatic secret rotation (coming in Phase 1.3)

See [ADR-012](ADR/ADR-012-secrets-management.md) for architecture details.

---

## Quick Start (Development)

### 1. Install Vault

```bash
# Option A: NixOS (recommended)
nix-shell -p vault

# Option B: Download binary
wget https://releases.hashicorp.com/vault/1.15.0/vault_1.15.0_linux_amd64.zip
unzip vault_1.15.0_linux_amd64.zip
sudo mv vault /usr/local/bin/

# Verify installation
vault version
```

### 2. Start Vault (Dev Mode)

```bash
# Start Vault server in dev mode (NOT for production!)
vault server -dev -dev-root-token-id="dev-token-12345"

# Output:
# ==> Vault server configuration:
#
#              Api Address: http://127.0.0.1:8200
#                      Cgo: disabled
#          Cluster Address: https://127.0.0.1:8201
#               Go Version: go1.21.3
#                Listener 1: tcp (addr: "127.0.0.1:8200", cluster address: "127.0.0.1:8201", max_request_duration: "1m30s", max_request_size: "33554432", tls: "disabled")
#                 Log Level:
#                     Mlock: supported: true, enabled: false
#             Recovery Mode: false
#                   Storage: inmem
#                   Version: Vault v1.15.0
#               Version Sha: 0a6da99f0be1b8e9cd7ce4f40ceffa66b1b47dd6
#
# ==> Vault server started! Log data will stream in below:
#
# WARNING! dev mode is enabled! Do not run this in production!
```

**Keep this terminal open** - Vault is running in foreground.

### 3. Configure Environment

Open a **new terminal**:

```bash
# Set Vault address
export VAULT_ADDR='http://127.0.0.1:8200'

# Set root token (dev mode only!)
export VAULT_TOKEN='dev-token-12345'

# Verify connection
vault status
```

### 4. Enable KV Secrets Engine

```bash
# Enable KV v2 secrets engine at path "secret"
vault secrets enable -path=secret kv-v2

# Verify
vault secrets list
```

### 5. Store Neoland Secrets

```bash
# Store LLM API keys
vault kv put secret/neoland/llm/deepseek \
  api_key="your-deepseek-api-key-here"

vault kv put secret/neoland/llm/openai \
  api_key="your-openai-api-key-here"

# Store Neoland authentication API keys
# Generate secure random keys
ADMIN_KEY="neoland_admin_prod_$(openssl rand -hex 32)"
USER_KEY="neoland_user_prod_$(openssl rand -hex 32)"
READONLY_KEY="neoland_readonly_prod_$(openssl rand -hex 32)"

vault kv put secret/neoland/api-keys/admin key="$ADMIN_KEY"
vault kv put secret/neoland/api-keys/user key="$USER_KEY"
vault kv put secret/neoland/api-keys/readonly key="$READONLY_KEY"

# Save these keys for testing!
echo "Admin Key: $ADMIN_KEY" >> ~/.neoland-dev-keys.txt
echo "User Key: $USER_KEY" >> ~/.neoland-dev-keys.txt
echo "ReadOnly Key: $READONLY_KEY" >> ~/.neoland-dev-keys.txt
cat ~/.neoland-dev-keys.txt
```

### 6. Verify Secrets

```bash
# Read secrets back
vault kv get secret/neoland/llm/deepseek

# Output:
# ======== Secret Path ========
# secret/data/neoland/llm/deepseek
#
# ======= Metadata =======
# Key                Value
# ---                -----
# created_time       2026-01-30T12:34:56.789Z
# custom_metadata    <nil>
# deletion_time      n/a
# destroyed          false
# version            1
#
# ==== Data ====
# Key        Value
# ---        -----
# api_key    your-deepseek-api-key-here

# List all secrets
vault kv list secret/neoland/llm
vault kv list secret/neoland/api-keys
```

### 7. Start Neoland with Vault

```bash
# In your neoland directory
cd /home/kernelcore/master/neoland

# Start server with Vault configuration
export VAULT_ADDR='http://127.0.0.1:8200'
export VAULT_TOKEN='dev-token-12345'

nix develop -c cargo run --bin neoland -- server

# You should see:
# 🚀 Inicializando Neoland Server...
# 🔐 Authentication manager initialized
# ✅ Connected to Vault successfully
# 📡 gRPC endpoint: [::]:50051
# 🌐 REST endpoint: 0.0.0.0:3001
```

### 8. Test Authentication

```bash
# Get your admin key from the file
ADMIN_KEY=$(grep "Admin Key:" ~/.neoland-dev-keys.txt | cut -d' ' -f3)

# Test API with Vault-loaded key
curl -X POST http://localhost:3001/v1/chat/completions \
  -H "X-API-Key: $ADMIN_KEY" \
  -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"Hello from Vault!"}]}'
```

---

## Fallback to Environment Variables

If Vault is not available, Neoland automatically falls back to environment variables:

```bash
# Stop Vault (Ctrl+C in Vault terminal)

# Set environment variables
export DEEPSEEK_API_KEY="sk-your-key-here"
export NEOLAND_ADMIN_API_KEY="neoland_admin_dev_key_change_in_production"
export NEOLAND_USER_API_KEY="neoland_user_dev_key_change_in_production"
export NEOLAND_READONLY_API_KEY="neoland_readonly_dev_key_change_in_production"

# Start server (will use env vars)
nix develop -c cargo run --bin neoland -- server

# You should see:
# ⚠️  Vault not configured (VAULT_ADDR/VAULT_TOKEN missing), using environment variables
```

---

## Production Setup

### 1. Install Vault on NixOS

**Configuration** (`/etc/nixos/configuration.nix`):
```nix
{
  services.vault = {
    enable = true;
    address = "0.0.0.0:8200";
    storageBackend = "file";
    storagePath = "/var/lib/vault";
    tlsCertFile = "/etc/vault/certs/server.crt";
    tlsKeyFile = "/etc/vault/certs/server.key";
  };

  # Open firewall
  networking.firewall.allowedTCPPorts = [ 8200 ];
}
```

Apply:
```bash
sudo nixos-rebuild switch
```

### 2. Initialize Vault

```bash
# First time only - generates unseal keys and root token
vault operator init -key-shares=5 -key-threshold=3

# Output (SAVE THESE SECURELY!):
# Unseal Key 1: <key1>
# Unseal Key 2: <key2>
# Unseal Key 3: <key3>
# Unseal Key 4: <key4>
# Unseal Key 5: <key5>
#
# Initial Root Token: <root-token>
```

**⚠️ CRITICAL**: Store unseal keys and root token securely!
- Use a password manager (1Password, LastPass, Bitwarden)
- Print and store in a safe
- Split among trusted team members

### 3. Unseal Vault

Vault starts "sealed" and must be unsealed after every restart:

```bash
# Unseal with 3 of 5 keys (threshold=3)
vault operator unseal <key1>
vault operator unseal <key2>
vault operator unseal <key3>

# Verify
vault status
# Should show: Sealed: false
```

### 4. Create Policy

**Create policy file** (`neoland-policy.hcl`):
```hcl
# Read-only access to neoland secrets
path "secret/data/neoland/*" {
  capabilities = ["read", "list"]
}

path "secret/metadata/neoland/*" {
  capabilities = ["list"]
}
```

**Apply policy**:
```bash
vault policy write neoland neoland-policy.hcl
```

### 5. Create AppRole

```bash
# Enable AppRole authentication
vault auth enable approle

# Create role with neoland policy
vault write auth/approle/role/neoland \
  token_policies="neoland" \
  token_ttl=1h \
  token_max_ttl=24h \
  secret_id_ttl=0 \
  token_num_uses=0

# Get role ID
ROLE_ID=$(vault read -field=role_id auth/approle/role/neoland/role-id)
echo "Role ID: $ROLE_ID"

# Generate secret ID
SECRET_ID=$(vault write -field=secret_id -f auth/approle/role/neoland/secret-id)
echo "Secret ID: $SECRET_ID"
```

### 6. Configure Neoland

**systemd service** (`/etc/systemd/system/neoland.service`):
```ini
[Unit]
Description=Neoland AI Agent Server
After=network.target vault.service

[Service]
Type=simple
User=neoland
Group=neoland
WorkingDirectory=/opt/neoland
ExecStart=/opt/neoland/bin/neoland server

# Vault configuration
Environment="VAULT_ADDR=https://vault.internal:8200"
Environment="VAULT_ROLE_ID=<role-id>"
Environment="VAULT_SECRET_ID=<secret-id>"

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/neoland

[Install]
WantedBy=multi-user.target
```

**Start service**:
```bash
sudo systemctl daemon-reload
sudo systemctl enable neoland
sudo systemctl start neoland

# Check logs
sudo journalctl -u neoland -f
```

---

## Vault Operations

### View Secrets

```bash
# List all secrets under neoland/
vault kv list secret/neoland/llm
vault kv list secret/neoland/api-keys

# Read specific secret
vault kv get secret/neoland/llm/deepseek

# Get only the value
vault kv get -field=api_key secret/neoland/llm/deepseek
```

### Update Secrets

```bash
# Update secret (creates new version)
vault kv put secret/neoland/llm/deepseek \
  api_key="new-key-here"

# Patch secret (merge with existing)
vault kv patch secret/neoland/llm/deepseek \
  api_key="updated-key"
```

### Delete Secrets

```bash
# Soft delete (can be undeleted)
vault kv delete secret/neoland/llm/deepseek

# Undelete
vault kv undelete -versions=1 secret/neoland/llm/deepseek

# Permanent delete
vault kv destroy -versions=1 secret/neoland/llm/deepseek
```

### Rotate Keys

```bash
# 1. Generate new key
NEW_KEY=$(openssl rand -hex 32)

# 2. Update Vault
vault kv put secret/neoland/api-keys/admin \
  key="neoland_admin_prod_$NEW_KEY"

# 3. Clear Neoland cache (via API or restart)
# Cache TTL is 30s, so new key will be used automatically
```

---

## Troubleshooting

### Issue: Vault connection refused

**Symptoms**:
```
⚠️  Failed to connect to Vault, falling back to environment variables
```

**Solutions**:
1. Check Vault is running: `vault status`
2. Verify `VAULT_ADDR`: `echo $VAULT_ADDR`
3. Check firewall: `sudo netstat -tlnp | grep 8200`

### Issue: Vault is sealed

**Symptoms**:
```
Error: Vault is sealed
```

**Solution**:
```bash
# Unseal with 3 keys
vault operator unseal <key1>
vault operator unseal <key2>
vault operator unseal <key3>
```

### Issue: Permission denied

**Symptoms**:
```
Error: permission denied
```

**Solutions**:
1. Check token: `vault token lookup`
2. Verify policy: `vault policy read neoland`
3. Check path: Ensure secrets are under `secret/neoland/`

### Issue: Secret not found

**Symptoms**:
```
Error: Secret not found in Vault or environment
```

**Solutions**:
1. List secrets: `vault kv list secret/neoland/llm`
2. Check path: Ensure using correct category (llm, api-keys, etc.)
3. Verify KV v2: `vault secrets list | grep secret`

---

## Security Best Practices

### ✅ DO

- Use TLS in production (`https://`)
- Rotate root token after initial setup
- Use AppRole for applications, not root token
- Enable audit logging: `vault audit enable file file_path=/var/log/vault_audit.log`
- Backup Vault data regularly
- Use separate Vault instances for dev/staging/prod
- Monitor Vault logs

### ❌ DON'T

- Use dev mode in production
- Commit Vault tokens to git
- Share unseal keys via Slack/email
- Run Vault without TLS
- Use root token for applications
- Store unseal keys on same server as Vault

---

## Monitoring

### Health Check

```bash
# Check Vault health
vault status

# Check seal status
vault operator seal-status

# Check auth methods
vault auth list
```

### Metrics

Vault exposes Prometheus metrics at `/v1/sys/metrics`:

```bash
curl -H "X-Vault-Token: $VAULT_TOKEN" \
  http://127.0.0.1:8200/v1/sys/metrics?format=prometheus
```

### Audit Logs

```bash
# Enable audit logging
vault audit enable file file_path=/var/log/vault_audit.log

# View logs
tail -f /var/log/vault_audit.log | jq
```

---

## Next Steps

- [ ] Set up production Vault cluster (HA)
- [ ] Configure automatic unsealing (Vault Auto-Unseal)
- [ ] Implement secret rotation (Phase 1.3)
- [ ] Add Vault monitoring to Prometheus
- [ ] Create disaster recovery runbook

---

## References

- [Official Vault Documentation](https://www.vaultproject.io/docs)
- [Vault Getting Started](https://developer.hashicorp.com/vault/tutorials/getting-started)
- [vaultrs Rust Client](https://docs.rs/vaultrs/latest/vaultrs/)
- [ADR-012: Secrets Management](ADR/ADR-012-secrets-management.md)
- [Production Hardening](https://www.vaultproject.io/docs/internals/security)

---

## Support

**Issues**: Report at [github.com/org/neoland/issues](https://github.com/org/neoland/issues)

**Security**: For security issues, email security@neoland.example.com
