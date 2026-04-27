#!/usr/bin/env bash

# Dev Assistant Hub - Logs Script

# Check if running in Docker or Nix/systemd environment
if docker-compose ps | grep -q "Up"; then
    # Docker environment
    case "$1" in
        "nginx")
            docker-compose logs -f nginx
            ;;
        "app")
            docker-compose logs -f app
            ;;
        "db")
            docker-compose logs -f db
            ;;
        "redis")
            docker-compose logs -f redis
            ;;
        *)
            echo "📋 Docker Environment - Available log options:"
            echo "  ./logs.sh nginx  - View Nginx logs"
            echo "  ./logs.sh app    - View Application logs"
            echo "  ./logs.sh db     - View Database logs"
            echo "  ./logs.sh redis  - View Redis logs"
            echo ""
            echo "📊 All Docker services logs:"
            docker-compose logs -f
            ;;
    esac
else
    # Nix/systemd environment
    case "$1" in
        "app"|"mission-control")
            journalctl -u mission-control -f
            ;;
        "metrics"|"mission-control-metrics")
            journalctl -u mission-control-metrics -f
            ;;
        "ollama")
            journalctl -u ollama -f
            ;;
        *)
            echo "📋 NixOS Environment - Available log options:"
            echo "  ./logs.sh app     - View Mission Control application logs"
            echo "  ./logs.sh metrics - View Mission Control metrics logs"
            echo "  ./logs.sh ollama  - View Ollama service logs"
            echo ""
            echo "📊 All systemd services:"
            systemctl list-units --type=service --state=active | grep -E "(mission-control|ollama)"
            ;;
    esac
fi
