pub mod types;
pub mod dbn;

use crate::dbn::dbn_local::start_server;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    start_server("127.0.0.1:8080", "CLX5_mbo.dbn".to_string(), 0).await
    // there is sleep time messages are intentionally nerfed for bettter visualization !
    // if we minimize the sleep time sends the message instantly there is a mild overhead I think
    // In the second time it send all the messages together if there would have been more messages then they would have been sent too I think it is hitting the target need more messages and bigger file to benchmark
}

