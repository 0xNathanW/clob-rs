use std::time::Instant;

// Estimate latency of clob.

#[tokio::main]
async fn main() {
    let clob = clob::ClobClient::from_env().unwrap();
    let start = Instant::now();
    let _ = clob.cancel_all().await.unwrap();
    let end = Instant::now();
    println!("Latency: {:?}", end - start);
}
