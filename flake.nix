{
  description = "Neoland - AI Agent Platform with Security-First Architecture";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    securellmBridge = {
      url = "git+ssh://git@github.com/VoidNxSEC/securellm-bridge";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    mlOpsApi = {
      url = "git+ssh://git@github.com/VoidNxSEC/ml-ops-api";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }@inputs:
    let
      moduleInputs = {
        inherit (inputs) securellmBridge mlOpsApi;
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
              socat
              rustToolchain
            ];
            text = ''
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

        neolandGodModeCmd = pkgs.writeShellApplication {
          name = "neoland-up";
          text = ''
            echo "🚀 Launching Neoland Full Stack (God Mode)..."

            # Trap SIGINT to kill background processes gracefully
            trap 'echo "🛑 Shutting down Neoland..."; kill $(jobs -p) 2>/dev/null; exit' SIGINT SIGTERM

            # Start Server in background
            echo "📡 Starting Control Plane (Port 3001/50051)..."
            neoland server &
            SERVER_PID=$!

            # Start DSPy Python Agents in background
            echo "🧠 Starting DSPy Agents (Port 8001)..."
            (cd agents && poetry run uvicorn neoland_agents.app:app --port 8001) > /dev/null 2>&1 &
            AGENTS_PID=$!

            # Wait for ports to bind before starting the TUI
            echo "⏳ Waiting for services to become healthy..."
            sleep 2

            # Launch TUI in the foreground (takes over the screen)
            NEOLAND_API_KEY="neoland_admin_53352f54e22da11f63edc17380c7bb48aef08811c0872caa" neoland client

            # When TUI exits, the trap won't catch it cleanly unless we kill manually
            echo "🛑 Shutting down backend services..."
            kill $SERVER_PID 2>/dev/null || true
            kill $AGENTS_PID 2>/dev/null || true
          '';
        };

        neolandCommandPackages = [
          neolandGodModeCmd
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
          runtimeInputs = with pkgs; [
            curl
            jq
          ];
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
            just
            direnv
          ];

          buildInputs =
            with pkgs;
            [
              rustToolchain
              cargo-audit # supply-chain CVE scanning — `cargo audit` no CI e local
              openssl
              sops
              age
              bun
              curl
              jq
              nodejs_24
              python313
              poetry
            ]
            ++ frontendCommandPackages
            ++ neolandCommandPackages;

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
              echo "│  🚀  Neoland Dev Shell (v0.1.0)                               │"
              echo "│  just → list all commands                                      │"
              echo "└─────────────────────────────────────────────────────────────────┘"
              echo ""
              echo -e "  \033[1mjust\033[0m  check     cargo check --lib (fast)"
              echo -e "  \033[1mjust\033[0m  clippy    cargo clippy --all-targets -- -D warnings"
              echo -e "  \033[1mjust\033[0m  test      cargo test --lib"
              echo -e "  \033[1mjust\033[0m  build     cargo build --release"
              echo -e "  \033[1mjust\033[0m  server    cargo run -- server"
              echo -e "  \033[1mjust\033[0m  client    cargo run -- client"
              echo -e "  \033[1mjust\033[0m  doctor    cargo run -- doctor --json"
              echo -e "  \033[1mjust\033[0m  setup-all setup hooks + direnv"
              echo -e "  \033[1mjust\033[0m  validate  production readiness check"
              echo "  Run \`just\` for the full list"
              echo ""
              echo "🔧 Legacy shortcuts also available:"
              echo "  neoland-server / neoland-client / neoland-test / neoland-doctor"
              echo "  nsrv / ncli  # short server/client"
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
