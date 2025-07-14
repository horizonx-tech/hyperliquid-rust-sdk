use async_trait::async_trait;
use ethers::{
    signers::{LocalWallet, Signer},
    types::{transaction::eip712::Eip712, Signature, H160, H256},
};

use crate::{Error, prelude::Result};
use super::{kms_signer::KmsSigner, sign_l1_action, sign_typed_data, create_signature::sign_hash};

#[async_trait]
pub trait HyperliquidSigner: Send + Sync {
    async fn sign_hash(&self, hash: H256) -> Result<Signature>;
    async fn sign_typed_data<T: Eip712 + Send + Sync>(&self, payload: &T) -> Result<Signature>;
    async fn sign_l1_action(&self, connection_id: H256, is_mainnet: bool) -> Result<Signature>;
    fn address(&self) -> H160;
}

#[derive(Debug)]
pub enum SignerType {
    LocalWallet(LocalWallet),
    Kms(KmsSigner),
}

#[async_trait]
impl HyperliquidSigner for SignerType {
    async fn sign_hash(&self, hash: H256) -> Result<Signature> {
        match self {
            SignerType::LocalWallet(wallet) => sign_hash(hash, wallet),
            SignerType::Kms(signer) => signer.sign_hash(hash).await,
        }
    }

    async fn sign_typed_data<T: Eip712 + Send + Sync>(&self, payload: &T) -> Result<Signature> {
        match self {
            SignerType::LocalWallet(wallet) => sign_typed_data(payload, wallet),
            SignerType::Kms(_) => {
                let encoded = payload
                    .encode_eip712()
                    .map_err(|e| Error::Eip712(e.to_string()))?;
                
                self.sign_hash(H256::from(encoded)).await
            }
        }
    }

    async fn sign_l1_action(&self, connection_id: H256, is_mainnet: bool) -> Result<Signature> {
        use crate::signature::agent::l1;
        
        match self {
            SignerType::LocalWallet(wallet) => sign_l1_action(wallet, connection_id, is_mainnet),
            SignerType::Kms(_) => {
                let source = if is_mainnet { "a" } else { "b" }.to_string();
                let payload = l1::Agent {
                    source,
                    connection_id,
                };
                
                self.sign_typed_data(&payload).await
            }
        }
    }

    fn address(&self) -> H160 {
        match self {
            SignerType::LocalWallet(wallet) => wallet.address(),
            SignerType::Kms(signer) => signer.address(),
        }
    }
}