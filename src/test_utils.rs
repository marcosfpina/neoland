//! Test Utilities Module
//!
//! This module provides common test utilities, mocks, and helpers for testing
//! NEOLAND components. It is only compiled when running tests (#[cfg(test)]).

#[cfg(test)]
pub mod mocks {
    use std::sync::Arc;

    use anyhow::Result;

    use crate::{auth::AuthManager, secrets::SecretsManager};

    /// Create a test SecretsManager with environment variable fallback
    pub async fn test_secrets_manager() -> Result<Arc<SecretsManager>> {
        let manager = SecretsManager::new().await?;
        Ok(Arc::new(manager))
    }

    /// Create a test AuthManager
    pub fn test_auth_manager() -> Arc<AuthManager> {
        Arc::new(AuthManager::new())
    }

    /// Set up test environment variables for secrets
    pub fn setup_test_env_vars() {
        std::env::set_var("DEEPSEEK_API_KEY", "test_deepseek_key");
        std::env::set_var("OPENAI_API_KEY", "test_openai_key");
        std::env::set_var("NEOLAND_ADMIN_API_KEY", "test_admin_key");
        std::env::set_var("NEOLAND_USER_API_KEY", "test_user_key");
        std::env::set_var("NEOLAND_READONLY_API_KEY", "test_readonly_key");
    }

    /// Clean up test environment variables
    pub fn cleanup_test_env_vars() {
        std::env::remove_var("DEEPSEEK_API_KEY");
        std::env::remove_var("OPENAI_API_KEY");
        std::env::remove_var("NEOLAND_ADMIN_API_KEY");
        std::env::remove_var("NEOLAND_USER_API_KEY");
        std::env::remove_var("NEOLAND_READONLY_API_KEY");
    }

    /// Sample chat request for testing
    pub fn sample_chat_request() -> crate::validation::ChatRequestValidation {
        crate::validation::ChatRequestValidation {
            messages: vec![crate::validation::ChatMessage {
                role: "user".to_string(),
                content: "Hello, world!".to_string(),
            }],
            metadata: serde_json::json!({}),
        }
    }

    /// Large chat request for testing size limits
    pub fn large_chat_request(size_kb: usize) -> crate::validation::ChatRequestValidation {
        let content = "a".repeat(size_kb * 1024);
        crate::validation::ChatRequestValidation {
            messages: vec![crate::validation::ChatMessage { role: "user".to_string(), content }],
            metadata: serde_json::json!({}),
        }
    }

    /// Many messages chat request for testing message count limits
    pub fn many_messages_request(count: usize) -> crate::validation::ChatRequestValidation {
        let messages = (0..count)
            .map(|i| crate::validation::ChatMessage {
                role: "user".to_string(),
                content: format!("Message {}", i),
            })
            .collect();

        crate::validation::ChatRequestValidation { messages, metadata: serde_json::json!({}) }
    }
}

#[cfg(test)]
pub mod assertions {
    use anyhow::Result;

    /// Assert that a result contains a specific error message
    pub fn assert_error_contains<T: std::fmt::Debug>(result: &Result<T>, expected: &str) {
        assert!(result.is_err(), "Expected error, got Ok");
        let err = result.as_ref().unwrap_err();
        assert!(
            err.to_string().contains(expected),
            "Error message '{}' does not contain '{}'",
            err.to_string(),
            expected
        );
    }

    /// Assert that a string contains sensitive data that should be redacted
    pub fn assert_no_sensitive_data(text: &str, sensitive_keywords: &[&str]) {
        for keyword in sensitive_keywords {
            assert!(
                !text.to_lowercase().contains(&keyword.to_lowercase()),
                "Text contains sensitive keyword: {}",
                keyword
            );
        }
    }
}

#[cfg(test)]
pub mod fixtures {
    /// Common test fixture: valid API key
    pub const VALID_API_KEY: &str = "neoland_test_key_12345";

    /// Common test fixture: invalid API key
    pub const INVALID_API_KEY: &str = "invalid_key";

    /// Common test fixture: admin user ID
    pub const ADMIN_USER_ID: &str = "admin@test.com";

    /// Common test fixture: regular user ID
    pub const USER_ID: &str = "user@test.com";

    /// Common test fixture: readonly user ID
    pub const READONLY_USER_ID: &str = "readonly@test.com";

    /// Common test fixture: test IP address
    pub const TEST_IP: &str = "203.0.113.42";

    /// Common test fixture: prompt for testing
    pub const TEST_PROMPT: &str = "Hello, this is a test prompt for NEOLAND!";

    /// Common test fixture: system prompt
    pub const SYSTEM_PROMPT: &str = "You are a helpful AI assistant for testing purposes.";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_secrets_manager_creation() {
        let manager = mocks::test_secrets_manager().await;
        assert!(manager.is_ok());
    }

    #[test]
    fn test_auth_manager_creation() {
        let manager = mocks::test_auth_manager();
        // Use the default development admin key
        assert!(manager.validate_api_key("neoland_admin_dev_key_change_in_production").is_ok());
    }

    #[test]
    fn test_sample_chat_request() {
        let req = mocks::sample_chat_request();
        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.messages[0].role, "user");
    }

    #[test]
    fn test_large_chat_request() {
        let req = mocks::large_chat_request(10); // 10KB
        assert!(req.messages[0].content.len() >= 10 * 1024);
    }

    #[test]
    fn test_many_messages_request() {
        let req = mocks::many_messages_request(50);
        assert_eq!(req.messages.len(), 50);
    }

    #[test]
    fn test_assert_error_contains() {
        let result: anyhow::Result<()> = Err(anyhow::anyhow!("Test error message"));
        assertions::assert_error_contains(&result, "Test error");
    }

    #[test]
    fn test_assert_no_sensitive_data() {
        let text = "This is a safe message";
        assertions::assert_no_sensitive_data(&text, &["password", "secret"]);
    }

    #[test]
    #[should_panic(expected = "contains sensitive keyword")]
    fn test_assert_no_sensitive_data_fails() {
        let text = "This message contains a password: hunter2";
        assertions::assert_no_sensitive_data(&text, &["password"]);
    }
}
