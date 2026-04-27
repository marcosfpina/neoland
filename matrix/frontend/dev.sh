#!/usr/bin/env bash

# Developer-friendly startup with benchmarking
# Measures startup time and resource usage

START_TIME=$(date +%s%N)
PORT=${PORT:-3006}

echo "🔧 DevHub Benchmark Mode"

# Pre-flight checks
check_deps() {
    local missing=()
    command -v node >/dev/null || missing+=("node")
    command -v npm >/dev/null || missing+=("npm")
    [ ! -f ".env.local" ] && missing+=(".env.local")
    
    if [ ${#missing[@]} -ne 0 ]; then
        echo "❌ Missing: ${missing[*]}"
        exit 1
    fi
    echo "✅ Dependencies OK"
}

# Resource monitoring
monitor_resources() {
    if command -v ps >/dev/null; then
        local pid=$1
        local cpu=$(ps -p $pid -o %cpu= 2>/dev/null | tr -d ' ')
        local mem=$(ps -p $pid -o %mem= 2>/dev/null | tr -d ' ')
        echo "📊 CPU: ${cpu}% | Memory: ${mem}%"
    fi
}

# Main execution
check_deps

echo "🚀 Starting application..."
npm run dev > .devhub.log 2>&1 &
APP_PID=$!

# Wait for ready
echo "⏳ Waiting for ready state..."
for i in {1..20}; do
    if curl -s http://localhost:$PORT > /dev/null 2>&1; then
        END_TIME=$(date +%s%N)
        STARTUP_TIME=$(( (END_TIME - START_TIME) / 1000000 ))
        
        echo "✅ Ready in ${STARTUP_TIME}ms"
        echo "🌐 http://localhost:$PORT"
        
        monitor_resources $APP_PID
        
        # Save PID for cleanup
        echo $APP_PID > .devhub.pid
        
        # Show quick stats
        echo "📈 Quick Stats:"
        echo "   • Port: $PORT"
        echo "   • PID: $APP_PID"
        echo "   • Startup: ${STARTUP_TIME}ms"
        echo "   • Stop: ./stop.sh"
        
        exit 0
    fi
    sleep 0.5
done

echo "❌ Failed to start"
kill $APP_PID 2>/dev/null
exit 1
