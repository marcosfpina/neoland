{
  description = "Neoland - AI Agent Platform with Security-First Architecture";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    securellmBridge = {
      url = "git+file:../securellm-bridge";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-overlay.follows = "rust-overlay";
      inputs.flake-utils.follows = "flake-utils";
    };
    mlOpsApi = {
      url = "git+file:../ml-ops-api";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-overlay.follows = "rust-overlay";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      securellmBridge,
      mlOpsApi,
    }:
    let
      moduleInputs = {
        inherit securellmBridge mlOpsApi;
      };

      neolandModule = import ./modules/applications/neoland.nix;
      securellmBridgeApiModule = import ./modules/applications/securellm-bridge-api.nix {
        inputs = moduleInputs;
      };
      mlOpsApiModule = import ./modules/applications/ml-ops-api.nix {
        inputs = moduleInputs;
      };
      neolandLlmSuiteModule = import ./modules/applications/neoland-llm-suite.nix;
    in
    {
      nixosModules = {
        default =
          { ... }:
          {
            imports = [
              neolandModule
              securellmBridgeApiModule
              mlOpsApiModule
              neolandLlmSuiteModule
            ];
          };
        neoland = neolandModule;
        securellmBridgeApi = securellmBridgeApiModule;
        mlOpsApi = mlOpsApiModule;
        llmSuite = neolandLlmSuiteModule;
      };
    }
    // flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ];
        };

        mkNeolandCommand =
          {
            name,
            subcommand ? null,
          }:
          pkgs.writeShellApplication {
            inherit name;
            runtimeInputs = with pkgs; [
              coreutils
              gitMinimal
              sops
              rustToolchain
            ];
            text =
              ''
                project_root="''${NEOLAND_PROJECT_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
                script="$project_root/scripts/neoland-run.sh"

                if [ ! -f "$script" ]; then
                  echo "neoland wrapper could not find $script" >&2
                  exit 1
                fi
              ''
              + (
                if subcommand == null then
                  ''
                    exec ${pkgs.bash}/bin/bash "$script" "$@"
                  ''
                else
                  ''
                    exec ${pkgs.bash}/bin/bash "$script" ${pkgs.lib.escapeShellArg subcommand} "$@"
                  ''
              );
          };

        neolandCmd = mkNeolandCommand {
          name = "neoland";
        };

        neolandServerCmd = mkNeolandCommand {
          name = "neoland-server";
          subcommand = "server";
        };

        neolandClientCmd = mkNeolandCommand {
          name = "neoland-client";
          subcommand = "client";
        };

        neolandTestCmd = mkNeolandCommand {
          name = "neoland-test";
          subcommand = "test";
        };

        neolandDoctorCmd = mkNeolandCommand {
          name = "neoland-doctor";
          subcommand = "doctor";
        };

        neolandRestartCmd = mkNeolandCommand {
          name = "neoland-restart";
          subcommand = "restart";
        };

        nsrvCmd = mkNeolandCommand {
          name = "nsrv";
          subcommand = "server";
        };

        ncliCmd = mkNeolandCommand {
          name = "ncli";
          subcommand = "client";
        };

        neolandSecretsCmd = pkgs.writeShellApplication {
          name = "neoland-secrets";
          runtimeInputs = with pkgs; [
            coreutils
            gitMinimal
            sops
          ];
          text = ''
            project_root="''${NEOLAND_PROJECT_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
            sops_env_file="''${NEOLAND_SOPS_ENV_FILE:-$project_root/secrets/neoland.sops.env}"
            exec sops "$sops_env_file" "$@"
          '';
        };

        neolandCommandPackages = [
          neolandCmd
          neolandServerCmd
          neolandClientCmd
          neolandTestCmd
          neolandDoctorCmd
          neolandRestartCmd
          nsrvCmd
          ncliCmd
          neolandSecretsCmd
        ];

        frontendInstallCmd = pkgs.writeShellApplication {
          name = "frontend-install";
          runtimeInputs = with pkgs; [ nodejs_24 ];
          text = ''
            project_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
            frontend_dir="''${NEOLAND_FRONTEND_DIR:-$project_root/matrix/frontend}"
            cd "$frontend_dir"
            npm install "$@"
          '';
        };

        frontendDevCmd = pkgs.writeShellApplication {
          name = "frontend-dev";
          runtimeInputs = with pkgs; [ nodejs_24 ];
          text = ''
            project_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
            frontend_dir="''${NEOLAND_FRONTEND_DIR:-$project_root/matrix/frontend}"
            frontend_host="''${NEOLAND_FRONTEND_HOST:-127.0.0.1}"
            frontend_port="''${NEOLAND_FRONTEND_PORT:-3006}"
            control_plane_url="''${NEOLAND_CONTROL_PLANE_URL:-http://127.0.0.1:3001}"
            dspy_url="''${NEOLAND_DSPY_URL:-http://127.0.0.1:8001}"
            export NEXT_PUBLIC_BACKEND_URL="''${NEXT_PUBLIC_BACKEND_URL:-$control_plane_url}"
            export NEOLAND_CONTROL_PLANE_URL="$control_plane_url"
            export NEOLAND_DSPY_URL="$dspy_url"
            cd "$frontend_dir"
            npm run dev -- --hostname "$frontend_host" --port "$frontend_port" "$@"
          '';
        };

        frontendBuildCmd = pkgs.writeShellApplication {
          name = "frontend-build";
          runtimeInputs = with pkgs; [ nodejs_24 ];
          text = ''
            project_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
            frontend_dir="''${NEOLAND_FRONTEND_DIR:-$project_root/matrix/frontend}"
            control_plane_url="''${NEOLAND_CONTROL_PLANE_URL:-http://127.0.0.1:3001}"
            dspy_url="''${NEOLAND_DSPY_URL:-http://127.0.0.1:8001}"
            export NEXT_PUBLIC_BACKEND_URL="''${NEXT_PUBLIC_BACKEND_URL:-$control_plane_url}"
            export NEOLAND_CONTROL_PLANE_URL="$control_plane_url"
            export NEOLAND_DSPY_URL="$dspy_url"
            cd "$frontend_dir"
            npm run build "$@"
          '';
        };

        frontendStartCmd = pkgs.writeShellApplication {
          name = "frontend-start";
          runtimeInputs = with pkgs; [ nodejs_24 ];
          text = ''
            project_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
            frontend_dir="''${NEOLAND_FRONTEND_DIR:-$project_root/matrix/frontend}"
            frontend_host="''${NEOLAND_FRONTEND_HOST:-127.0.0.1}"
            frontend_port="''${NEOLAND_FRONTEND_PORT:-3006}"
            control_plane_url="''${NEOLAND_CONTROL_PLANE_URL:-http://127.0.0.1:3001}"
            dspy_url="''${NEOLAND_DSPY_URL:-http://127.0.0.1:8001}"
            export NEXT_PUBLIC_BACKEND_URL="''${NEXT_PUBLIC_BACKEND_URL:-$control_plane_url}"
            export NEOLAND_CONTROL_PLANE_URL="$control_plane_url"
            export NEOLAND_DSPY_URL="$dspy_url"
            cd "$frontend_dir"
            npm run start -- --hostname "$frontend_host" --port "$frontend_port" "$@"
          '';
        };

        frontendLintCmd = pkgs.writeShellApplication {
          name = "frontend-lint";
          runtimeInputs = with pkgs; [ nodejs_24 ];
          text = ''
            project_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
            frontend_dir="''${NEOLAND_FRONTEND_DIR:-$project_root/matrix/frontend}"
            cd "$frontend_dir"
            npm run lint "$@"
          '';
        };

        frontendCleanCmd = pkgs.writeShellApplication {
          name = "frontend-clean";
          runtimeInputs = with pkgs; [ coreutils ];
          text = ''
            project_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
            frontend_dir="''${NEOLAND_FRONTEND_DIR:-$project_root/matrix/frontend}"
            cd "$frontend_dir"
            rm -rf .next
            echo "cleared $frontend_dir/.next"
          '';
        };

        frontendHealthCmd = pkgs.writeShellApplication {
          name = "frontend-health";
          runtimeInputs = with pkgs; [ curl jq ];
          text = ''
            frontend_url="''${NEOLAND_FRONTEND_URL:-http://''${NEOLAND_FRONTEND_HOST:-127.0.0.1}:''${NEOLAND_FRONTEND_PORT:-3006}}"
            curl -fsS "$frontend_url/api/health" | jq . || {
              echo "frontend not reachable at $frontend_url"
              exit 1
            }
          '';
        };

        frontendStackCmd = pkgs.writeShellApplication {
          name = "frontend-stack";
          runtimeInputs = with pkgs; [ curl ];
          text = ''
            frontend_url="''${NEOLAND_FRONTEND_URL:-http://''${NEOLAND_FRONTEND_HOST:-127.0.0.1}:''${NEOLAND_FRONTEND_PORT:-3006}}"
            control_plane_url="''${NEOLAND_CONTROL_PLANE_URL:-http://127.0.0.1:3001}"
            dspy_url="''${NEOLAND_DSPY_URL:-http://127.0.0.1:8001}"

            echo "frontend:"
            if curl -fsS "$frontend_url/api/health" >/dev/null 2>&1; then
              echo "  up at $frontend_url"
            else
              echo "  down"
            fi

            echo "control-plane:"
            if curl -fsS "$control_plane_url/health" >/dev/null 2>&1; then
              echo "  up at $control_plane_url"
            else
              echo "  down"
            fi

            echo "dspy-pipeline:"
            if curl -fsS "$dspy_url/health" >/dev/null 2>&1; then
              echo "  up at $dspy_url"
            else
              echo "  down"
            fi
          '';
        };

        frontendCommandPackages = [
          frontendInstallCmd
          frontendDevCmd
          frontendBuildCmd
          frontendStartCmd
          frontendLintCmd
          frontendCleanCmd
          frontendHealthCmd
          frontendStackCmd
        ];

        neolandPackage = pkgs.rustPlatform.buildRustPackage {
          pname = "neoland";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
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
            cargo-audit   # supply-chain CVE scanning — `cargo audit` no CI e local
            openssl
            sops
            age
            bun
            curl
            jq
            nodejs_24
            python313
            poetry
          ] ++ frontendCommandPackages ++ neolandCommandPackages;

          # Garante que o protoc seja encontrado
          PROTOC = "${pkgs.protobuf}/bin/protoc";
          PKG_CONFIG_PATH = "$SHELL";
          shellHook = ''
            export NEOLAND_PROJECT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
            export NEOLAND_FRONTEND_DIR="$NEOLAND_PROJECT_ROOT/matrix/frontend"
            export NEOLAND_FRONTEND_HOST="''${NEOLAND_FRONTEND_HOST:-127.0.0.1}"
            export NEOLAND_FRONTEND_PORT="''${NEOLAND_FRONTEND_PORT:-3006}"
            export NEOLAND_FRONTEND_URL="''${NEOLAND_FRONTEND_URL:-http://$NEOLAND_FRONTEND_HOST:$NEOLAND_FRONTEND_PORT}"
            export NEOLAND_CONTROL_PLANE_URL="''${NEOLAND_CONTROL_PLANE_URL:-http://127.0.0.1:3001}"
            export NEOLAND_ML_API_URL="''${NEOLAND_ML_API_URL:-http://127.0.0.1:8080}"
            export ML_OPS_API_URL="''${ML_OPS_API_URL:-http://127.0.0.1:8083}"
            export LLAMACPP_URL="''${LLAMACPP_URL:-http://127.0.0.1:5001}"
            export VLLM_URL="''${VLLM_URL:-}"
            export NEOLAND_DSPY_URL="''${NEOLAND_DSPY_URL:-http://127.0.0.1:8001}"
            export NEXT_PUBLIC_BACKEND_URL="''${NEXT_PUBLIC_BACKEND_URL:-$NEOLAND_CONTROL_PLANE_URL}"

            frontend-install() { command frontend-install "$@"; }
            frontend-dev() { command frontend-dev "$@"; }
            frontend-build() { command frontend-build "$@"; }
            frontend-start() { command frontend-start "$@"; }
            frontend-lint() { command frontend-lint "$@"; }
            frontend-health() { command frontend-health "$@"; }
            frontend-clean() { command frontend-clean "$@"; }
            frontend-stack() { command frontend-stack "$@"; }
            nfdev() { frontend-dev "$@"; }
            nfbuild() { frontend-build "$@"; }
            nflint() { frontend-lint "$@"; }
            nfhealth() { frontend-health "$@"; }
            nfclean() { frontend-clean "$@"; }

            # Python agents via Poetry (padrão Cerebro)
            if [[ $- == *i* ]] && [ ! -f "$NEOLAND_PROJECT_ROOT/agents/.nix-installed-agents" ]; then
              echo "→ Syncing Neoland agents dependencies via Poetry..."
              (cd "$NEOLAND_PROJECT_ROOT/agents" && poetry install --no-interaction 2>/dev/null || true)
              touch "$NEOLAND_PROJECT_ROOT/agents/.nix-installed-agents"
              echo "✅ Neoland agents ready!"
            fi
            agents-start() { (cd "$NEOLAND_PROJECT_ROOT/agents" && poetry run uvicorn neoland_agents.app:app --reload --port 8001); }
            agents-test-contract() { (cd "$NEOLAND_PROJECT_ROOT/agents" && poetry run pytest tests/ -m contract -v "$@"); }
            agents-test-integration() { (cd "$NEOLAND_PROJECT_ROOT/agents" && poetry run pytest tests/ -m integration -v "$@"); }

            if [[ $- == *i* ]]; then
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
              echo "  neoland server           # Unified CLI wrapper in the dev shell"
              echo "  neoland-server           # Start gRPC + REST server"
              echo "  neoland-client           # Launch TUI client"
              echo "  neoland-test / doctor / restart"
              echo "  nsrv / ncli              # Short server/client shortcuts"
              echo ""
              echo "🌐 Frontend:"
              echo "  frontend-install         # Install matrix/apps/frontend dependencies"
              echo "  frontend-dev             # Run Next.js on $NEOLAND_FRONTEND_URL"
              echo "  frontend-build           # Production build for the Neoland web surface"
              echo "  frontend-start           # Start production Next.js server"
              echo "  frontend-lint            # Lint current frontend scope"
              echo "  frontend-clean           # Remove .next when Next dev cache gets stuck"
              echo "  frontend-health          # Query /api/health for the frontend"
              echo "  frontend-stack           # Check frontend + control plane + DSPy reachability"
              echo "  nfdev / nfbuild / nflint / nfhealth / nfclean"
              echo ""
              echo "🧠 LLM Runtime:"
              echo "  services.securellm-bridge-api via nixosModules.securellmBridgeApi"
              echo "  services.ml-ops-api via nixosModules.mlOpsApi"
              echo "  LLAMACPP_URL=$LLAMACPP_URL"
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
            fi
          '';
        };
      }
    );
}
