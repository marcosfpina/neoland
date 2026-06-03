{
  description = "PHANTOM UX Spec Environment - Terminal Agent UX Orchestration";

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
        pkgs = nixpkgs.legacyPackages.${system};

        # MCP server for UX specs
        ux-spec-mcp = pkgs.buildNpmPackage {
          pname = "ux-spec-mcp-server";
          version = "1.0.0";

          src = ./ux-spec-server;

          npmDepsHash = pkgs.lib.fakeSha256; # Update após primeiro build

          meta = {
            description = "MCP server for delivering UX specifications to AI agents";
            license = pkgs.lib.licenses.mit;
          };
        };

        # Script para gerar prompts UX otimizados
        ux-prompt-gen = pkgs.writeShellScriptBin "ux-prompt" ''
          #!${pkgs.bash}/bin/bash
          # UX Prompt Generator para agentes de terminal

          SPEC_DIR="''${UX_SPECS_DIR:-$HOME/.config/phantom/ux-specs}"
          SPEC_NAME="$1"
          COMPONENT="$2"
          FORMAT="''${3:-prompt}"

          if [ -z "$SPEC_NAME" ]; then
            echo "Usage: ux-prompt <spec-name> [component-type] [format]"
            echo "Available specs:"
            ls -1 "$SPEC_DIR"/*.yml | xargs -n1 basename -s .yml
            exit 1
          fi

          ${pkgs.yq-go}/bin/yq eval '.' "$SPEC_DIR/$SPEC_NAME.yml" | \
            ${pkgs.jq}/bin/jq -r --arg comp "$COMPONENT" '
              "# UX SPECIFICATION FOR AGENT\n" +
              "\n## Design Philosophy" +
              "\nTONE: \(.philosophy.tone)" +
              "\nPURPOSE: \(.philosophy.purpose)" +
              "\nUNFORGETTABLE: \(.philosophy.memorable_element)" +
              "\n\n## Typography" +
              "\n\(.typography | to_entries[] | "\(.key): \(.value)")" +
              "\n\n## Color System" +
              "\n\(.color_system | to_entries[] | "\(.key): \(.value)")" +
              "\n\n## Animation" +
              "\n\(.animation | to_entries[] | "\(.key): \(.value)")" +
              "\n\n## Constraints" +
              "\n\(.constraints | to_entries[] | "- \(.key): \(.value)")" +
              (if $comp != "" then
                "\n\n## COMPONENT: \($comp)\nGenerate production-ready React + Tailwind code for this component."
              else "" end) +
              "\n\n---\nIMPORTANT: Follow this specification EXACTLY. No generic AI aesthetics."
            '
        '';

        # CLI tool integrado com PHANTOM
        phantom-ux = pkgs.writeShellScriptBin "phantom-ux" ''
          #!${pkgs.bash}/bin/bash
          # PHANTOM UX - Frontend specification orchestrator

          set -euo pipefail

          CMD="$1"
          shift

          case "$CMD" in
            spec)
              # Mostra spec completa
              ux-prompt "$@"
              ;;

            gen)
              # Gera componente via Claude API
              SPEC="$1"
              COMPONENT="$2"

              PROMPT=$(ux-prompt "$SPEC" "$COMPONENT")

              # Chama Claude via API (assumindo que ANTHROPIC_API_KEY está setada)
              ${pkgs.curl}/bin/curl https://api.anthropic.com/v1/messages \
                -H "x-api-key: $ANTHROPIC_API_KEY" \
                -H "anthropic-version: 2023-06-01" \
                -H "content-type: application/json" \
                -d "{
                  \"model\": \"claude-sonnet-4-20250514\",
                  \"max_tokens\": 4096,
                  \"messages\": [{
                    \"role\": \"user\",
                    \"content\": $(echo "$PROMPT" | ${pkgs.jq}/bin/jq -Rs .)
                  }]
                }" | ${pkgs.jq}/bin/jq -r '.content[0].text' > "./generated-component.tsx"

              echo "Component generated: ./generated-component.tsx"
              ;;

            validate)
              # Valida se código gerado segue a spec
              SPEC="$1"
              FILE="$2"

              # Extrai regras da spec e valida contra o arquivo
              echo "Validating $FILE against $SPEC..."

              # Check 1: Fonts corretas
              FONT=$(ux-prompt "$SPEC" | grep "display_font:" | cut -d: -f2 | xargs)
              if ! grep -q "$FONT" "$FILE"; then
                echo "❌ Wrong font used (expected: $FONT)"
              else
                echo "✅ Font check passed"
              fi

              # Check 2: Color system
              PRIMARY_COLOR=$(ux-prompt "$SPEC" | grep "primary:" | cut -d: -f2 | xargs)
              if ! grep -q "$PRIMARY_COLOR" "$FILE"; then
                echo "⚠️  Primary color not found in code"
              else
                echo "✅ Color system check passed"
              fi
              ;;

            list)
              # Lista specs disponíveis
              ls -1 ~/.config/phantom/ux-specs/*.yml | xargs -n1 basename -s .yml
              ;;

            create)
              # Cria nova spec interativamente
              SPEC_NAME="$1"
              ${pkgs.vim}/bin/vim ~/.config/phantom/ux-specs/"$SPEC_NAME".yml
              ;;

            *)
              cat <<EOF
          PHANTOM UX - Frontend Specification Orchestrator

          Commands:
            spec <name> [component]     - Show UX specification
            gen <spec> <component>      - Generate component via Claude
            validate <spec> <file>      - Validate code against spec
            list                        - List available specs
            create <name>               - Create new UX spec

          Examples:
            phantom-ux spec phantom-security-dashboard
            phantom-ux gen phantom-security-dashboard ThreatCard
            phantom-ux validate phantom-security-dashboard ./ThreatCard.tsx
          EOF
              ;;
          esac
        '';

      in
      {
        packages = {
          default = phantom-ux;
          ux-spec-mcp = ux-spec-mcp;
          ux-prompt-gen = ux-prompt-gen;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Core tools
            nodejs_24
            typescript

            # MCP server
            ux-spec-mcp

            # CLI tools
            phantom-ux
            ux-prompt-gen

            # Utilities
            jq
            yq-go
            curl

            # Frontend dev
            bun # Mais rápido que npm
          ];

          shellHook = ''
            echo "🎨 PHANTOM UX Environment Ready"
            echo ""
            echo "Available commands:"
            echo "  phantom-ux - Main UX orchestrator CLI"
            echo "  ux-prompt  - Generate UX prompts for agents"
            echo ""
            echo "MCP Server: ux-spec-mcp-server"
            echo ""

            # Setup UX specs directory
            export UX_SPECS_DIR="$HOME/.config/phantom/ux-specs"
            mkdir -p "$UX_SPECS_DIR"

            # Auto-start MCP server se PHANTOM_MCP_AUTO=1
            if [ "''${PHANTOM_MCP_AUTO:-0}" = "1" ]; then
              echo "Starting UX Spec MCP server..."
              ux-spec-mcp-server &
            fi
          '';
        };

        # NixOS module para integração sistema-wide
        nixosModules.phantom-ux =
          {
            config,
            lib,
            pkgs,
            ...
          }:
          {
            options.services.phantom-ux = {
              enable = lib.mkEnableOption "PHANTOM UX Specification Service";

              specsDir = lib.mkOption {
                type = lib.types.path;
                default = "/var/lib/phantom/ux-specs";
                description = "Directory for UX specifications";
              };

              mcpPort = lib.mkOption {
                type = lib.types.port;
                default = 3000;
                description = "Port for MCP server";
              };
            };

            config = lib.mkIf config.services.phantom-ux.enable {
              systemd.services.phantom-ux-mcp = {
                description = "PHANTOM UX Specification MCP Server";
                wantedBy = [ "multi-user.target" ];

                environment = {
                  UX_SPECS_DIR = config.services.phantom-ux.specsDir;
                  PORT = toString config.services.phantom-ux.mcpPort;
                };

                serviceConfig = {
                  ExecStart = "${ux-spec-mcp}/bin/ux-spec-mcp-server";
                  Restart = "always";
                  User = "phantom";
                  Group = "phantom";
                };
              };

              users.users.phantom = {
                isSystemUser = true;
                group = "phantom";
                home = config.services.phantom-ux.specsDir;
                createHome = true;
              };

              users.groups.phantom = { };
            };
          };
      }
    );
}
