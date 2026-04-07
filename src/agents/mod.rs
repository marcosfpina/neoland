//! Agent pipeline control plane.
//!
//! This module orchestrates the multi-agent DSPy pipeline (Python/FastAPI) from Rust.
//! The Python side reasons; this side orchestrates, manages sessions, and persists state.

pub mod checkpoint_store;
pub mod client;
pub mod escalation;
pub mod orchestrator;
pub mod session;
