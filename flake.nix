{
  description = "Neoland - AI Agent Platform with Security-First Architecture";

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

        neolandPackage = pkgs.rustPlatform.buildRustPackage {
          pname = "neoland";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
            protobuf
            maturin
          ];

          buildInputs = with pkgs; [
            openssl
          ];

          PROTOC = "${pkgs.protobuf}/bin/protoc";

          # Enable tests (Phase 0: Foundation)
          doCheck = true;

          meta = with pkgs.lib; {
            description = "Neoland - Terminal AI Agent with Enterprise Security";
            license = licenses.mit;
            maintainers = [ "kernelcore" ];
          };
        };
      in
      {
        packages = {
          default = neolandPackage;
          neoland = neolandPackage;
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            pkg-config
            protobuf # Necessário para gRPC/Prost
          ];

          buildInputs = with pkgs; [
            rustToolchain
            openssl
            sops
            age
            python313
            python313Packages.pip
          ];

          # Garante que o protoc seja encontrado
          PROTOC = "${pkgs.protobuf}/bin/protoc";
          PKG_CONFIG_PATH = "$SHELL";
          shellHook = ''
            export NEOLAND_PROJECT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"

            neoland-server() { "$NEOLAND_PROJECT_ROOT/scripts/neoland-run.sh" server "$@"; }
            neoland-client() { "$NEOLAND_PROJECT_ROOT/scripts/neoland-run.sh" client "$@"; }
            nsrv() { "$NEOLAND_PROJECT_ROOT/scripts/neoland-run.sh" server "$@"; }
            ncli() { "$NEOLAND_PROJECT_ROOT/scripts/neoland-run.sh" client "$@"; }
            neoland-secrets() { sops "$NEOLAND_PROJECT_ROOT/secrets/neoland.sops.env"; }

            # Python agents venv — recriar se não existir ou se a versão for diferente de 3.13
            _venv_py_ver=$("$NEOLAND_PROJECT_ROOT/agents/.venv/bin/python" --version 2>/dev/null | grep -o "3\.[0-9]*" | head -1)
            if [ ! -d "$NEOLAND_PROJECT_ROOT/agents/.venv" ] || [ "$_venv_py_ver" != "3.13" ]; then
              echo "→ Criando venv Python 3.13 para agents/..."
              rm -rf "$NEOLAND_PROJECT_ROOT/agents/.venv"
              python3.13 -m venv "$NEOLAND_PROJECT_ROOT/agents/.venv"
              "$NEOLAND_PROJECT_ROOT/agents/.venv/bin/pip" install -e "$NEOLAND_PROJECT_ROOT/agents/[dev]" -q
            fi
            agents-start() { uvicorn neoland_agents.app:app --reload --port 8001 --app-dir "$NEOLAND_PROJECT_ROOT/agents"; }
            agents-test-contract() { (cd "$NEOLAND_PROJECT_ROOT/agents" && "$NEOLAND_PROJECT_ROOT/agents/.venv/bin/pytest" tests/ -m contract -v "$@"); }
            agents-test-integration() { (cd "$NEOLAND_PROJECT_ROOT/agents" && "$NEOLAND_PROJECT_ROOT/agents/.venv/bin/pytest" tests/ -m integration -v "$@"); }
            export PATH="$NEOLAND_PROJECT_ROOT/agents/.venv/bin:$PATH"

            echo ""
            echo "┌─────────────────────────────────────────────────────────────────┐"
            echo "│  🚀 Neoland Development Environment (v0.1.0)                   │"
            echo "│  AI Agent Platform | Security-First Architecture             │"
            echo "└─────────────────────────────────────────────────────────────────┘"
            echo ""
            echo "📦 Quick Commands:"
            echo "  cargo check              # Validate compilation"
            echo "  cargo build --release    # Build optimized binary"
            echo "  cargo test               # Run test suite"
            echo ""
            echo "🔧 Development:"
            echo "  neoland-server           # Start gRPC + REST server"
            echo "  neoland-client           # Launch TUI client"
            echo "  nsrv / ncli              # Short functions for server/client"
            echo ""
            echo "🔐 Secrets:"
            echo "  neoland-secrets          # Edit secrets/neoland.sops.env with SOPS"
            echo "  NEOLAND_SOPS_ENV_FILE    # Override the default encrypted dotenv path"
            echo ""
            echo "📊 Validation:"
            echo "  nix flake check          # Validate flake"
            echo "  cargo clippy             # Linter checks"
            echo ""
            echo "📚 Documentation:"
            echo "  docs/ADR.md              # Architecture Decision Records"
            echo "  README.md                # Project overview + roadmap"
            echo ""
            echo "🤖 Agents (DSPy pipeline):"
            echo "  agents-start             # Start DSPy pipeline (:8001)"
            echo "  agents-test-contract     # Run schema tests (sem LLM)"
            echo "  agents-test-integration  # Run integration tests (requer LLM_API_KEY)"
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
