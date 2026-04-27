#!/usr/bin/env bash

# Dev Assistant Hub - Discrete Local Development
# Lightweight startup for developer benchmarking

set -e

# Discrete mode - minimal output
DISCRETE_MODE=${DISCRETE_MODE:-true}
PORT=${PORT:-3006}
HOST=${HOST:-localhost}

# Colors (only if not discrete)
if [ "$DISCRETE_MODE" != "true" ]; then
    GREEN='\033[0;32m'
    BLUE='\033[0;34m'
    NC='\033[0m'
else
    GREEN=''
    BLUE=''
    NC=''
fi

log() {
    if [ "$DISCRETE_MODE" != "true" ]; then
        echo -e "${BLUE}[DevHub]${NC} $1"
    fi
}

success() {
    if [ "$DISCRETE_MODE" != "true" ]; then
        echo -e "${GREEN}[Ready]${NC} $1"
    else
        echo "$1"
    fi
}

# Check if .env.local exists
if [ ! -f ".env.local" ]; then
    echo "Error: .env.local not found. Please create it first."
    exit 1
fi

# Load environment variables
export $(grep -v '^#' .env.local | xargs)

# Check if node_modules exists
if [ ! -d "node_modules" ]; then
    log "Installing dependencies..."
    npm install --silent
fi

# Check if .next exists and is recent
if [ ! -d ".next" ] || [ ".next" -ot "package.json" ]; then
    log "Building application..."
    npm run build --silent
fi

# Kill any existing process on the port
if lsof -Pi :$PORT -sTCP:LISTEN -t >/dev/null 2>&1; then
    log "Stopping existing process on port $PORT..."
    kill -9 $(lsof -Pi :$PORT -sTCP:LISTEN -t) 2>/dev/null || true
    sleep 1
fi

# Start the application in background
log "Starting Dev Assistant Hub..."

# Start the application and save PID
export NODE_ENV=development
export PORT=$PORT
export HOST=$HOST
npm run dev > /dev/null 2>&1 &
APP_PID=$!
echo $APP_PID > .devhub.pid

# Wait for the application to be ready
log "Waiting for application..."
for i in {1..30}; do
    if curl -s http://$HOST:$PORT/api/health > /dev/null 2>&1; then
        break
    fi
    sleep 1
done

# Check if app is running
if curl -s http://$HOST:$PORT/api/health > /dev/null 2>&1; then
    success "http://$HOST:$PORT"

    # Discrete mode - just show the URL
    if [ "$DISCRETE_MODE" = "true" ]; then
        echo "http://$HOST:$PORT"
    fi
else
    echo "Failed to start application"
    exit 1
fi
