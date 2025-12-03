# HFT Task Setup

## Quick Start

### 1. Start Python Client First
```bash
cd py-server
python3 server.py
```

### 2. Start React App
```bash
cd react-client-for-api/my-react-app
npm install
npm run dev
```

### 3. Start Rust Server
```bash
cargo run
```



## Important Config

Add 20µs sleep in `src/dbn/dbn_local.rs` for better visualization:

```rust
    start_server("127.0.0.1:8080", "CLX5_mbo.dbn".to_string(), 20).await
```

## Endpoints

- TCP: `127.0.0.1:8080` (Python client)
- HTTP: `http://localhost:3001/api/messages` (React app)

## Requirements

- rustc 1.91.1 (ed61e7d7e 2025-11-07)
- Python 3.12.3  (no venv needed - uses native libraries only)
- Node.js v22.19.0

## Notes

- Python client uses only standard library (asyncio, json, time)
- No external pip dependencies required
- Always start in order: Python → React → Rust
