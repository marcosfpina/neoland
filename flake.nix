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
    aiAgentOs = {
      url = "git+ssh://git@github.com/VoidNxSEC/ai-agent-os";
      flake = false;
    };
    phantom = {
      url = "git+ssh://git@github.com/VoidNxSEC/phantom";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    owasaka = {
      url = "git+ssh://git@github.com/VoidNxSEC/O.W.A.S.A.K.A.";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    spectre = {
      url = "git+ssh://git@github.com/VoidNxSEC/spectre";
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
        inherit (inputs) securellmBridge mlOpsApi spectre owasaka;
      };

      neolandModule = import ./modules/applications/neoland.nix;
      securellmBridgeApiModule = import ./modules/applications/securellm-bridge-api.nix {
        inputs = moduleInputs;
      };
      mlOpsApiModule = import ./modules/applications/ml-ops-api.nix {
        inputs = moduleInputs;
      };
      neolandLlmSuiteModule    = import ./modules/applications/neoland-llm-suite.nix;
      spectreEventBusModule    = import ./modules/applications/spectre-event-bus.nix {
        inputs = moduleInputs;
      };
      owasakaModule            = import ./modules/applications/owasaka.nix {
        inputs = moduleInputs;
      };
      neolandStackModule       = import ./modules/applications/neoland-stack.nix {
        inputs = moduleInputs;
      };
    in
    {
      nixosModules = {
        # ── Individual service modules ────────────────────────────────
        neoland          = neolandModule;
        securellmBridgeApi = securellmBridgeApiModule;
        mlOpsApi         = mlOpsApiModule;
        llmSuite         = neolandLlmSuiteModule;
        spectreEventBus  = spectreEventBusModule;
        owasaka          = owasakaModule;

        # ── Composite: full stack in one import ───────────────────────
        stack            = neolandStackModule;

        # ── Default: all individual modules (no stack opinions) ───────
        default =
          { ... }:
          {
            imports = [
              neolandModule
              securellmBridgeApiModule
              mlOpsApiModule
              neolandLlmSuiteModule
              spectreEventBusModule
              owasakaModule
            ];
          };
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
          targets = [ "wasm32-unknown-unknown" ];
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
              uv
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

        neolandWebPackage = pkgs.stdenv.mkDerivation {
          pname = "neoland-web";
          version = "0.2.0";
          src = ./.;
          nativeBuildInputs = with pkgs; [ trunk rustToolchain pkg-config openssl ];
          buildPhase = ''
            cd web
            export HOME="$TMPDIR"
            export CARGO_HOME="$TMPDIR/.cargo"
            mkdir -p "$CARGO_HOME"
            trunk build --release
          '';
          installPhase = ''
            mkdir -p $out
            cp -r web/dist/* $out/
          '';
          meta = with pkgs.lib; {
            description = "Neoland Web Console — Leptos WASM SPA";
            license = licenses.mit;
            maintainers = [ "kernelcore" ];
          };
        };

        ibmPlexMono = pkgs.callPackage ./nix/ibm-plex-mono.nix { };

        neolandDesktopPackage = pkgs.stdenv.mkDerivation {
          pname = "neoland-desktop";
          version = "0.4.0";
          src = ./desktop;
          nativeBuildInputs = with pkgs; [
            rustToolchain
            pkg-config
            trunk
            openssl
            glib
            gtk3
            webkitgtk_4_1
            libsoup_3
          ];
          buildInputs = with pkgs; [
            openssl
            glib
            gtk3
            webkitgtk_4_1
            libsoup_3
          ];
          buildPhase = ''
            export HOME="$TMPDIR"
            export CARGO_HOME="$TMPDIR/.cargo"
            mkdir -p "$CARGO_HOME"
            cd ../web
            trunk build --release
            cd ../desktop/src-tauri
            cargo check --release
          '';
          installPhase = ''
            mkdir -p $out
            echo "Neoland Desktop v0.4.0 — build validated" > $out/README.txt
            echo "Run cd desktop && cargo tauri build for native binaries." >> $out/README.txt
          '';
          meta = with pkgs.lib; {
            description = "Neoland Desktop — Tauri-native AI console";
            license = licenses.mit;
            maintainers = [ "kernelcore" ];
            platforms = platforms.linux ++ platforms.darwin;
          };
        };
      in
      {
        packages = {
          default = neolandPackage;
          neoland = neolandPackage;
          neoland-web = neolandWebPackage;
          ibm-plex-mono = ibmPlexMono;
          neoland-desktop = neolandDesktopPackage;
        };

        formatter = pkgs.nixfmt-tree;

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
              cargo-watch # `just watch` — hot-reload check on save
              openssl
              sops
              age
              bun
              curl
              uv
              trunk
              wasm-bindgen-cli
              wasm-pack
              chromedriver
              chromium
              cargo-wasi
              jq
              rustup
              python313
              poetry
              # Required for Rust-based Python extensions (tokenizers, dspy via litellm)
              stdenv.cc.cc.lib
              glib
              gtk3
              webkitgtk_4_1
              libsoup_3
              cairo
            ]
            ++ neolandCommandPackages;

          LD_LIBRARY_PATH = "${pkgs.stdenv.cc.cc.lib}/lib";

          # Garante que o protoc seja encontrado
          PROTOC = "${pkgs.protobuf}/bin/protoc";
          # PKG_CONFIG_PATH is auto-populated by Nix from nativeBuildInputs

          # Force wasm-pack to use the Nix-provided chromedriver (statically linked)
          # instead of downloading a dynamically-linked binary that fails on NixOS.
          CHROMEDRIVER = "${pkgs.chromedriver}/bin/chromedriver";

          shellHook = ''
            export NEOLAND_PROJECT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"

            # Spectre event bus — NATS already running via Docker (spectre-nats:4222).
            # Override to point at a different bus if needed.
            export NEOLAND_NATS_URL="''${NEOLAND_NATS_URL:-nats://127.0.0.1:4222}"

            # Spectre observability endpoints (for `just doctor` / tracing)
            export SPECTRE_JAEGER_URL="''${SPECTRE_JAEGER_URL:-http://127.0.0.1:16686}"
            export OTEL_EXPORTER_OTLP_ENDPOINT="''${OTEL_EXPORTER_OTLP_ENDPOINT:-http://127.0.0.1:4317}"

            # Make ai-agent-os available at the expected relative path for cargo path overrides.
            # .cargo/config.toml patches hyprland-ipc via ../ai-agent-os/crates/hyprland-ipc.
            _ai_agent_os_link="$(dirname "$NEOLAND_PROJECT_ROOT")/ai-agent-os"
            if [ "$(readlink "$_ai_agent_os_link" 2>/dev/null)" != "${inputs.aiAgentOs}" ]; then
              ln -sfn ${inputs.aiAgentOs} "$_ai_agent_os_link"
            fi
            unset _ai_agent_os_link

            # Local dev database (no credentials — peer auth via unix socket).
            export DATABASE_URL="''${DATABASE_URL:-postgresql:///neoland?host=/run/postgresql}"

            export NEOLAND_CONTROL_PLANE_URL="''${NEOLAND_CONTROL_PLANE_URL:-http://127.0.0.1:3001}"
            export NEOLAND_GATEWAY_URL="''${NEOLAND_GATEWAY_URL:-''${NEOLAND_ML_API_URL:-http://127.0.0.1:8081}}"
            export NEOLAND_ML_API_URL="''${NEOLAND_ML_API_URL:-$NEOLAND_GATEWAY_URL}"
            export ML_OPS_API_URL="''${ML_OPS_API_URL:-http://127.0.0.1:8083}"
            export LLAMACPP_URL="''${LLAMACPP_URL:-http://127.0.0.1:8080}"
            export VLLM_URL="''${VLLM_URL:-}"
            export NEOLAND_DSPY_URL="''${NEOLAND_DSPY_URL:-http://127.0.0.1:8001}"
            export NEOLAND_CHECKPOINT_DIR="''${NEOLAND_CHECKPOINT_DIR:-$HOME/.local/share/neoland/checkpoints/adr}"

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

            # Short aliases: nd = doctor, nt = test (no SOPS)
            alias nd="neoland-doctor"
            alias nt="cargo test --lib"

            if [[ $- == *i* ]]; then
              echo ""
              echo "  ╭─────────────────────────────────────────────────────────────╮"
              echo "  │  Neoland dev shell — v0.4.0-beta                           │"
              echo "  ╰─────────────────────────────────────────────────────────────╯"
              echo ""
              echo "  Run                    Alias"
              echo "  ─────────────────────  ──────"
              echo "  neoland client  / TUI  ncli"
              echo "  neoland server         nsrv"
              echo "  neoland doctor         nd"
              echo "  cargo test --lib       nt"
              echo ""
              echo "  just dev               # servidor + TUI (Ctrl+C mata tudo)"
              echo "  just dev-web           # servidor + Web Console :8080"
              echo "  just serve             # só servidor"
              echo "  just ci                # check → fmt → clippy → test"
              echo "  just test-web-wasm     # WASM integration tests"
              echo "  just gen-certs         # gerar certificados mTLS"
              echo ""
              echo "  🌐 Local URLs (com servidor rodando):"
              echo "     Web Console  http://localhost:3001"
              echo "     Landing      http://localhost:3001/landing.html"
              echo "     API Docs     http://localhost:3001/swagger-ui"
              echo "     OpenAPI JSON http://localhost:3001/openapi.json"
              echo "     Metrics      http://localhost:3001/metrics"
              echo "     Health       http://localhost:3001/health"
              echo ""
              echo "  TUI: /why  /steer  /search  /name  /help  /theme"
              echo ""
              echo "  Agents:"
              echo "    agents-start           # DSPy :8001 (uvicorn --reload)"
              echo "    agents-test-contract   # pytest -m contract"
              echo ""
              echo "  Spectre stack (Docker):"
              echo "    NATS    nats://127.0.0.1:4222  (event bus)"
              echo "    Jaeger  http://127.0.0.1:16686  (tracing)"
              echo "    Grafana http://127.0.0.1:3005   (metrics)"
              echo ""
              echo "  NixOS modules: neoland · spectreEventBus · owasaka · stack"
              echo "  Run 'just' to list all recipes."
              echo ""
            fi
          '';
        };
      }
    );
}
