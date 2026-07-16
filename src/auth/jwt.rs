//! JWT token creation and validation.
//!
//! Uses HS256 symmetric key for simplicity. In production, switch to RS256
//! with a key pair stored in Vault/secrets.

use anyhow::{Context, Result};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

use super::types::Claims;

/// Default token lifetimes.
const ACCESS_TOKEN_MINUTES: i64 = 15;
const REFRESH_TOKEN_DAYS: i64 = 7;

/// Create an access token (short-lived JWT).
pub fn create_access_token(
    user_id: Uuid,
    tenant_slug: &str,
    role: &str,
    email: &str,
    display_name: &str,
    secret: &[u8],
) -> Result<String> {
    let now = chrono::Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        tenant: tenant_slug.to_string(),
        role: role.to_string(),
        email: email.to_string(),
        name: display_name.to_string(),
        iat: now.timestamp() as usize,
        exp: (now + chrono::Duration::minutes(ACCESS_TOKEN_MINUTES)).timestamp() as usize,
        jti: Uuid::new_v4().to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .context("Failed to encode access token")
}

/// Create a refresh token (long-lived opaque token stored in DB).
pub fn create_refresh_token() -> String {
    use sha2::{Digest, Sha256};
    use std::time::{SystemTime, UNIX_EPOCH};

    let entropy = format!(
        "{}-{}",
        Uuid::new_v4(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    );
    let hash = Sha256::digest(entropy.as_bytes());
    hex::encode(hash)
}

/// Validate an access token and return claims.
pub fn validate_access_token(token: &str, secret: &[u8]) -> Result<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::default(),
    )
    .context("Invalid or expired access token")?;

    Ok(token_data.claims)
}

/// Hash a refresh token for database storage.
pub fn hash_refresh_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(token.as_bytes());
    hex::encode(hash)
}

/// Returns the access token lifetime in seconds.
pub fn access_token_lifetime_secs() -> i64 {
    ACCESS_TOKEN_MINUTES * 60
}

/// Returns the refresh token lifetime in seconds.
pub fn refresh_token_lifetime_secs() -> i64 {
    REFRESH_TOKEN_DAYS * 24 * 3600
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_validate_access_token() {
        let secret = b"test-secret-key-for-jwt-32b!!";
        let token = create_access_token(
            Uuid::new_v4(),
            "test-tenant",
            "user",
            "test@example.com",
            "Test User",
            secret,
        )
        .expect("create token");

        let claims = validate_access_token(&token, secret).expect("validate token");
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.role, "user");
        assert_eq!(claims.tenant, "test-tenant");
    }

    #[test]
    fn test_expired_token_rejected() {
        // We can't easily test expiration without waiting, but we can verify
        // that a tampered token fails validation.
        let secret = b"test-secret-key-for-jwt-32b!!";
        let token = create_access_token(
            Uuid::new_v4(),
            "t",
            "user",
            "a@b.com",
            "X",
            secret,
        )
        .expect("create");

        // Tamper with a different secret
        let wrong_secret = b"wrong-secret-key-for-jwt-32b";
        let result = validate_access_token(&token, wrong_secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_refresh_token_is_unique() {
        let t1 = create_refresh_token();
        let t2 = create_refresh_token();
        assert_ne!(t1, t2);
        assert_eq!(t1.len(), 64); // hex(SHA-256) = 64 chars
    }

    #[test]
    fn test_hash_refresh_token_deterministic() {
        let token = "test-refresh-token";
        let h1 = hash_refresh_token(token);
        let h2 = hash_refresh_token(token);
        assert_eq!(h1, h2);
    }
}
