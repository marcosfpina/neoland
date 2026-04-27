#!/usr/bin/env bash

# Ultra-discrete startup - single command
# Usage: ./quick-start.sh [port]

PORT=${1:-3006}
export DISCRETE_MODE=true

# Check dependencies
[ ! -f ".env.local" ] && echo "Missing .env.local" && exit 1
[ ! -f "package.json" ] && echo "Not a Node.js project" && exit 1

# Install if needed
[ ! -d "node_modules" ] && npm i -s

# Kill existing
pkill -f "next dev" 2>/dev/null || true

# Start
NODE_ENV=development PORT=$PORT npm run dev > /dev/null 2>&1 &
PID=$!

# Wait and test
sleep 3
if curl -s http://localhost:$PORT > /dev/null 2>&1; then
    echo "http://localhost:$PORT"
    echo $PID > .devhub.pid
else
    echo "Failed"
    kill $PID 2>/dev/null
    exit 1
fi
