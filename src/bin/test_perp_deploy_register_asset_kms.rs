use std::collections::HashMap;

use log::info;

use hyperliquid_rust_sdk::{
    BaseUrl, ExchangeClient, ExchangeResponseStatus, PerpDexSchemaInput, 
    KmsSigner, SignerType,
};

#[tokio::main]
async fn main() {
    env_logger::init();
    
    // Get KMS key ID from environment variable
    let kms_key_id = std::env::var("KMS_KEY_ID")
        .expect("KMS_KEY_ID environment variable must be set");
    
    // Create KMS signer
    let kms_signer = KmsSigner::new(kms_key_id, 421614)
        .await
        .expect("Failed to create KMS signer");
    
    info!("Using KMS address: {:?}", kms_signer.address());
    
    let signer = SignerType::Kms(kms_signer);
    
    let exchange_client = ExchangeClient::new_with_signer(None, signer, Some(BaseUrl::Testnet), None, None)
        .await
        .unwrap();

    // Test 1: Register asset without schema
    info!("Test 1: Registering asset without schema...");
    let dex = "tstdex";
    let coin_id = format!("{}:{}", dex, "TESTCOIN");
    match exchange_client
        .perp_deploy_register_asset(
            dex,
            None, // max_gas
            format!("{}:{}", dex, "TESTCOIN").as_str(),
            2,        // sz_decimals
            "10.0",  // oracle_px
            10,        // margin_table_id
            false,    // only_isolated
                 Some(PerpDexSchemaInput {
                    full_name: "New Coin".to_string(),
                    collateral_token: 0,  // 0 for USDC
                    oracle_updater: Some("0x0000000000000000000000000000000000000000".to_string()),
                }),
        )
        .await
    {
        Ok(response) => match response {
            ExchangeResponseStatus::Ok(data) => {
                info!("Successfully registered asset without schema: {:?}", data);
            }
            ExchangeResponseStatus::Err(e) => {
                info!("Expected error registering asset without schema: {}", e);
            }
        },
        Err(e) => {
            info!("Network/parsing error: {:?}", e);
            info!("Note: This error might be expected if you don't have permissions to register assets on testnet");
        }
    }

    let mut oracle_pxs = HashMap::new();
    oracle_pxs.insert(coin_id, "10.0".to_string());
    let all_mark_pxs = vec![];
    
    let oracle_result = exchange_client.perp_deploy_set_oracle(dex, oracle_pxs, all_mark_pxs).await;
    match oracle_result {
        Ok(response) => {
            info!("Successfully set oracle: {:?}", response);
        }
        Err(e) => {
            info!("Error setting oracle: {:?}", e);
        }
    }
}