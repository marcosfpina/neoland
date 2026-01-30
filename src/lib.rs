// Library crate para Neoland
// Expõe modules que podem ser usados por binaries
pub mod cli;
pub mod engine;
pub mod nlp;
pub mod hyprland_ops;
pub mod tui;
pub mod ml_offload;
pub mod llm;
pub mod auth;

// Re-export server function e tipos gRPC
pub mod server;

// Re-export tipos gRPC para TUI
pub use server::llamachat;
