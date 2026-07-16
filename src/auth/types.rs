//! Enterprise multi-tenant authentication types.
//!
//! Defines the core data structures for users, tenants, OAuth2 accounts,
//! and JWT claims used across the auth subsystem.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ── Tenant ────────────────────────────────────────────────────────────────

/// Represents an organization/team that owns resources.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub plan: String, // free | pro | enterprise
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── User ──────────────────────────────────────────────────────────────────

/// A user account, optionally scoped to a tenant.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub provider: String, // google | github | local
    pub is_owner: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

// ── OAuth Account ─────────────────────────────────────────────────────────

/// Links a user to an OAuth2 provider identity.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OAuthAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── User Role ─────────────────────────────────────────────────────────────

/// A user's role within a specific tenant.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserRole {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub role: String, // admin | user | readonly
    pub created_at: DateTime<Utc>,
}

// ── Session ───────────────────────────────────────────────────────────────

/// Tracks an active refresh-token session for revocation.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// ── JWT Claims ────────────────────────────────────────────────────────────

/// Claims embedded in the access token JWT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject — user UUID
    pub sub: String,
    /// Tenant slug (empty string = personal account)
    pub tenant: String,
    /// Role within the tenant (admin | user | readonly)
    pub role: String,
    /// Email for display/debugging
    pub email: String,
    /// Display name
    pub name: String,
    /// Issued at (epoch seconds)
    pub iat: usize,
    /// Expiration (epoch seconds)
    pub exp: usize,
    /// Token ID (UUID) for revocation
    pub jti: String,
}

// ── Auth User (extracted by middleware) ───────────────────────────────────

/// The authenticated user extracted from a valid JWT by the auth middleware.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub tenant_slug: String,
    pub email: String,
    pub display_name: String,
    pub role: String,
}

impl AuthUser {
    /// Returns true if this user has admin privileges in their tenant.
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    /// Returns true if the role meets or exceeds the required level.
    pub fn has_role(&self, required: &str) -> bool {
        match (self.role.as_str(), required) {
            ("admin", _) => true,
            ("user", "user" | "readonly") => true,
            ("readonly", "readonly") => true,
            _ => false,
        }
    }
}

// ── OAuth2 Provider User Info ─────────────────────────────────────────────

/// Normalized user info returned by an OAuth2 provider.
#[derive(Debug, Clone)]
pub struct OAuthUserInfo {
    pub provider: String,
    pub provider_user_id: String,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
}
