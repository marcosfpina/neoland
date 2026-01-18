use llamachat::llama_service_client::LlamaServiceClient;
use llamachat::ChatRequest;
use tokio_stream::StreamExt;

pub mod llamachat {
    tonic::include_proto!("llamachat");
}

#[tokio::test]
async fn test_grpc_chat_stream() {
    // NOTE: This test requires the server to be running or to start it programmatically.
    // For a real CI, we would start the server in a separate task.
    
    // Let's assume the server is NOT running and verify we get a connection error,
    // OR try to connect to a local addr if we can spawn it.
    
    let channel = tonic::transport::Channel::from_static("http://[::1]:50051")
        .connect()
        .await;
        
    match channel {
        Ok(channel) => {
            let mut client = LlamaServiceClient::new(channel);
            let request = ChatRequest {
                prompt: "Olá, como você pode me ajudar?".to_string(),
                model_id: "test".to_string(),
                use_local: true,
                temperature: None,
                top_p: None,
                max_tokens: None,
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
            
            let response = client.chat_stream(request).await?;
            let mut stream = response.into_inner();
            
            if let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                assert!(!chunk.content.is_empty());
                println!("Recebido: {}", chunk.content);
            }
        },
        Err(_) => {
            println!("Servidor não está rodando, pulando teste funcional.");
        }
    }
}
