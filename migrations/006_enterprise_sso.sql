-- Migration 006: Enterprise SSO — LDAP directory config
-- v0.4.0 #3 — Stores LDAP config per tenant for enterprise deployments.
--
-- This table is optional; LDAP can also be configured entirely via
-- environment variables. The table allows per-tenant overrides.

CREATE TABLE IF NOT EXISTS ldap_configs (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID REFERENCES tenants(id) ON DELETE CASCADE,
    -- Connection settings
    url         TEXT NOT NULL DEFAULT 'ldap://localhost:389',
    base_dn     TEXT NOT NULL DEFAULT 'dc=example,dc=com',
    bind_dn     TEXT,
    -- Encrypted bind password (use Vault or SOPS in production)
    bind_password_encrypted TEXT,
    -- Attribute mappings
    email_attr         TEXT NOT NULL DEFAULT 'mail',
    display_name_attr  TEXT NOT NULL DEFAULT 'displayName',
    -- User search filter template
    user_filter        TEXT NOT NULL DEFAULT '(uid={username})',
    -- TLS settings
    starttls   BOOLEAN NOT NULL DEFAULT false,
    enabled    BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- One LDAP config per tenant
    UNIQUE(tenant_id)
);

-- OIDC SSO provider configurations per tenant
CREATE TABLE IF NOT EXISTS oidc_configs (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID REFERENCES tenants(id) ON DELETE CASCADE,
    issuer_url  TEXT NOT NULL,
    client_id   TEXT NOT NULL,
    -- Encrypted client secret (use Vault or SOPS in production)
    client_secret_encrypted TEXT NOT NULL,
    redirect_url TEXT NOT NULL,
    display_name TEXT NOT NULL DEFAULT 'Enterprise SSO',
    enabled     BOOLEAN NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(tenant_id)
);

-- Index for quick lookup by tenant
CREATE INDEX IF NOT EXISTS idx_ldap_configs_tenant ON ldap_configs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_oidc_configs_tenant ON oidc_configs(tenant_id);

-- Auto-update triggers
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_ldap_configs_updated_at') THEN
        CREATE TRIGGER update_ldap_configs_updated_at
            BEFORE UPDATE ON ldap_configs
            FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_oidc_configs_updated_at') THEN
        CREATE TRIGGER update_oidc_configs_updated_at
            BEFORE UPDATE ON oidc_configs
            FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
    END IF;
END $$;
