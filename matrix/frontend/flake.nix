{
  description = "Mission Control - Enterprise-grade NixOS System Manager for DevOps and LLM Laboratory";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };

        # Node.js environment for the Next.js app
        nodejs = pkgs.nodejs_24;

        # System monitoring dependencies
        systemDeps = with pkgs; [
          # Process monitoring
          procps
          lsof
          htop
          btop

          # GPU monitoring
          nvtopPackages.nvidia

          # Network monitoring
          iftop
          nethogs
          bandwhich

          # Disk monitoring
          iotop
          ncdu
          dust

          # System info
          neofetch
          fastfetch
          lshw
          dmidecode

          # Service management
          systemd

          # Database tools
          postgresql
          sqlite

          # AI/LLM
          ollama

          # Development
          git
          jq
          yq
          ripgrep
          fd
        ];

        # Python environment for system scripts
        pythonEnv = pkgs.python313.withPackages (
          ps: with ps; [
            psutil
            aiohttp
            websockets
            pydbus
            gpustat
            prometheus-client
            fastapi
            uvicorn
            httpx
          ]
        );

        # Build the Next.js application
        missionControl = pkgs.buildNpmPackage {
          pname = "mission-control";
          version = "2.0.0";
          src = ./.;

          # CRITICAL: Run 'nix run nixpkgs#prefetch-npm-deps package-lock.json' to get this hash
          npmDepsHash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

          # Enable standalone build in next.config.mjs before running this
          buildPhase = ''
            npm run build
          '';

          installPhase = ''
            mkdir -p $out/share/mission-control

            # Copy standalone build artifacts (requires output: "standalone" in next.config.mjs)
            cp -r .next/standalone/* $out/share/mission-control/

            # Copy static assets which are not included in the standalone folder by default
            mkdir -p $out/share/mission-control/.next
            cp -r .next/static $out/share/mission-control/.next/static
            cp -r public $out/share/mission-control/public

            mkdir -p $out/bin
            cat > $out/bin/mission-control << EOF
            #!/bin/sh
            export NODE_ENV=production
            # Bind to 0.0.0.0 to allow external access if configured, otherwise defaults to localhost
            export HOST=\''${HOST:-0.0.0.0}
            export PORT=\''${PORT:-3000}
            exec ${nodejs}/bin/node $out/share/mission-control/server.js
            EOF
            chmod +x $out/bin/mission-control
          '';

          meta = with pkgs.lib; {
            description = "Mission Control - NixOS System Manager";
            license = licenses.mit;
            platforms = platforms.linux;
          };
        };

      in
      {
        # Development shell
        devShells.default = pkgs.mkShell {
          name = "mission-control-dev";

          buildInputs = systemDeps ++ [
            nodejs
            pkgs.nodePackages.npm
            pkgs.nodePackages.pnpm
            pythonEnv

            # Development tools
            pkgs.just
            pkgs.typescript
            pkgs.nodePackages.typescript-language-server
            pkgs.nodePackages.eslint
            pkgs.nodePackages.prettier
          ];

          shellHook = ''
            # Colors
            RED='\033[0;31m'
            GREEN='\033[0;32m'
            YELLOW='\033[1;33m'
            BLUE='\033[0;34m'
            CYAN='\033[0;36m'
            NC='\033[0m'
            BOLD='\033[1m'

            echo -e "''${BLUE}╔══════════════════════════════════════════════════════════════╗''${NC}"
            echo -e "''${BLUE}║''${NC}  ''${BOLD}🎨 AI Assistant Frontend - Development Environment''${NC}       ''${BLUE}║''${NC}"
            echo -e "''${BLUE}╚══════════════════════════════════════════════════════════════╝''${NC}"
            echo ""

            # Environment Info
            echo -e "''${BOLD}📦 Environment:''${NC}"
            echo -e "   ''${GREEN}✓''${NC} Node.js:    $(node --version | cut -c2-)"
            echo -e "   ''${GREEN}✓''${NC} npm:        $(npm --version)"
            echo -e "   ''${GREEN}✓''${NC} TypeScript: $(tsc --version | cut -d' ' -f2)"
            echo -e "   ''${GREEN}✓''${NC} Python:     $(python --version | cut -d' ' -f2)"
            echo -e "   ''${GREEN}✓''${NC} Ollama:     $(ollama --version 2>/dev/null | cut -d' ' -f3 || echo 'not installed')"
            echo ""

            # Exports
            export NEXT_PUBLIC_BACKEND_URL=''${NEXT_PUBLIC_BACKEND_URL:-http://localhost:8000}
            export NODE_ENV=''${NODE_ENV:-development}
            export MISSION_CONTROL_ENV="development"
            export OLLAMA_HOST="http://localhost:11434"
            export PROC_PATH="/proc"
            export SYS_PATH="/sys"

            # Shell Aliases
            alias dev='npm run dev'
            alias build='npm run build'
            alias start='npm start'
            alias lint='npm run lint'
            alias type-check='tsc --noEmit'
            alias backend='curl -s http://localhost:8000/health | jq'
            alias metrics='curl -s http://localhost:8000/metrics | grep ai_agent'

            # Custom commands
            mc-status() {
              echo -e "''${BOLD}=== System Status ===''${NC}"
              echo -e "  CPU:    $(grep -c processor /proc/cpuinfo) cores"
              echo -e "  RAM:    $(free -h | awk '/Mem:/ {print $3 "/" $2}')"
              echo -e "  GPU:    $(nvidia-smi --query-gpu=name,memory.used,memory.total --format=csv,noheader 2>/dev/null || echo 'N/A')"
              echo -e "  Ollama: $(systemctl is-active ollama 2>/dev/null || echo 'unknown')"
            }



            # AI Assistant specific commands
            backend-test() {
              echo -e "''${BOLD}Testing backend connectivity...''${NC}"
              curl -X POST http://localhost:8000/rank \
                -H "Content-Type: application/json" \
                -d '{"agent_id":"test","decision_type":"test","context":{},"proposed_action":"test"}' \
                | jq
            }

            export -f mc-status
            export -f mc-ollama
            export -f backend-test

            # Available Commands
            echo -e "''${BOLD}⚡ Quick Commands:''${NC}"
            echo -e "   ''${CYAN}dev''${NC}          - Start development server (Next.js)"
            echo -e "   ''${CYAN}build''${NC}        - Build for production"
            echo -e "   ''${CYAN}lint''${NC}         - Run ESLint"
            echo -e "   ''${CYAN}type-check''${NC}   - Check TypeScript types"
            echo -e "   ''${CYAN}backend''${NC}      - Check backend health"
            echo -e "   ''${CYAN}metrics''${NC}      - View AI metrics"
            echo -e "   ''${CYAN}mc-ollama''${NC}    - Ollama management"
            echo -e "   ''${CYAN}mc-status''${NC}    - Check system status"
            echo -e "   ''${CYAN}backend-test''${NC} - Test backend /rank endpoint"
            echo ""

            # Just commands (if available)
            if command -v just &> /dev/null; then
              echo -e "''${BOLD}🚀 Just Commands:''${NC}"
              echo -e "   ''${YELLOW}just --list''${NC}  - Show all available recipes"
              echo ""
            fi

            # Health Checks
            echo -e "''${BOLD}🏥 Health Checks:''${NC}"

            # Check node_modules
            if [ -d "node_modules" ]; then
              echo -e "   ''${GREEN}✓''${NC} Dependencies: Installed"
            else
              echo -e "   ''${YELLOW}○''${NC} Dependencies: Not installed (run 'npm install')"
            fi

            # Check Ollama
            if systemctl is-active ollama &> /dev/null; then
              echo -e "   ''${GREEN}✓''${NC} Ollama: Running"
            else
              echo -e "   ''${YELLOW}○''${NC} Ollama: Not running (use 'mc-ollama start')"
            fi

            # Check if frontend is running
            if curl -s http://localhost:3000 &> /dev/null; then
              echo -e "   ''${GREEN}✓''${NC} Frontend: Running on :3000"
            else
              echo -e "   ''${YELLOW}○''${NC} Frontend: Not running (use 'dev' to start)"
            fi

            # Check if backend is running
            if curl -s http://localhost:8000/health &> /dev/null; then
              echo -e "   ''${GREEN}✓''${NC} Backend: Running on :8000"
            else
              echo -e "   ''${YELLOW}○''${NC} Backend: Not running"
            fi

            # Check llama
            # Simple curl check to user provided host
            if curl -s http://localhost:8081/health &> /dev/null; then
              echo -e "   ''${GREEN}✓''${NC} Llama Server: Connected (http://localhost:8081)"
            else
              echo -e "   ''${YELLOW}○''${NC} Llama Server: Not reachable at http://localhost:8081"
            fi

            echo ""
            echo -e "''${BOLD}💡 Tips:''${NC}"
            echo -e "   • Run ''${CYAN}dev''${NC} to start frontend on http://localhost:3000"
            echo -e "   • Backend should be running on :8000 for full functionality"
            echo -e "   • Use ''${CYAN}just --list''${NC} to see all automation tasks"
            echo ""

            # Ensure node_modules exists
            if [ ! -d "node_modules" ]; then
              echo -e "''${YELLOW}Installing dependencies...''${NC}"
              npm install
            fi
          '';

          # Environment variables for system integration
          MISSION_CONTROL_ENV = "development";
          OLLAMA_HOST = "http://localhost:11434";

          PROC_PATH = "/proc";
          SYS_PATH = "/sys";
        };

        # Packages
        packages = {
          default = missionControl;
          mission-control = missionControl;

          # System monitoring daemon
          mission-control-daemon = pkgs.writeShellScriptBin "mission-control-daemon" ''
            #!${pkgs.bash}/bin/bash
            export PATH="${pkgs.lib.makeBinPath systemDeps}:$PATH"
            exec ${pythonEnv}/bin/python ${./nix/system-daemon.py}
          '';
        };

        # NixOS module
        nixosModules.default =
          {
            config,
            lib,
            pkgs,
            ...
          }:
          with lib;
          let
            cfg = config.services.mission-control;
          in
          {
            options.services.mission-control = {
              enable = mkEnableOption "Mission Control system manager";

              port = mkOption {
                type = types.port;
                default = 3000;
                description = "Port to run Mission Control on";
              };

              host = mkOption {
                type = types.str;
                default = "127.0.0.1";
                description = "Host to bind to";
              };

              openFirewall = mkOption {
                type = types.bool;
                default = false;
                description = "Open firewall for Mission Control";
              };

              ollamaIntegration = mkOption {
                type = types.bool;
                default = true;
                description = "Enable Ollama integration";
              };

              gpuMonitoring = mkOption {
                type = types.bool;
                default = true;
                description = "Enable NVIDIA GPU monitoring";
              };

              systemMetrics = mkOption {
                type = types.bool;
                default = true;
                description = "Enable real-time system metrics collection";
              };

              dataDir = mkOption {
                type = types.path;
                default = "/var/lib/mission-control";
                description = "Data directory for Mission Control";
              };
            };

            config = mkIf cfg.enable {
              # Create system user
              users.users.mission-control = {
                isSystemUser = true;
                group = "mission-control";
                home = cfg.dataDir;
                createHome = true;
              };
              users.groups.mission-control = { };

              # Systemd service
              systemd.services.mission-control = {
                description = "Mission Control - NixOS System Manager";
                wantedBy = [ "multi-user.target" ];

                after = [ "network.target" ];

                environment = {
                  NODE_ENV = "production";
                  PORT = toString cfg.port;
                  HOST = cfg.host;
                  OLLAMA_HOST = "http://localhost:11434";
                  DATA_DIR = cfg.dataDir;
                  ENABLE_GPU_MONITORING = boolToString cfg.gpuMonitoring;
                  ENABLE_SYSTEM_METRICS = boolToString cfg.systemMetrics;
                };

                serviceConfig = {
                  Type = "simple";
                  User = "mission-control";
                  Group = "mission-control";
                  ExecStart = "${missionControl}/bin/mission-control";
                  Restart = "always";
                  RestartSec = "5";
                  WorkingDirectory = cfg.dataDir;

                  # Hardening
                  NoNewPrivileges = true;
                  ProtectSystem = "strict";
                  ProtectHome = true;
                  PrivateTmp = true;
                  ReadWritePaths = [ cfg.dataDir ];

                  # Allow reading system info
                  ReadOnlyPaths = [
                    "/proc"
                    "/sys"
                  ];

                  # Capabilities for system monitoring
                  AmbientCapabilities = [
                    "CAP_SYS_PTRACE"
                    "CAP_NET_ADMIN"
                  ];
                };
              };

              # System metrics collection daemon
              systemd.services.mission-control-metrics = mkIf cfg.systemMetrics {
                description = "Mission Control Metrics Collector";
                wantedBy = [ "multi-user.target" ];
                after = [ "mission-control.service" ];

                serviceConfig = {
                  Type = "simple";
                  User = "mission-control";
                  Group = "mission-control";
                  ExecStart = "${self.packages.${pkgs.system}.mission-control-daemon}/bin/mission-control-daemon";
                  Restart = "always";
                  RestartSec = "10";
                };
              };

              # Firewall
              networking.firewall.allowedTCPPorts = mkIf cfg.openFirewall [ cfg.port ];

              # Ensure Ollama is available if integration is enabled

            };
          };

        # Apps for direct execution
        apps.default = {
          type = "app";
          program = "${missionControl}/bin/mission-control";
        };
      }
    )
    // {
      # Overlay for use in other flakes
      overlays.default = final: prev: {
        mission-control = self.packages.${prev.system}.mission-control;
      };
    };
}
