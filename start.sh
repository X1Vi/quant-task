#!/usr/bin/env bash
set -e

echo "🚀 HFT System - Docker Startup"
echo "=============================="

case "${1:-up}" in
  up)
    echo "Starting all services..."
    docker-compose up --build -d
    echo ""
    echo "✅ Services started!"
    echo ""
    echo "📊 Endpoints:"
    echo "   TCP Stream:  localhost:8080"
    echo "   HTTP API:    http://localhost:3001/api/messages"
    echo "   React UI:    http://localhost:5173"
    echo ""
    echo "📝 Commands:"
    echo "   ./start.sh logs    - View logs"
    echo "   ./start.sh stop    - Stop all"
    echo "   ./start.sh restart - Restart all"
    ;;
  stop)
    echo "Stopping all services..."
    docker-compose down
    echo "✅ Stopped"
    ;;
  restart)
    echo "Restarting..."
    docker-compose down
    docker-compose up --build -d
    echo "✅ Restarted"
    ;;
  logs)
    docker-compose logs -f
    ;;
  *)
    echo "Usage: ./start.sh [up|stop|restart|logs]"
    ;;
esac
