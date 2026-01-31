//! Input Validation Module
//!
//! This module provides input validation and sanitization for all user inputs.
//! It protects against oversized requests, malformed data, and potential
//! security issues.
//!
//! See ADR-014 for validation strategy decisions.

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Maximum prompt size in bytes (100KB)
pub const MAX_PROMPT_SIZE: usize = 100 * 1024;

/// Maximum number of messages in a conversation
pub const MAX_MESSAGE_COUNT: usize = 100;

/// Maximum metadata size in bytes
pub const MAX_METADATA_SIZE: usize = 10 * 1024;

/// Validation error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    PromptTooLarge { size: usize, max: usize },
    TooManyMessages { count: usize, max: usize },
    MetadataTooLarge { size: usize, max: usize },
    EmptyPrompt,
    InvalidCharacters { field: String },
    MalformedRequest { reason: String },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PromptTooLarge { size, max } => {
                write!(f, "Prompt too large: {} bytes (max: {} bytes)", size, max)
            },
            Self::TooManyMessages { count, max } => {
                write!(f, "Too many messages: {} (max: {})", count, max)
            },
            Self::MetadataTooLarge { size, max } => {
                write!(f, "Metadata too large: {} bytes (max: {} bytes)", size, max)
            },
            Self::EmptyPrompt => write!(f, "Prompt cannot be empty"),
            Self::InvalidCharacters { field } => {
                write!(f, "Invalid characters in field: {}", field)
            },
            Self::MalformedRequest { reason } => {
                write!(f, "Malformed request: {}", reason)
            },
        }
    }
}

impl std::error::Error for ValidationError {}

/// Chat message validation
#[derive(Debug)]
pub struct MessageValidator;

impl MessageValidator {
    /// Validate a single prompt
    pub fn validate_prompt(prompt: &str) -> Result<(), ValidationError> {
        // Check if empty
        if prompt.trim().is_empty() {
            return Err(ValidationError::EmptyPrompt);
        }

        // Check size
        let size = prompt.len();
        if size > MAX_PROMPT_SIZE {
            return Err(ValidationError::PromptTooLarge { size, max: MAX_PROMPT_SIZE });
        }

        // Check for null bytes (potential injection)
        if prompt.contains('\0') {
            return Err(ValidationError::InvalidCharacters { field: "prompt".to_string() });
        }

        Ok(())
    }

    /// Validate message count
    pub fn validate_message_count(count: usize) -> Result<(), ValidationError> {
        if count > MAX_MESSAGE_COUNT {
            return Err(ValidationError::TooManyMessages { count, max: MAX_MESSAGE_COUNT });
        }
        Ok(())
    }

    /// Sanitize user input (remove potentially dangerous characters)
    pub fn sanitize_input(input: &str) -> String {
        input
            .chars()
            .filter(|c| {
                // Allow most printable characters, newlines, tabs
                c.is_ascii_graphic()
                    || c.is_ascii_whitespace()
                    || !c.is_ascii() // Allow non-ASCII (UTF-8)
            })
            .filter(|c| *c != '\0') // Remove null bytes
            .collect()
    }
}

/// Request size validator
#[derive(Debug)]
pub struct RequestValidator;

impl RequestValidator {
    /// Validate total request body size
    pub fn validate_body_size(size: usize) -> Result<(), ValidationError> {
        const MAX_REQUEST_SIZE: usize = 1024 * 1024; // 1MB

        if size > MAX_REQUEST_SIZE {
            return Err(ValidationError::MalformedRequest {
                reason: format!("Request body too large: {} bytes (max: 1MB)", size),
            });
        }

        Ok(())
    }

    /// Validate metadata size
    pub fn validate_metadata_size(metadata: &serde_json::Value) -> Result<(), ValidationError> {
        let size = serde_json::to_string(metadata).map(|s| s.len()).unwrap_or(0);

        if size > MAX_METADATA_SIZE {
            return Err(ValidationError::MetadataTooLarge { size, max: MAX_METADATA_SIZE });
        }

        Ok(())
    }
}

/// Chat request validation (for REST API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequestValidation {
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatRequestValidation {
    /// Validate the entire chat request
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Validate message count
        MessageValidator::validate_message_count(self.messages.len())?;

        // Validate each message
        for msg in &self.messages {
            // Validate role
            if !["user", "assistant", "system"].contains(&msg.role.as_str()) {
                return Err(ValidationError::MalformedRequest {
                    reason: format!("Invalid role: {}", msg.role),
                });
            }

            // Validate content
            MessageValidator::validate_prompt(&msg.content)?;
        }

        // Validate metadata size
        RequestValidator::validate_metadata_size(&self.metadata)?;

        Ok(())
    }

    /// Sanitize all messages in the request
    pub fn sanitize(&mut self) {
        for msg in &mut self.messages {
            msg.content = MessageValidator::sanitize_input(&msg.content);
        }
    }
}

/// Document upload validation
#[derive(Debug)]
pub struct DocumentValidator;

impl DocumentValidator {
    /// Maximum document size (10MB)
    pub const MAX_DOCUMENT_SIZE: usize = 10 * 1024 * 1024;

    /// Validate document content
    pub fn validate_document(content: &[u8], filename: &str) -> Result<(), ValidationError> {
        // Check size
        if content.len() > Self::MAX_DOCUMENT_SIZE {
            return Err(ValidationError::MalformedRequest {
                reason: format!("Document too large: {} bytes (max: 10MB)", content.len()),
            });
        }

        // Validate filename (prevent path traversal)
        if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
            return Err(ValidationError::InvalidCharacters { field: "filename".to_string() });
        }

        // Check for empty filename
        if filename.trim().is_empty() {
            return Err(ValidationError::MalformedRequest {
                reason: "Filename cannot be empty".to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_prompt_success() {
        let result = MessageValidator::validate_prompt("Hello, world!");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_prompt_empty() {
        let result = MessageValidator::validate_prompt("");
        assert!(matches!(result, Err(ValidationError::EmptyPrompt)));
    }

    #[test]
    fn test_validate_prompt_too_large() {
        let large_prompt = "a".repeat(MAX_PROMPT_SIZE + 1);
        let result = MessageValidator::validate_prompt(&large_prompt);
        assert!(matches!(result, Err(ValidationError::PromptTooLarge { .. })));
    }

    #[test]
    fn test_validate_prompt_null_byte() {
        let result = MessageValidator::validate_prompt("Hello\0World");
        assert!(matches!(result, Err(ValidationError::InvalidCharacters { .. })));
    }

    #[test]
    fn test_sanitize_input() {
        let input = "Hello\0World\x01Test";
        let sanitized = MessageValidator::sanitize_input(input);
        assert_eq!(sanitized, "HelloWorldTest");
    }

    #[test]
    fn test_validate_message_count() {
        let result = MessageValidator::validate_message_count(50);
        assert!(result.is_ok());

        let result = MessageValidator::validate_message_count(MAX_MESSAGE_COUNT + 1);
        assert!(matches!(result, Err(ValidationError::TooManyMessages { .. })));
    }

    #[test]
    fn test_chat_request_validation() {
        let request = ChatRequestValidation {
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Test message".to_string(),
            }],
            metadata: serde_json::json!({}),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_chat_request_invalid_role() {
        let request = ChatRequestValidation {
            messages: vec![ChatMessage {
                role: "invalid".to_string(),
                content: "Test message".to_string(),
            }],
            metadata: serde_json::json!({}),
        };

        assert!(matches!(request.validate(), Err(ValidationError::MalformedRequest { .. })));
    }

    #[test]
    fn test_document_validation() {
        let content = b"Hello, world!";
        let result = DocumentValidator::validate_document(content, "test.txt");
        assert!(result.is_ok());
    }

    #[test]
    fn test_document_path_traversal() {
        let content = b"Hello";
        let result = DocumentValidator::validate_document(content, "../etc/passwd");
        assert!(matches!(result, Err(ValidationError::InvalidCharacters { .. })));
    }

    #[test]
    fn test_document_too_large() {
        let large_content = vec![0u8; DocumentValidator::MAX_DOCUMENT_SIZE + 1];
        let result = DocumentValidator::validate_document(&large_content, "large.bin");
        assert!(matches!(result, Err(ValidationError::MalformedRequest { .. })));
    }
}
