mod dbn;
mod types;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting HFT server...");

    // IMPORTANT: await the server future and return its Result
    dbn::dbn_local::start_server(
        "0.0.0.0:8080",
        "CLX5_mbo.dbn".to_string(),
        20,
        50
    ).await
}
