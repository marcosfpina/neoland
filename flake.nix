{
  description = "LlamaChat PoC Dev Environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ];
        };
      in
      {
        packages = {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "neoland";
            version = "0.1.0";

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            nativeBuildInputs = with pkgs; [
              pkg-config
              protobuf
            ];

            buildInputs = with pkgs; [
              openssl
            ];

            PROTOC = "${pkgs.protobuf}/bin/protoc";

            # Skip tests for now (may require additional setup)
            doCheck = false;

            meta = with pkgs.lib; {
              description = "LlamaChat PoC - GTK4 Chat Client with gRPC Backend";
              license = licenses.mit;
              maintainers = [ "kernelcore" ];
            };
          };
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            pkg-config
            protobuf # Necessário para gRPC/Prost
          ];

          buildInputs = with pkgs; [
            rustToolchain
            openssl
          ];

          # Garante que o protoc seja encontrado
          PROTOC = "${pkgs.protobuf}/bin/protoc";

          shellHook = ''
            echo ""
            echo "┌─────────────────────────────────────────────────────────────────┐"
            echo "│  🚀 Neoland Development Environment (v0.2.0)                   │"
            echo "│  LocalFirst AI Assistant | ML-Offload + SecureLLM             │"
            echo "└─────────────────────────────────────────────────────────────────┘"
            echo ""
            echo "📦 Quick Commands:"
            echo "  cargo check              # Validate compilation"
            echo "  cargo build --release    # Build optimized binary"
            echo "  cargo test               # Run test suite"
            echo ""
            echo "🔧 Development:"
            echo "  cargo run --bin neoland -- server  # Start gRPC + REST server"
            echo "  cargo run --bin neoland -- client  # Launch TUI client"
            echo ""
            echo "📊 Validation:"
            echo "  nix flake check          # Validate flake"
            echo "  cargo clippy             # Linter checks"
            echo ""
            echo "📚 Documentation:"
            echo "  docs/ADR.md              # Architecture Decision Records"
            echo "  README.md                # Project overview + roadmap"
            echo ""
            echo "💡 Tip: Set environment variables for SecureLLM:"
            echo "  export SECURELLM_PROVIDER=deepseek"
            echo "  export DEEPSEEK_API_KEY=sk-xxx"
            echo ""
          '';
        };
      }
    );
}
