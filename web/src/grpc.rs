//! Protobuf message types for the Neoland control plane.
//!
//! These are the WASM-compatible prost-generated types from
//! `proto/llamachat.proto`.  The full gRPC-web client bridge
//! (with streaming, bidirectional RPC) is planned for v0.4.0
//! once `tonic` gains WASM support or we implement a manual
//! fetch-based transport.
//!
//! Server-side gRPC-web support (tonic-web layer) is already
//! live on the Neoland gRPC server — browsers can call gRPC
//! endpoints via HTTP/1.1 + gRPC-web wire format.

// Generated protobuf code from `proto/llamachat.proto`
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/llamachat.rs"));
}
