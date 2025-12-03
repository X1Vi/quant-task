#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Using root: $ROOT_DIR"

# 1) Root Dockerfile (Rust server)
cat > "$ROOT_DIR/Dockerfile" << 'EOF'
FROM rust:1.91.1-slim AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main(){}" > src/main.rs && cargo build --release && rm -rf src

COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/hft-task .
COPY CLX5_mbo.dbn .

EXPOSE 8080 3001

CMD ["./hft-task"]
EOF

# 2) docker-compose.yml
cat > "$ROOT_DIR/docker-compose.yml" << 'EOF'
services:
  rust-server:
    build: .
    container_name: hft-server
    ports:
      - "8080:8080"
      - "3001:3001"
    restart: unless-stopped

  python-client:
    build: ./py-server
    container_name: hft-python
    depends_on:
      - rust-server
    environment:
      - SERVER_HOST=rust-server
    restart: unless-stopped

  react-client:
    build: ./react-client-for-api/my-react-app
    container_name: hft-react
    ports:
      - "5173:5173"
    depends_on:
      - rust-server
    restart: unless-stopped
EOF

# 3) start.sh
cat > "$ROOT_DIR/start.sh" << 'EOF'
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
EOF
chmod +x "$ROOT_DIR/start.sh"

# 4) py-server/Dockerfile
cat > "$ROOT_DIR/py-server/Dockerfile" << 'EOF'
FROM python:3.12-slim

WORKDIR /app
COPY server.py .

CMD ["python", "server.py"]
EOF

# 5) Update py-server/server.py (non-destructive backup)
if [ -f "$ROOT_DIR/py-server/server.py" ]; then
  cp "$ROOT_DIR/py-server/server.py" "$ROOT_DIR/py-server/server.py.bak"
fi

cat > "$ROOT_DIR/py-server/server.py" << 'EOF'
import asyncio
import time
import os

SERVER_HOST = os.getenv("SERVER_HOST", "127.0.0.1")
SERVER_PORT = 8080

async def main():
    print(f"Connecting to {SERVER_HOST}:{SERVER_PORT}...")
    while True:
        try:
            reader, writer = await asyncio.open_connection(SERVER_HOST, SERVER_PORT)
            print("Connected to TCP stream")
            count = 0
            start = time.time()
            while True:
                line = await reader.readline()
                if not line:
                    break
                count += 1
                if count % 10000 == 0:
                    elapsed = time.time() - start
                    rate = count / elapsed if elapsed > 0 else 0
                    print(f"Received {count} msgs | {rate:.0f} msg/s")
        except Exception as e:
            print(f"Connection error: {e}, retrying in 2s...")
            await asyncio.sleep(2)

if __name__ == "__main__":
    asyncio.run(main())
EOF

# 6) React Dockerfile
cat > "$ROOT_DIR/react-client-for-api/my-react-app/Dockerfile" << 'EOF'
FROM node:22-alpine

WORKDIR /app

COPY package*.json ./
RUN npm install

COPY . .

EXPOSE 5173

CMD ["npm", "run", "dev", "--", "--host"]
EOF

echo "✅ Scaffolding complete."
echo "Next steps:"
echo "  cd \"$ROOT_DIR\""
echo "  ./start.sh"

