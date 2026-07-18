//! Build script for the Neoland Web Console.
//!
//! Compiles protobuf message types (not gRPC stubs — tonic requires tokio,
//! which is not WASM-compatible). Full gRPC-web client bridge planned for v0.1.0.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    prost_build::compile_protos(&["../proto/llamachat.proto"], &["../proto/"])?;
    Ok(())
}
