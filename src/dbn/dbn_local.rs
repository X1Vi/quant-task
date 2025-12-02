use dbn::decode::dbn::Decoder;
use dbn::decode::DecodeStream;
use dbn::record::MboMsg;
use fallible_streaming_iterator::FallibleStreamingIterator;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::io::AsyncWriteExt;
use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use serde_json;
use crate::types::msg::{RecordHeader, MboMsg as C_MboMsg};
use tokio::task;
use axum::{extract::State, routing::get, Json, Router};
use tower_http::cors::CorsLayer;

type BroadcastMsg = Vec<u8>;
type MessageCache = Arc<Mutex<HashMap<usize, C_MboMsg>>>;

/// Start TCP server and broadcast data to all connected clients
pub async fn start_server(addr: &str, file_path: String, sleep_time: u64) -> Result<(), Box<dyn std::error::Error>>{
    let listener = TcpListener::bind(addr).await?;
    println!("Server listening on {}", addr);
    
    let (tx, _rx) = broadcast::channel::<BroadcastMsg>(100);
    let cache: MessageCache = Arc::new(Mutex::new(HashMap::with_capacity(20)));
    
    // Rate tracking
    let message_counter = Arc::new(AtomicU64::new(0));
    
    // Start rate monitor
    let counter_clone = message_counter.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            let count = counter_clone.swap(0, Ordering::Relaxed);
            println!("📊 Server Rate: {} msg/s", count);
        }
    });

    // Start HTTP API server
    let cache_for_http = cache.clone();
    tokio::spawn(async move {
        start_http_server(cache_for_http).await;
    });
    
    let file_tx = tx.clone();
    let cache_clone = cache.clone();
    let counter_for_reader = message_counter.clone();
    
    // Spawn task to read DBN file and broadcast messages
    tokio::spawn(async move {
        if let Err(e) = read_and_broadcast_dbn(file_path, file_tx, cache_clone, counter_for_reader, sleep_time).await {
            eprintln!("Error reading DBN file: {}", e);
        }
    });
    
    // Accept client connections
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                let rx = tx.subscribe();
                tokio::spawn(handle_client(socket, addr, rx));
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}

async fn start_http_server(cache: MessageCache) {
    let app = Router::new()
        .route("/api/messages", get(get_messages))
        .layer(CorsLayer::permissive())
        .with_state(cache);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .unwrap();
    
    println!("HTTP API listening on http://0.0.0.0:3001");
    axum::serve(listener, app).await.unwrap();
}

async fn get_messages(State(cache): State<MessageCache>) -> Json<Vec<C_MboMsg>> {
    let cache_guard = cache.lock().unwrap();
    let mut messages: Vec<C_MboMsg> = cache_guard.values().cloned().collect();
    messages.sort_by_key(|m| m.sequence);
    Json(messages)
}

async fn read_and_broadcast_dbn(
    file_path: String,
    tx: broadcast::Sender<BroadcastMsg>,
    cache: MessageCache,
    counter: Arc<AtomicU64>,
    sleep_time: u64
) -> Result<(), Box<dyn std::error::Error>> {
    let result = task::spawn_blocking(move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let decoder = Decoder::from_file(file_path)?;
        let mut stream = decoder.decode_stream::<MboMsg>();
        let mut index = 0usize;
        
        while let Some(mbo_msg) = stream.next()? {
            let custom_msg = C_MboMsg {
                hd: RecordHeader {
                    rtype: mbo_msg.hd.rtype,
                    publisher_id: mbo_msg.hd.publisher_id,
                    instrument_id: mbo_msg.hd.instrument_id,
                    ts_event: mbo_msg.hd.ts_event,
                },
                order_id: mbo_msg.order_id,
                price: mbo_msg.price,
                size: mbo_msg.size,
                flags: mbo_msg.flags.raw(),
                channel_id: mbo_msg.channel_id,
                action: mbo_msg.action,
                side: mbo_msg.side,
                ts_recv: mbo_msg.ts_recv,
                ts_in_delta: mbo_msg.ts_in_delta,
                sequence: mbo_msg.sequence,
            };
            
            {
                let mut cache_guard = cache.lock().unwrap();
                cache_guard.insert(index % 20, custom_msg.clone());
            }
            
            index += 1;
            
            let serialized = serde_json::to_vec(&custom_msg)?;
            let mut message = serialized;
            message.push(b'\n');
            
            let _ = tx.send(message);
            
            // Increment counter for rate tracking
            counter.fetch_add(1, Ordering::Relaxed);
            
            // Remove println to avoid bottleneck
            std::thread::sleep(std::time::Duration::from_micros(sleep_time));
        }
        
        Ok(())
    }).await;
    
    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(Box::new(e)),
    }
}

async fn handle_client(
    mut socket: TcpStream,
    addr: SocketAddr,
    mut rx: broadcast::Receiver<BroadcastMsg>,
) {
    println!("New client connected: {}", addr);
    
    loop {
        match rx.recv().await {
            Ok(msg) => {
                if let Err(e) = socket.write_all(&msg).await {
                    eprintln!("Failed to send to {}: {}", addr, e);
                    break;
                }
            }
            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                eprintln!("Client {} lagged, skipped {} messages", addr, skipped);
                continue;
            }
            Err(broadcast::error::RecvError::Closed) => {
                println!("Broadcast channel closed");
                break;
            }
        }
    }
    
    println!("Client disconnected: {}", addr);
}
