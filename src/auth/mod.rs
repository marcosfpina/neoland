//! Authentication and Authorization Module
//!
//! Multi-protocol authentication system for Neoland:
//! - REST API: API key via X-API-Key header (legacy) + JWT Bearer token
//! - OAuth2: Google & GitHub OAuth2 login (v0.0.1)
//! - SSO/LDAP: Enterprise identity providers (v0.0.1 #3)
//! - gRPC: mTLS (mutual TLS) authentication
//! - RBAC: Role-Based Access Control (admin, user, read-only)
//! - Multi-tenant: User → Tenant → Role scoping
//!
//! See ADR-011 for detailed architecture decisions.

// ── Enterprise auth (v0.0.1) ──────────────────────────────────────────────
pub mod jwt;
pub mod middleware;
pub mod oauth;
pub mod routes;
pub mod sso;
pub mod types;

// Re-export commonly-used types
pub use middleware::JwtSecret;
pub use types::{AuthUser, Claims, Tenant, User, UserRole};

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use anyhow::{anyhow, Result};

use crate::secrets::{SecretType, SecretsManager};

/// User role for RBAC
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    /// Full system access (can modify configuration, view all data)
    Admin,
    /// Standard user access (can make requests, add documents)
    User,
    /// Read-only access (can only query, no modifications)
    ReadOnly,
}

impl Role {
    /// Check if this role has the required permission
    pub fn has_permission(&self, required: &Role) -> bool {
        match (self, required) {
            // Admin has all permissions
            (Role::Admin, _) => true,
            // User has User and ReadOnly permissions
            (Role::User, Role::User) | (Role::User, Role::ReadOnly) => true,
            // ReadOnly only has ReadOnly permission
            (Role::ReadOnly, Role::ReadOnly) => true,
            // Everything else is denied
            _ => false,
        }
    }
}

/// API key with associated metadata
#[derive(Debug, Clone)]
pub struct ApiKey {
    pub key: String,
    pub role: Role,
    pub user_id: String,
    pub description: String,
}

/// Authentication manager
///
/// In production, this should be backed by a database or secrets management
/// system. For now, we use an in-memory store with predefined keys.
pub struct AuthManager {
    api_keys: Arc<RwLock<HashMap<String, ApiKey>>>,
}

impl AuthManager {
    /// Create a new AuthManager with default development keys.
    ///
    /// **Development only.** In production use `new_with_secrets()` (Phase 1.2,
    /// completed) which loads keys from Vault/environment. Set
    /// `NEOLAND_REQUIRE_VAULT_KEYS=1` to enforce this at startup and reject
    /// dev keys.
    pub fn new() -> Self {
        let mut keys = HashMap::new();

        // Default admin key (should be loaded from secrets management in production)
        keys.insert(
            "neoland_admin_dev_key_change_in_production".to_string(),
            ApiKey {
                key: "neoland_admin_dev_key_change_in_production".to_string(),
                role: Role::Admin,
                user_id: "admin".to_string(),
                description: "Development admin key".to_string(),
            },
        );

        // Default user key
        keys.insert(
            "neoland_user_dev_key_change_in_production".to_string(),
            ApiKey {
                key: "neoland_user_dev_key_change_in_production".to_string(),
                role: Role::User,
                user_id: "user".to_string(),
                description: "Development user key".to_string(),
            },
        );

        // Default read-only key
        keys.insert(
            "neoland_readonly_dev_key_change_in_production".to_string(),
            ApiKey {
                key: "neoland_readonly_dev_key_change_in_production".to_string(),
                role: Role::ReadOnly,
                user_id: "readonly".to_string(),
                description: "Development read-only key".to_string(),
            },
        );

        Self { api_keys: Arc::new(RwLock::new(keys)) }
    }

    /// Create a new AuthManager with keys from SecretsManager (Phase 1.2)
    ///
    /// Attempts to load API keys from Vault. Falls back to default keys if
    /// Vault unavailable.
    pub async fn new_with_secrets(secrets_manager: &SecretsManager) -> Self {
        let mut keys = HashMap::new();

        // Try to load keys from Vault/secrets manager
        let roles = vec![
            ("admin", Role::Admin, "Administrator API key"),
            ("user", Role::User, "Standard user API key"),
            ("readonly", Role::ReadOnly, "Read-only API key"),
        ];

        for (role_name, role, description) in roles {
            match secrets_manager
                .get_secret(SecretType::NeolandApiKey(role_name.to_string()))
                .await
            {
                Ok(key) => {
                    tracing::info!(
                        role = role_name,
                        source = if secrets_manager.is_vault_available() {
                            "vault"
                        } else {
                            "env"
                        },
                        "Loaded API key for role"
                    );
                    keys.insert(
                        key.clone(),
                        ApiKey {
                            key: key.clone(),
                            role,
                            user_id: role_name.to_string(),
                            description: description.to_string(),
                        },
                    );
                },
                Err(e) => {
                    tracing::warn!(
                        role = role_name,
                        error = %e,
                        "Failed to load API key from secrets, using default development key"
                    );
                    // Fallback to default dev key
                    let default_key = format!("neoland_{}_dev_key_change_in_production", role_name);
                    keys.insert(
                        default_key.clone(),
                        ApiKey {
                            key: default_key,
                            role,
                            user_id: role_name.to_string(),
                            description: format!("Development {} key (fallback)", role_name),
                        },
                    );
                },
            }
        }

        let manager = Self { api_keys: Arc::new(RwLock::new(keys)) };

        // Production safety check: refuse to start with dev keys when explicitly
        // required. Set NEOLAND_REQUIRE_VAULT_KEYS=1 in production to enforce
        // this.
        if std::env::var("NEOLAND_REQUIRE_VAULT_KEYS").as_deref() == Ok("1") {
            let has_dev_keys = manager
                .api_keys
                .read()
                .map(|k| k.keys().any(|key| key.contains("_dev_key_")))
                .unwrap_or(false);

            if has_dev_keys {
                tracing::error!(
                    "⛔ NEOLAND_REQUIRE_VAULT_KEYS=1 is set but development API keys are active. \
                     Set NEOLAND_ADMIN_API_KEY / NEOLAND_USER_API_KEY / NEOLAND_READONLY_API_KEY \
                     or configure Vault before starting in production."
                );
                panic!("Production key requirement violated — refusing to start with dev keys");
            }
        }

        manager
    }

    /// Validate an API key and return the associated metadata
    pub fn validate_api_key(&self, key: &str) -> Result<ApiKey> {
        let keys = self.api_keys.read().map_err(|e| anyhow!("Failed to acquire lock: {}", e))?;

        keys.get(key).cloned().ok_or_else(|| anyhow!("Invalid API key"))
    }

    /// Add a new API key (admin operation)
    pub fn add_api_key(&self, api_key: ApiKey) -> Result<()> {
        let mut keys =
            self.api_keys.write().map_err(|e| anyhow!("Failed to acquire lock: {}", e))?;

        keys.insert(api_key.key.clone(), api_key);
        Ok(())
    }

    /// Revoke an API key (admin operation)
    pub fn revoke_api_key(&self, key: &str) -> Result<()> {
        let mut keys =
            self.api_keys.write().map_err(|e| anyhow!("Failed to acquire lock: {}", e))?;

        keys.remove(key).ok_or_else(|| anyhow!("API key not found"))?;

        Ok(())
    }

    /// List all API keys (admin operation, excludes actual key values)
    pub fn list_api_keys(&self) -> Result<Vec<(String, Role, String)>> {
        let keys = self.api_keys.read().map_err(|e| anyhow!("Failed to acquire lock: {}", e))?;

        Ok(keys
            .values()
            .map(|k| (k.user_id.clone(), k.role.clone(), k.description.clone()))
            .collect())
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;
    use tokio::sync::Mutex;

    use super::*;

    lazy_static! {
        static ref ENV_LOCK: Mutex<()> = Mutex::new(());
    }

    #[test]
    fn test_role_permissions() {
        // Admin has all permissions
        assert!(Role::Admin.has_permission(&Role::Admin));
        assert!(Role::Admin.has_permission(&Role::User));
        assert!(Role::Admin.has_permission(&Role::ReadOnly));

        // User has User and ReadOnly permissions
        assert!(!Role::User.has_permission(&Role::Admin));
        assert!(Role::User.has_permission(&Role::User));
        assert!(Role::User.has_permission(&Role::ReadOnly));

        // ReadOnly only has ReadOnly permission
        assert!(!Role::ReadOnly.has_permission(&Role::Admin));
        assert!(!Role::ReadOnly.has_permission(&Role::User));
        assert!(Role::ReadOnly.has_permission(&Role::ReadOnly));
    }

    #[test]
    fn test_auth_manager() {
        let manager = AuthManager::new();

        // Validate default admin key
        let result = manager.validate_api_key("neoland_admin_dev_key_change_in_production");
        assert!(result.is_ok());
        let api_key = result.unwrap();
        assert_eq!(api_key.role, Role::Admin);
        assert_eq!(api_key.user_id, "admin");

        // Validate invalid key
        let result = manager.validate_api_key("invalid_key");
        assert!(result.is_err());
    }

    #[test]
    fn test_add_and_revoke_api_key() {
        let manager = AuthManager::new();

        // Add new key
        let new_key = ApiKey {
            key: "test_key_123".to_string(),
            role: Role::User,
            user_id: "testuser".to_string(),
            description: "Test key".to_string(),
        };

        assert!(manager.add_api_key(new_key.clone()).is_ok());

        // Validate new key
        let result = manager.validate_api_key("test_key_123");
        assert!(result.is_ok());

        // Revoke key
        assert!(manager.revoke_api_key("test_key_123").is_ok());

        // Key should no longer be valid
        let result = manager.validate_api_key("test_key_123");
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_api_key_overwrites() {
        let manager = AuthManager::new();

        let key1 = ApiKey {
            key: "duplicate_key".to_string(),
            role: Role::User,
            user_id: "user1".to_string(),
            description: "First key".to_string(),
        };

        let key2 = ApiKey {
            key: "duplicate_key".to_string(), // Same key
            role: Role::Admin,
            user_id: "user2".to_string(),
            description: "Second key".to_string(),
        };

        // Add first key
        assert!(manager.add_api_key(key1).is_ok());

        // Adding duplicate key overwrites the previous one
        assert!(manager.add_api_key(key2).is_ok());

        // Validate that second key is now active
        let result = manager.validate_api_key("duplicate_key").unwrap();
        assert_eq!(result.user_id, "user2"); // Should be the second key
        assert_eq!(result.role, Role::Admin);
    }

    #[test]
    fn test_revoke_nonexistent_key() {
        let manager = AuthManager::new();

        // Revoking non-existent key should fail
        let result = manager.revoke_api_key("nonexistent_key");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_empty_api_key() {
        let manager = AuthManager::new();

        // Empty API key should fail validation
        let result = manager.validate_api_key("");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_api_keys() {
        let manager = AuthManager::new();

        // Should have 3 default keys (admin, user, readonly)
        let keys = manager.list_api_keys().unwrap();
        assert_eq!(keys.len(), 3);

        // Check that all default users are present
        let user_ids: Vec<String> = keys.iter().map(|(user_id, _, _)| user_id.clone()).collect();
        assert!(user_ids.contains(&"admin".to_string()));
        assert!(user_ids.contains(&"user".to_string()));
        assert!(user_ids.contains(&"readonly".to_string()));
    }

    #[test]
    fn test_role_hierarchy() {
        // Test role hierarchy: Admin > User > ReadOnly
        assert!(Role::Admin.has_permission(&Role::Admin));
        assert!(Role::Admin.has_permission(&Role::User));
        assert!(Role::Admin.has_permission(&Role::ReadOnly));

        assert!(!Role::User.has_permission(&Role::Admin));
        assert!(Role::User.has_permission(&Role::User));
        assert!(Role::User.has_permission(&Role::ReadOnly));

        assert!(!Role::ReadOnly.has_permission(&Role::Admin));
        assert!(!Role::ReadOnly.has_permission(&Role::User));
        assert!(Role::ReadOnly.has_permission(&Role::ReadOnly));
    }

    #[tokio::test]
    async fn test_auth_manager_with_secrets() {
        use std::sync::Arc;

        use crate::secrets::SecretsManager;

        let _guard = ENV_LOCK.lock().await;

        // Set up test environment
        std::env::remove_var("NEOLAND_REQUIRE_VAULT_KEYS");
        std::env::set_var("NEOLAND_ADMIN_API_KEY", "vault_admin_key");
        std::env::set_var("NEOLAND_USER_API_KEY", "vault_user_key");
        std::env::set_var("NEOLAND_READONLY_API_KEY", "vault_readonly_key");

        let secrets_manager = Arc::new(SecretsManager::new().await.unwrap());
        let manager = AuthManager::new_with_secrets(&secrets_manager).await;

        // Should load keys from environment (via SecretsManager)
        let result = manager.validate_api_key("vault_admin_key");
        assert!(result.is_ok());
        let api_key = result.unwrap();
        assert_eq!(api_key.role, Role::Admin);

        // Cleanup
        std::env::remove_var("NEOLAND_ADMIN_API_KEY");
        std::env::remove_var("NEOLAND_USER_API_KEY");
        std::env::remove_var("NEOLAND_READONLY_API_KEY");
    }

    #[test]
    fn test_api_key_description() {
        let manager = AuthManager::new();

        let key = ApiKey {
            key: "test_key".to_string(),
            role: Role::User,
            user_id: "testuser".to_string(),
            description: "This is a test key for unit testing".to_string(),
        };

        manager.add_api_key(key.clone()).unwrap();

        let result = manager.validate_api_key("test_key").unwrap();
        assert_eq!(result.description, "This is a test key for unit testing");
    }

    #[test]
    fn test_multiple_users_same_role() {
        let manager = AuthManager::new();

        let user1 = ApiKey {
            key: "user1_key".to_string(),
            role: Role::User,
            user_id: "user1@example.com".to_string(),
            description: "User 1".to_string(),
        };

        let user2 = ApiKey {
            key: "user2_key".to_string(),
            role: Role::User,
            user_id: "user2@example.com".to_string(),
            description: "User 2".to_string(),
        };

        manager.add_api_key(user1).unwrap();
        manager.add_api_key(user2).unwrap();

        // Both should validate successfully
        assert!(manager.validate_api_key("user1_key").is_ok());
        assert!(manager.validate_api_key("user2_key").is_ok());

        // Both should have User role
        let key1 = manager.validate_api_key("user1_key").unwrap();
        let key2 = manager.validate_api_key("user2_key").unwrap();
        assert_eq!(key1.role, Role::User);
        assert_eq!(key2.role, Role::User);
    }

    // ── NEOLAND_REQUIRE_VAULT_KEYS guard (Phase 4.7) ─────────────────────────

    /// When NEOLAND_REQUIRE_VAULT_KEYS=1 and no API key env vars are set,
    /// new_with_secrets() falls back to dev keys and then panics.
    /// Uses tokio::task::spawn so the panic is caught as a JoinError,
    /// allowing cleanup to run in the parent task.
    #[tokio::test]
    async fn test_require_vault_keys_panics_with_dev_keys() {
        let _guard = ENV_LOCK.lock().await;

        // Ensure no real API keys are set (force dev key fallback)
        std::env::remove_var("NEOLAND_ADMIN_API_KEY");
        std::env::remove_var("NEOLAND_USER_API_KEY");
        std::env::remove_var("NEOLAND_READONLY_API_KEY");
        std::env::set_var("NEOLAND_REQUIRE_VAULT_KEYS", "1");

        let sm = Arc::new(SecretsManager::new().await.unwrap());
        let result = tokio::task::spawn(async move {
            AuthManager::new_with_secrets(&sm).await;
        })
        .await;

        // Cleanup before assertion (runs even if task panicked)
        std::env::remove_var("NEOLAND_REQUIRE_VAULT_KEYS");

        assert!(
            result.is_err(),
            "Task should have panicked due to dev keys with REQUIRE_VAULT_KEYS=1"
        );
    }

    /// When NEOLAND_REQUIRE_VAULT_KEYS=1 and real API keys are provided via
    /// env, new_with_secrets() must NOT panic.
    #[tokio::test]
    async fn test_require_vault_keys_passes_with_real_keys() {
        let _guard = ENV_LOCK.lock().await;

        std::env::set_var("NEOLAND_ADMIN_API_KEY", "prod_admin_key_abc123");
        std::env::set_var("NEOLAND_USER_API_KEY", "prod_user_key_xyz789");
        std::env::set_var("NEOLAND_READONLY_API_KEY", "prod_readonly_key_def456");
        std::env::set_var("NEOLAND_REQUIRE_VAULT_KEYS", "1");

        let sm = Arc::new(SecretsManager::new().await.unwrap());
        let manager = AuthManager::new_with_secrets(&sm).await;

        // Cleanup
        std::env::remove_var("NEOLAND_ADMIN_API_KEY");
        std::env::remove_var("NEOLAND_USER_API_KEY");
        std::env::remove_var("NEOLAND_READONLY_API_KEY");
        std::env::remove_var("NEOLAND_REQUIRE_VAULT_KEYS");

        // Should load real keys (no dev keys → no panic)
        assert!(manager.validate_api_key("prod_admin_key_abc123").is_ok());
        assert!(manager.validate_api_key("prod_user_key_xyz789").is_ok());
        assert!(manager.validate_api_key("prod_readonly_key_def456").is_ok());
    }
}
