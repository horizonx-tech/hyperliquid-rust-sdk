# AWS KMS Signing Support

This SDK now supports signing transactions using AWS KMS (Key Management Service) in addition to local wallets.

## Setup

1. Ensure you have AWS credentials configured:
   ```bash
   aws configure
   ```

2. Create or use an existing KMS key with the following key spec:
   - Key type: `ECC_SECG_P256K1` (Secp256k1)
   - Key usage: `SIGN_VERIFY`

3. Grant your IAM user/role the following KMS permissions:
   - `kms:GetPublicKey`
   - `kms:Sign`

## Usage

### Using KMS Signer

```rust
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient, KmsSigner, SignerType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create KMS signer with your key ID
    let kms_key_id = "arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012";
    let kms_signer = KmsSigner::new(kms_key_id.to_string(), 421614).await?;
    
    println!("KMS address: {:?}", kms_signer.address());
    
    // Create exchange client with KMS signer
    let signer = SignerType::Kms(kms_signer);
    let exchange_client = ExchangeClient::new_with_signer(
        None,
        signer,
        Some(BaseUrl::Testnet),
        None,
        None
    ).await?;
    
    // Use the exchange client normally
    // All signing operations will use AWS KMS
    
    Ok(())
}
```

### Using Local Wallet (existing behavior)

```rust
use ethers::signers::LocalWallet;
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wallet: LocalWallet = "your_private_key_here".parse()?;
    
    let exchange_client = ExchangeClient::new(
        None,
        wallet,
        Some(BaseUrl::Testnet),
        None,
        None
    ).await?;
    
    // Use as before
    
    Ok(())
}
```

## Environment Variables

For the KMS example binaries, set the following environment variable:

```bash
export KMS_KEY_ID="your-kms-key-id-or-arn"
```

## Security Considerations

1. **Access Control**: Ensure your KMS key has appropriate access policies
2. **Audit Trail**: All KMS signing operations are logged in AWS CloudTrail
3. **Key Rotation**: KMS supports automatic key rotation for enhanced security
4. **Region**: Make sure your KMS key is in a region close to your application for optimal performance

## Example

See `src/bin/test_perp_deploy_register_asset_kms.rs` for a complete example of using KMS signing.