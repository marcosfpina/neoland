// Library crate para Neoland
// Expõe modules que podem ser usados por binaries
pub mod agents;
pub mod audit;
pub mod auth;
pub mod cli;
pub mod commands;
pub mod config;
pub mod engine;
pub mod health; // Phase 4.3: Health Checks & Readiness Probes
pub mod hyprland_ops;
pub mod llm;
pub mod logging; // Phase 4.2: Structured Logging
pub mod mcp;
pub mod metrics; // Phase 4.1: Prometheus Metrics
pub mod ml_offload;
pub mod nlp;
pub mod openapi; // Ciclo 3: OpenAPI spec + Swagger UI
pub mod secrets;
pub mod storage; // Phase 4.7: Persistent Vector Store
pub mod tui;
pub mod validation;

// Re-export server function e tipos gRPC
pub mod server;

// Test utilities (only compiled for tests)
#[cfg(test)]
pub mod test_utils;

// Re-export tipos gRPC para TUI
pub use server::llamachat;
