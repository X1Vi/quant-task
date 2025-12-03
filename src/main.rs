pub mod types;
pub mod dbn;

use crate::dbn::dbn_local::start_server;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    start_server("127.0.0.1:8080", "CLX5_mbo.dbn".to_string(), 0).await
}

