# Docker Setup Guide

## Prerequisites

```bash
docker ps                    # verify Docker is running
docker compose version       # verify Docker Compose is installed
```

If Docker isn't running:
```bash
sudo systemctl start docker  # Linux
# or use Docker Desktop      # macOS/Windows
```

## Start Everything

```bash
chmod +x start.sh
./start.sh
```

## Endpoints

| Service | URL | Purpose |
|---------|-----|---------|
| **Rust Server (TCP)** | `localhost:8080` | Raw MBO stream (newline-delimited JSON) |
| **Rust Server (HTTP)** | `http://localhost:3001/api/messages` | Last 20 cached messages |
| **React Frontend** | `http://localhost:5173` | Live order book visualization |

## Commands

```bash
./start.sh logs              # view live logs
./start.sh stop              # stop all services
./start.sh restart           # restart all services
docker compose ps            # check container status
```

## Troubleshooting

**Rust server exits immediately:**
```bash
docker compose logs rust-server
# should show "Server listening on 0.0.0.0:8080"
# if not, rebuild: docker compose build rust-server
```

**Python client can't connect:**
```bash
docker compose logs python-client
# check for connection errors - Rust may not be ready yet
```

**React shows no data:**
```bash
curl http://localhost:3001/api/messages
# should return JSON array, even if empty
```

**Port conflict (8080, 3001, or 5173):**
```bash
docker compose down
# then try again
```
