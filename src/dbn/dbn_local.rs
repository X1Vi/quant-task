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
use serde_json;
use crate::types::msg::{RecordHeader, MboMsg as C_MboMsg};
use tokio::task;
type BroadcastMsg = Vec<u8>;
type MessageCache = Arc<Mutex<HashMap<usize, C_MboMsg>>>;

/// Start TCP server and broadcast data to all connected clients
pub async fn start_server(addr: &str, file_path: String) -> Result<(), Box<dyn std::error::Error>>{
    let listener = TcpListener::bind(addr).await?;
    println!("Server listening on {}", addr);
    
    let (tx, _rx) = broadcast::channel::<BroadcastMsg>(100);
    
    // Create bounded cache - max 20 messages
    let cache: MessageCache = Arc::new(Mutex::new(HashMap::with_capacity(20)));
    
    let file_tx = tx.clone();
    let cache_clone = cache.clone();
    
    // Spawn task to read DBN file and broadcast messages
    tokio::spawn(async move {
        if let Err(e) = read_and_broadcast_dbn(file_path, file_tx, cache_clone).await {
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

async fn read_and_broadcast_dbn(
    file_path: String,
    tx: broadcast::Sender<BroadcastMsg>,
    cache: MessageCache,
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
            
            println!("Broadcasted: {:?}", custom_msg);
            
            // Add delay to simulate real-time streaming
            std::thread::sleep(std::time::Duration::from_micros(100));
        }
        
        Ok(())
    }).await;
    
    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(Box::new(e)),
    }
}

// handle_client remains the same
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
