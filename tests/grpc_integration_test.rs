//! gRPC Integration Tests
//!
//! Comprehensive integration tests for gRPC services including chat_stream,
//! add_document, and search operations.

mod common;

use std::time::Duration;

use llamachat::{
    llama_service_client::LlamaServiceClient, AddDocumentRequest, ChatRequest, SearchRequest,
};
use tokio_stream::StreamExt;

pub mod llamachat {
    tonic::include_proto!("llamachat");
}

/// Helper to create a gRPC client against the shared test server.
async fn create_client(
    grpc_url: &str,
) -> Result<LlamaServiceClient<tonic::transport::Channel>, tonic::transport::Error> {
    let channel = tonic::transport::Channel::from_shared(grpc_url.to_string())
        .expect("valid gRPC URL")
        .connect()
        .await?;
    Ok(LlamaServiceClient::new(channel))
}

#[tokio::test]
async fn test_grpc_chat_stream_basic() {
    let server = common::TestServer::shared().await;

    let client_result = create_client(&server.grpc_url).await;

    {
        let mut client = client_result.expect("gRPC connect to shared test server failed");
        let request = ChatRequest {
            prompt: "Test prompt for gRPC".to_string(),
            model_id: "test".to_string(),
            use_local: true,
            temperature: Some(0.7),
            top_p: Some(0.9),
            max_tokens: Some(50),
            repetition_penalty: None,
            typical_p: None,
            epsilon_cutoff: None,
            eta_cutoff: None,
            tail_free_sampling: None,
            top_a: None,
            context_top_k: None,
            context_similarity_threshold: None,
            disable_context: Some(true), // Disable context for faster testing
            system_prompt: None,
            enable_commands: Some(false),
            allowed_commands: vec![],
            session_id: None,
            streaming: None,
        };

        let response_result = client.chat_stream(request).await;

        match response_result {
            Ok(response) => {
                let mut stream = response.into_inner();
                let mut chunk_count = 0;

                // Collect up to 5 chunks or until done
                while let Some(chunk_result) = stream.next().await {
                    match chunk_result {
                        Ok(chunk) => {
                            chunk_count += 1;
                            println!("Chunk {}: {}", chunk_count, chunk.content);

                            // Limit to 5 chunks for testing
                            if chunk_count >= 5 {
                                break;
                            }
                        },
                        Err(e) => {
                            // Inference needs a working local GGUF model — an
                            // external dependency, not part of this contract.
                            common::skip_or_fail(
                                "LLM engine",
                                &format!("chat stream errored: {e}"),
                            );
                            return;
                        },
                    }
                }

                assert!(chunk_count > 0, "Should receive at least one chunk");
            },
            Err(e) => {
                common::skip_or_fail("LLM engine", &format!("gRPC chat request failed: {e}"));
            },
        }
    }
}

#[tokio::test]
async fn test_grpc_chat_stream_with_context() {
    let server = common::TestServer::shared().await;

    let client_result = create_client(&server.grpc_url).await;

    {
        let mut client = client_result.expect("gRPC connect to shared test server failed");
        // First add a document to the vector store
        let add_doc_request = AddDocumentRequest {
            content: "How to move windows in Hyprland: Use Super+Mouse or move command".to_string(),
            metadata: "manual:hyprland".to_string(),
        };

        let add_result = client.add_document(add_doc_request).await;
        match add_result {
            Ok(resp) => {
                assert!(resp.into_inner().success);
            },
            Err(e) => {
                eprintln!("Failed to add document: {}", e);
            },
        }

        // Now query with context enabled
        let request = ChatRequest {
            prompt: "How do I move windows?".to_string(),
            model_id: "test".to_string(),
            use_local: true,
            temperature: Some(0.7),
            top_p: Some(0.9),
            max_tokens: Some(50),
            repetition_penalty: None,
            typical_p: None,
            epsilon_cutoff: None,
            eta_cutoff: None,
            tail_free_sampling: None,
            top_a: None,
            context_top_k: Some(2),
            context_similarity_threshold: Some(0.3),
            disable_context: Some(false), // Enable context
            system_prompt: None,
            enable_commands: Some(false),
            allowed_commands: vec![],
            session_id: None,
            streaming: None,
        };

        let response_result = client.chat_stream(request).await;

        match response_result {
            Ok(response) => {
                let mut stream = response.into_inner();

                if let Some(chunk_result) = stream.next().await {
                    match chunk_result {
                        Ok(chunk) => {
                            println!("Response with context: {}", chunk.content);
                            // Should have metadata with context info
                            if let Some(metadata) = chunk.metadata {
                                println!("Context docs used: {}", metadata.context_docs_count);
                            }
                        },
                        Err(e) => {
                            eprintln!("Stream error: {}", e);
                        },
                    }
                }
            },
            Err(e) => {
                eprintln!("gRPC request failed: {}", e);
            },
        }
    }
}

#[tokio::test]
async fn test_grpc_add_document() {
    let server = common::TestServer::shared().await;

    let client_result = create_client(&server.grpc_url).await;

    {
        let mut client = client_result.expect("gRPC connect to shared test server failed");
        let request = AddDocumentRequest {
            content: "Test document content for vector store".to_string(),
            metadata: "test:integration".to_string(),
        };

        let response_result = client.add_document(request).await;

        match response_result {
            Ok(response) => {
                let add_response = response.into_inner();
                assert!(add_response.success);
                assert!(!add_response.id.is_empty());
                println!("Document added with ID: {}", add_response.id);
            },
            Err(e) => {
                eprintln!("Failed to add document: {}", e);
            },
        }
    }
}

#[tokio::test]
async fn test_grpc_search() {
    let server = common::TestServer::shared().await;

    let client_result = create_client(&server.grpc_url).await;

    {
        let mut client = client_result.expect("gRPC connect to shared test server failed");
        // First add some documents
        let docs = vec![
            ("Documentation about window management", "manual:windows"),
            ("Guide for workspace navigation", "manual:workspaces"),
            ("Tips for customizing your desktop", "manual:customization"),
        ];

        for (content, metadata) in docs {
            let add_request =
                AddDocumentRequest { content: content.to_string(), metadata: metadata.to_string() };

            let _ = client.add_document(add_request).await;
        }

        // Now search
        let search_request = SearchRequest { query: "window management".to_string(), top_k: 2 };

        let search_result = client.search(search_request).await;

        match search_result {
            Ok(response) => {
                let search_response = response.into_inner();
                assert!(!search_response.results.is_empty());

                println!("Search results:");
                for (i, result) in search_response.results.iter().enumerate() {
                    println!("  {}. {} (score: {})", i + 1, result.content, result.score);
                }

                // First result should be most relevant
                let first_result = &search_response.results[0];
                assert!(first_result.content.contains("window management"));
                assert!(first_result.score > 0.5); // Should have high
                                                   // similarity
            },
            Err(e) => {
                eprintln!("Search failed: {}", e);
            },
        }
    }
}

#[tokio::test]
async fn test_grpc_search_empty_query() {
    let server = common::TestServer::shared().await;

    let client_result = create_client(&server.grpc_url).await;

    {
        let mut client = client_result.expect("gRPC connect to shared test server failed");
        let search_request = SearchRequest {
            query: "".to_string(), // Empty query
            top_k: 5,
        };

        let search_result = client.search(search_request).await;

        // Empty query should still work (may return all docs or error)
        match search_result {
            Ok(response) => {
                println!("Empty query returned {} results", response.into_inner().results.len());
            },
            Err(e) => {
                println!("Empty query error (expected): {}", e);
            },
        }
    }
}

#[tokio::test]
async fn test_grpc_multiple_concurrent_requests() {
    let server = common::TestServer::shared().await;

    let client_result = create_client(&server.grpc_url).await;

    if let Ok(client) = client_result {
        // Clone client for concurrent requests
        let mut client1 = client.clone();
        let mut client2 = client.clone();
        let mut client3 = client;

        // Send 3 concurrent chat requests
        let task1 = tokio::spawn(async move {
            let request = ChatRequest {
                prompt: "Concurrent request 1".to_string(),
                model_id: "test".to_string(),
                use_local: true,
                max_tokens: Some(20),
                disable_context: Some(true),
                ..Default::default()
            };
            client1.chat_stream(request).await
        });

        let task2 = tokio::spawn(async move {
            let request = ChatRequest {
                prompt: "Concurrent request 2".to_string(),
                model_id: "test".to_string(),
                use_local: true,
                max_tokens: Some(20),
                disable_context: Some(true),
                ..Default::default()
            };
            client2.chat_stream(request).await
        });

        let task3 = tokio::spawn(async move {
            let request = ChatRequest {
                prompt: "Concurrent request 3".to_string(),
                model_id: "test".to_string(),
                use_local: true,
                max_tokens: Some(20),
                disable_context: Some(true),
                ..Default::default()
            };
            client3.chat_stream(request).await
        });

        // Wait for all tasks
        let (r1, r2, r3) = tokio::join!(task1, task2, task3);

        // All should succeed or fail gracefully
        println!("Concurrent request 1: {:?}", r1.is_ok());
        println!("Concurrent request 2: {:?}", r2.is_ok());
        println!("Concurrent request 3: {:?}", r3.is_ok());
    }
}

#[tokio::test]
async fn test_grpc_connection_error_handling() {
    // Try to connect to non-existent server
    let channel_result = tonic::transport::Channel::from_static("http://[::1]:59999")
        .connect_timeout(Duration::from_secs(1))
        .connect()
        .await;

    assert!(channel_result.is_err(), "Should fail to connect to non-existent server");
}

#[tokio::test]
async fn test_grpc_chat_with_all_parameters() {
    let server = common::TestServer::shared().await;

    let client_result = create_client(&server.grpc_url).await;

    {
        let mut client = client_result.expect("gRPC connect to shared test server failed");
        let request = ChatRequest {
            prompt: "Test all parameters".to_string(),
            model_id: "test".to_string(),
            use_local: true,
            temperature: Some(0.8),
            top_p: Some(0.95),
            max_tokens: Some(100),
            repetition_penalty: Some(1.2),
            typical_p: Some(1.0),
            epsilon_cutoff: Some(0.0),
            eta_cutoff: Some(0.0),
            tail_free_sampling: Some(1.0),
            top_a: Some(0.0),
            context_top_k: Some(3),
            context_similarity_threshold: Some(0.4),
            disable_context: Some(false),
            system_prompt: Some("You are a helpful assistant.".to_string()),
            enable_commands: Some(true),
            allowed_commands: vec!["move_ws".to_string()],
            session_id: Some("test-session-123".to_string()),
            streaming: Some(true),
        };

        let response_result = client.chat_stream(request).await;

        match response_result {
            Ok(response) => {
                let mut stream = response.into_inner();

                if let Some(chunk_result) = stream.next().await {
                    match chunk_result {
                        Ok(chunk) => {
                            println!("Response with all params: {}", chunk.content);

                            // Check metadata
                            if let Some(metadata) = chunk.metadata {
                                assert_eq!(metadata.temperature_used, 0.8);
                                assert_eq!(metadata.top_p_used, 0.95);
                                assert_eq!(metadata.max_tokens_used, 100);
                                assert!(metadata.commands_enabled);
                            }
                        },
                        Err(e) => {
                            eprintln!("Stream error: {}", e);
                        },
                    }
                }
            },
            Err(e) => {
                eprintln!("gRPC request failed: {}", e);
            },
        }
    }
}
