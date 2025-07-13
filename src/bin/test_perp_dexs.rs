use hyperliquid_rust_sdk::{BaseUrl, InfoClient};

#[tokio::main]
async fn main() {
    let info_client = InfoClient::new(None, Some(BaseUrl::Testnet))
        .await
        .expect("Failed to create info client");

    match info_client.perp_dexs().await {
        Ok(response) => {
            println!("Perp dexs response: {}", serde_json::to_string_pretty(&response).unwrap());
        }
        Err(e) => {
            eprintln!("Error fetching perp dexs: {:?}", e);
        }
    }
}