#!/usr/bin/env bash

# Stop script for Dev Assistant Hub
if [ -f ".devhub.pid" ]; then
    PID=$(cat .devhub.pid)
    if kill -0 $PID 2>/dev/null; then
        kill $PID
        echo "Stopped"
    else
        echo "Process not running"
    fi
    rm -f .devhub.pid
else
    # Fallback - kill any Next.js dev processes
    pkill -f "next dev" 2>/dev/null && echo "Stopped (fallback)" || echo "Not running"
fi
