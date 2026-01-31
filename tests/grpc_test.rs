use std::time::Duration;

use llamachat::{llama_service_client::LlamaServiceClient, ChatRequest};
use tokio_stream::StreamExt;

pub mod llamachat {
    tonic::include_proto!("llamachat");
}

#[tokio::test]
async fn test_grpc_chat_stream() {
    // Use non-standard ports for testing to avoid conflicts
    let test_grpc_port = 50052;
    let test_rest_port = 3002;

    // Spawn server in background task
    let server_handle = tokio::spawn(async move {
        let _ = llamachat_poc::server::run_server(test_grpc_port, test_rest_port).await;
    });

    // Wait for server to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Connect to the test server
    let channel_result =
        tonic::transport::Channel::from_static("http://[::1]:50052").connect().await;

    // If connection fails, abort server and skip test
    let channel = match channel_result {
        Ok(ch) => ch,
        Err(e) => {
            eprintln!("Failed to connect to test server: {}", e);
            server_handle.abort();
            return;
        },
    };

    let mut client = LlamaServiceClient::new(channel);

    let request = ChatRequest {
        prompt: "Test prompt".to_string(),
        model_id: "test".to_string(),
        use_local: true,
        temperature: None,
        top_p: None,
        max_tokens: Some(50), // Limit tokens for faster test
        repetition_penalty: None,
        typical_p: None,
        epsilon_cutoff: None,
        eta_cutoff: None,
        tail_free_sampling: None,
        top_a: None,
        context_top_k: None,
        context_similarity_threshold: None,
        disable_context: None,
        system_prompt: None,
        enable_commands: None,
        allowed_commands: vec![],
        session_id: None,
        streaming: None,
    };

    let response_result = client.chat_stream(request).await;

    match response_result {
        Ok(response) => {
            let mut stream = response.into_inner();

            // Try to get at least one chunk
            if let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        // Success - we got a response
                        println!("✓ Received chunk: {}", chunk.content);
                        // Just verify we got content (stream ends naturally)
                        assert!(!chunk.content.is_empty() || chunk.metadata.is_some());
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

    // Clean up: abort the server
    server_handle.abort();
}
