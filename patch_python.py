content = open("flake.nix").read()

god_mode = """
        neolandGodModeCmd = pkgs.writeShellApplication {
          name = "neoland-up";
          runtimeInputs = [ pkgs.tmux ];
          text = ''
            echo "🚀 Launching Neoland Full Stack (God Mode)..."
            tmux new-session -d -s neoland-stack "neoland server"
            tmux split-window -h "cd agents && poetry run uvicorn neoland_agents.app:app --port 8001"
            tmux select-pane -t 0
            tmux split-window -v "sleep 2 && neoland client"
            tmux -2 attach-session -t neoland-stack
          '';
        };
"""

content = content.replace(
    "        neolandCommandPackages = [\n          neolandCmd",
    god_mode + "\n        neolandCommandPackages = [\n          neolandGodModeCmd\n          neolandCmd"
)

content = content.replace(
    "            poetry\n          ] ++ frontendCommandPackages ++ neolandCommandPackages;",
    "            poetry\n            tmux\n          ] ++ frontendCommandPackages ++ neolandCommandPackages;"
)

open("flake.nix", "w").write(content)
