{
  description = "AI Assistant Backend - Observability & Ranking System";

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
        pythonEnv = pkgs.python313.withPackages (
          ps: with ps; [
            fastapi
            uvicorn
            prometheus-client
            elasticsearch
            psycopg2
            pandas
            scikit-learn
            mlflow
            pyyaml # Required for STF parser

            # Testing
            pytest
            pytest-cov
            pytest-asyncio
          ]
        );
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            pythonEnv
            prometheus
            grafana
            postgresql
            postgresqlPackages.timescaledb

            # Tools
            just
            jq
            git
            curl
          ];

          shellHook = ''
            # Colors for output
            RED='\033[0;31m'
            GREEN='\033[0;32m'
            YELLOW='\033[1;33m'
            BLUE='\033[0;34m'
            CYAN='\033[0;36m'
            NC='\033[0m' # No Color
            BOLD='\033[1m'

            # Banner
            echo -e "''${CYAN}╔════════════════════════════════════════════════════════════╗''${NC}"
            echo -e "''${CYAN}║''${NC}  ''${BOLD}🤖 AI Assistant Backend - Development Environment''${NC}     ''${CYAN}║''${NC}"
            echo -e "''${CYAN}╚════════════════════════════════════════════════════════════╝''${NC}"
            echo ""

            # Environment Info
            echo -e "''${BOLD}📦 Environment:''${NC}"
            echo -e "   ''${GREEN}✓''${NC} Python:     $(python3 --version | cut -d' ' -f2)"
            echo -e "   ''${GREEN}✓''${NC} Pytest:     $(pytest --version | head -n1 | cut -d' ' -f2)"
            echo -e "   ''${GREEN}✓''${NC} Prometheus: $(prometheus --version 2>&1 | head -n1 | cut -d' ' -f3)"
            echo -e "   ''${GREEN}✓''${NC} PostgreSQL: $(psql --version | cut -d' ' -f3)"
            echo ""

            # Exports
            export PYTHONPATH=$PWD/src:$PYTHONPATH
            export NIXPKGS_ALLOW_UNFREE=1
            export STF_PATH=''${STF_PATH:-/home/kernelcore/master/adr-ledger/.stf/neutron.stf}
            export DATABASE_URL=''${DATABASE_URL:-postgresql://localhost:5432/ai_agent_hub}
            export ALLOWED_ORIGINS=''${ALLOWED_ORIGINS:-http://localhost:3000,http://localhost:3001}

            # Shell Aliases
            alias dev='uvicorn src.ranking.main:app --reload --host 0.0.0.0 --port 8000'
            alias test='pytest tests/ --cov=src -v'
            alias test-quick='pytest tests/ -v --tb=short'
            alias test-watch='pytest-watch tests/ -- --cov=src'
            alias metrics='curl -s http://localhost:8000/metrics | grep ai_agent'
            alias health='curl -s http://localhost:8000/health | jq'
            alias prom='prometheus --config.file=prometheus.yml'
            alias format='black src/ tests/'
            alias lint='ruff check src/ tests/'
            alias db-init='psql -f src/db/timescale_init.sql $DATABASE_URL'

            # Helper Functions
            rank() {
              curl -X POST http://localhost:8000/rank \
                -H "Content-Type: application/json" \
                -d "{\"agent_id\":\"test\",\"decision_type\":\"$1\",\"context\":{},\"proposed_action\":\"test\"}" \
                | jq
            }

            # Available Commands
            echo -e "''${BOLD}⚡ Quick Commands:''${NC}"
            echo -e "   ''${CYAN}dev''${NC}          - Start development server (uvicorn)"
            echo -e "   ''${CYAN}test''${NC}         - Run all tests with coverage"
            echo -e "   ''${CYAN}test-quick''${NC}   - Run tests without coverage"
            echo -e "   ''${CYAN}metrics''${NC}      - View Prometheus metrics"
            echo -e "   ''${CYAN}health''${NC}       - Check backend health"
            echo -e "   ''${CYAN}prom''${NC}         - Start Prometheus"
            echo -e "   ''${CYAN}format''${NC}       - Format code with black"
            echo -e "   ''${CYAN}lint''${NC}         - Lint code with ruff"
            echo -e "   ''${CYAN}db-init''${NC}      - Initialize TimescaleDB schema"
            echo -e "   ''${CYAN}rank <type>''${NC}  - Test /rank endpoint"
            echo ""

            # Just commands (if available)
            if command -v just &> /dev/null; then
              echo -e "''${BOLD}🚀 Just Commands:''${NC}"
              echo -e "   ''${YELLOW}just --list''${NC}  - Show all available recipes"
              echo ""
            fi

            # Health Checks
            echo -e "''${BOLD}🏥 Health Checks:''${NC}"

            # Check STF file
            if [ -f "$STF_PATH" ]; then
              echo -e "   ''${GREEN}✓''${NC} STF Protocol: $STF_PATH"
            else
              echo -e "   ''${RED}✗''${NC} STF Protocol: Not found at $STF_PATH"
            fi

            # Check if backend is running
            if curl -s http://localhost:8000/health &> /dev/null; then
              echo -e "   ''${GREEN}✓''${NC} Backend: Running on :8000"
            else
              echo -e "   ''${YELLOW}○''${NC} Backend: Not running (use 'dev' to start)"
            fi

            # Check if Prometheus is running
            if curl -s http://localhost:9090/-/healthy &> /dev/null; then
              echo -e "   ''${GREEN}✓''${NC} Prometheus: Running on :9090"
            else
              echo -e "   ''${YELLOW}○''${NC} Prometheus: Not running (use 'prom' to start)"
            fi

            echo ""
            echo -e "''${BOLD}💡 Tip:''${NC} Run ''${CYAN}dev''${NC} to start the backend, then ''${CYAN}test''${NC} in another terminal"
            echo ""
          '';
        };
      }
    )
    // {
      nixosModules.default = import ./modules/ai-agent-hub.nix;
    };
}
