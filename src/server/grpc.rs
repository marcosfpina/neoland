//! Serviço gRPC (`LlamaService`) sobre tonic.

use super::*;

// gRPC Service Implementation
pub struct MyLlamaService {
    pub(crate) state: Arc<AppState>,
}

#[tonic::async_trait]
impl LlamaService for MyLlamaService {
    type ChatStreamStream = ReceiverStream<Result<ChatResponse, Status>>;

    async fn chat_stream(
        &self,
        request: Request<ChatRequest>,
    ) -> Result<Response<Self::ChatStreamStream>, Status> {
        let grpc_start = Instant::now();
        crate::metrics::GRPC_REQUESTS_TOTAL
            .with_label_values(&["chat_stream", "started"])
            .inc();

        let req = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let state = self.state.clone();

        // Parse config from request
        let mut config = GenerationConfig::default();

        if let Some(temp) = req.temperature {
            config.temperature = temp as f64;
        }
        if let Some(top_p) = req.top_p {
            config.top_p = top_p as f64;
        }
        if let Some(max_tokens) = req.max_tokens {
            config.max_tokens = max_tokens as usize;
        }
        if let Some(rep_penalty) = req.repetition_penalty {
            config.repetition_penalty = rep_penalty;
        }
        if let Some(typical_p) = req.typical_p {
            config.typical_p = typical_p as f64;
        }
        if let Some(epsilon_cutoff) = req.epsilon_cutoff {
            config.epsilon_cutoff = epsilon_cutoff as f64;
        }
        if let Some(eta_cutoff) = req.eta_cutoff {
            config.eta_cutoff = eta_cutoff as f64;
        }
        if let Some(tail_free_sampling) = req.tail_free_sampling {
            config.tail_free_sampling = tail_free_sampling as f64;
        }
        if let Some(top_a) = req.top_a {
            config.top_a = top_a as f64;
        }
        if let Some(ctx_k) = req.context_top_k {
            config.context_top_k = ctx_k as usize;
        }
        if let Some(ctx_threshold) = req.context_similarity_threshold {
            config.context_similarity_threshold = ctx_threshold;
        }
        if let Some(disable_ctx) = req.disable_context {
            config.disable_context = disable_ctx;
        }
        if let Some(sys_prompt) = req.system_prompt {
            config.system_prompt = Some(sys_prompt);
        }
        if let Some(enable_cmds) = req.enable_commands {
            config.enable_commands = enable_cmds;
        }
        if !req.allowed_commands.is_empty() {
            config.allowed_commands = req.allowed_commands.clone();
        }

        tokio::task::spawn_blocking(move || {
            let mut engine_guard = match state.engine.lock() {
                Ok(guard) => guard,
                Err(e) => {
                    let _ =
                        tx.blocking_send(Err(Status::internal(format!("Mutex poisoned: {}", e))));
                    return;
                },
            };
            if engine_guard.is_none() {
                match LocalEngine::new() {
                    Ok(e) => *engine_guard = Some(e),
                    Err(err) => {
                        tracing::error!(error = %err, "gRPC: failed to initialize LocalEngine");
                        let _ = tx.blocking_send(Err(Status::internal(err.to_string())));
                        return;
                    },
                }
            }
            if let Some(engine) = engine_guard.as_mut() {
                let vs_guard = match state.vector_store.lock() {
                    Ok(guard) => guard,
                    Err(e) => {
                        let _ = tx.blocking_send(Err(Status::internal(format!(
                            "VectorStore mutex poisoned: {}",
                            e
                        ))));
                        return;
                    },
                };

                // Send metadata first
                let metadata = llamachat::ResponseMetadata {
                    temperature_used: config.temperature as f32,
                    top_p_used: config.top_p as f32,
                    max_tokens_used: config.max_tokens as i32,
                    context_docs_count: 0, // Will be updated after generation
                    context_doc_ids: vec![],
                    system_prompt_used: config
                        .system_prompt
                        .clone()
                        .unwrap_or_else(|| "default".to_string()),
                    commands_enabled: config.enable_commands,
                };

                let _ = tx.blocking_send(Ok(ChatResponse {
                    content: String::new(),
                    is_command: false,
                    metadata: Some(metadata.clone()),
                }));

                match engine.generate_stream(&req.prompt, Some(&vs_guard), &config, |token| {
                    // Filter commands if needed
                    let should_send = if token.contains("[[CMD:") {
                        if !config.enable_commands {
                            false
                        } else if !config.allowed_commands.is_empty() {
                            // Check if command is in whitelist
                            config.allowed_commands.iter().any(|cmd| token.contains(cmd))
                        } else {
                            true
                        }
                    } else {
                        true
                    };

                    if should_send {
                        let _ = tx.blocking_send(Ok(ChatResponse {
                            is_command: token.contains("[[CMD:"),
                            content: token,
                            metadata: None,
                        }));
                    }
                }) {
                    Ok(context_doc_ids) => {
                        // Send final metadata update with context info
                        let final_metadata = llamachat::ResponseMetadata {
                            temperature_used: config.temperature as f32,
                            top_p_used: config.top_p as f32,
                            max_tokens_used: config.max_tokens as i32,
                            context_docs_count: context_doc_ids.len() as i32,
                            context_doc_ids: context_doc_ids.clone(),
                            system_prompt_used: config
                                .system_prompt
                                .clone()
                                .unwrap_or_else(|| "default".to_string()),
                            commands_enabled: config.enable_commands,
                        };
                        let _ = tx.blocking_send(Ok(ChatResponse {
                            content: "[[METADATA_UPDATE]]".to_string(),
                            is_command: false,
                            metadata: Some(final_metadata),
                        }));
                    },
                    Err(e) => {
                        let _ = tx.blocking_send(Err(Status::internal(e.to_string())));
                    },
                }
            }
        });
        crate::metrics::GRPC_REQUESTS_TOTAL
            .with_label_values(&["chat_stream", "ok"])
            .inc();
        crate::metrics::GRPC_REQUEST_DURATION_SECONDS
            .with_label_values(&["chat_stream"])
            .observe(grpc_start.elapsed().as_secs_f64());

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn add_document(
        &self,
        request: Request<AddDocumentRequest>,
    ) -> Result<Response<AddDocumentResponse>, Status> {
        let grpc_start = Instant::now();
        let req = request.into_inner();

        // 1. Add to in-memory VectorStore (sync, used for context injection)
        let doc_id = {
            let mut vs = self
                .state
                .vector_store
                .lock()
                .map_err(|e| Status::internal(format!("VectorStore mutex poisoned: {}", e)))?;
            match vs.add_document(&req.content, &req.metadata) {
                Ok(()) => {
                    crate::metrics::utils::update_vector_store_documents(vs.len());
                    uuid::Uuid::new_v4().to_string()
                },
                Err(e) => {
                    tracing::error!(error = %e, "gRPC: add_document (in-memory) failed");
                    crate::metrics::GRPC_REQUESTS_TOTAL
                        .with_label_values(&["add_document", "error"])
                        .inc();
                    crate::metrics::GRPC_REQUEST_DURATION_SECONDS
                        .with_label_values(&["add_document"])
                        .observe(grpc_start.elapsed().as_secs_f64());
                    return Err(Status::internal(e.to_string()));
                },
            }
        };

        // 2. Mirror to PersistentVectorStore if configured (fire-and-forget)
        if let Some(ps) = self.state.persistent_store.clone() {
            let content = req.content.clone();
            let metadata = req.metadata.clone();
            tokio::spawn(async move {
                match ps.add_document(&content, &metadata).await {
                    Ok(uuid) => tracing::debug!(
                        uuid = %uuid,
                        "Mirrored document to PersistentVectorStore"
                    ),
                    Err(e) => tracing::warn!(
                        error = %e,
                        "Failed to mirror document to PersistentVectorStore (in-memory write succeeded)"
                    ),
                }
            });
        }

        tracing::info!(id = %doc_id, metadata = %req.metadata, "gRPC: document added");

        crate::metrics::GRPC_REQUESTS_TOTAL
            .with_label_values(&["add_document", "ok"])
            .inc();
        crate::metrics::GRPC_REQUEST_DURATION_SECONDS
            .with_label_values(&["add_document"])
            .observe(grpc_start.elapsed().as_secs_f64());

        Ok(Response::new(AddDocumentResponse { id: doc_id, success: true }))
    }

    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        let grpc_start = Instant::now();
        let req = request.into_inner();

        // Prefer PersistentVectorStore (pgvector) when configured; fall back to
        // in-memory.
        if let Some(ps) = &self.state.persistent_store {
            match ps.search(&req.query, req.top_k as usize, None).await {
                Ok(results) => {
                    let grpc_results = results
                        .into_iter()
                        .map(|(doc, score)| llamachat::Document {
                            id: doc.id.to_string(),
                            content: doc.content,
                            score,
                        })
                        .collect();
                    crate::metrics::GRPC_REQUESTS_TOTAL.with_label_values(&["search", "ok"]).inc();
                    crate::metrics::GRPC_REQUEST_DURATION_SECONDS
                        .with_label_values(&["search"])
                        .observe(grpc_start.elapsed().as_secs_f64());
                    return Ok(Response::new(SearchResponse { results: grpc_results }));
                },
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        query = %req.query,
                        "PersistentVectorStore search failed, falling back to in-memory"
                    );
                },
            }
        }

        // In-memory fallback
        let vs = self
            .state
            .vector_store
            .lock()
            .map_err(|e| Status::internal(format!("VectorStore mutex poisoned: {}", e)))?;
        let result = match vs.search(&req.query, req.top_k as usize) {
            Ok(results) => {
                let grpc_results = results
                    .into_iter()
                    .map(|(doc, score)| llamachat::Document {
                        id: doc.id,
                        content: doc.content,
                        score,
                    })
                    .collect();
                Ok(Response::new(SearchResponse { results: grpc_results }))
            },
            Err(e) => {
                tracing::error!(error = %e, query = %req.query, "gRPC: search failed");
                Err(Status::internal(e.to_string()))
            },
        };
        let status_label = if result.is_ok() { "ok" } else { "error" };
        crate::metrics::GRPC_REQUESTS_TOTAL
            .with_label_values(&["search", status_label])
            .inc();
        crate::metrics::GRPC_REQUEST_DURATION_SECONDS
            .with_label_values(&["search"])
            .observe(grpc_start.elapsed().as_secs_f64());
        result
    }
}
