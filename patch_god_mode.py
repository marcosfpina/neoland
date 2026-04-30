content = open("flake.nix").read()

import re

# We will remove neolandGodModeCmd and replace it with a native rust/shell runner

new_runner = """
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
            neoland client
            
            # When TUI exits, the trap won't catch it cleanly unless we kill manually
            echo "🛑 Shutting down backend services..."
            kill $SERVER_PID 2>/dev/null || true
            kill $AGENTS_PID 2>/dev/null || true
          '';
        };
"""

content = re.sub(
    r"neolandGodModeCmd = pkgs\.writeShellApplication \{.*?tmux -2 attach-session -t neoland-stack\n\s+'';\n\s+\};\n",
    new_runner,
    content,
    flags=re.DOTALL
)

content = content.replace(
    "            poetry\n            tmux\n          ] ++ frontendCommandPackages ++ neolandCommandPackages;",
    "            poetry\n          ] ++ frontendCommandPackages ++ neolandCommandPackages;"
)

open("flake.nix", "w").write(content)
