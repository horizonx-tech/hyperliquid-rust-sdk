use aws_sdk_kms::{types::SigningAlgorithmSpec, Client as KmsClient};
use ethers::{
    core::k256::{
        ecdsa::{RecoveryId, Signature as K256Signature, VerifyingKey},
    },
    types::{Signature, H160, H256, U256},
    utils::keccak256,
};

use crate::{proxy_digest::Sha256Proxy, Error, prelude::Result};

#[derive(Clone, Debug)]
pub struct KmsSigner {
    client: KmsClient,
    key_id: String,
    public_key: VerifyingKey,
    _chain_id: u64,
}

impl KmsSigner {
    pub async fn new(key_id: String, chain_id: u64) -> Result<Self> {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = KmsClient::new(&config);

        let public_key = Self::get_public_key(&client, &key_id).await?;

        Ok(Self {
            client,
            key_id,
            public_key,
            _chain_id: chain_id,
        })
    }

    pub async fn from_client(
        client: KmsClient,
        key_id: String,
        chain_id: u64,
    ) -> Result<Self> {
        let public_key = Self::get_public_key(&client, &key_id).await?;

        Ok(Self {
            client,
            key_id,
            public_key,
            _chain_id: chain_id,
        })
    }

    async fn get_public_key(
        client: &KmsClient,
        key_id: &str,
    ) -> Result<VerifyingKey> {
        let resp = client
            .get_public_key()
            .key_id(key_id)
            .send()
            .await
            .map_err(|e| Error::Kms(format!("Failed to get public key: {}", e)))?;

        let public_key = resp
            .public_key()
            .ok_or_else(|| Error::Kms("No public key in response".to_string()))?;

        let key_bytes = public_key.as_ref();

        VerifyingKey::from_sec1_bytes(key_bytes)
            .map_err(|e| Error::Kms(format!("Invalid public key: {}", e)))
    }

    pub fn address(&self) -> H160 {
        use ethers::core::k256::elliptic_curve::sec1::ToEncodedPoint;
        
        let public_key_bytes = self.public_key.to_encoded_point(false);
        let public_key_bytes = &public_key_bytes.as_bytes()[1..];
        
        let hash = keccak256(public_key_bytes);
        H160::from_slice(&hash[12..])
    }

    pub async fn sign_hash(&self, hash: H256) -> Result<Signature> {
        let digest_bytes = hash.as_bytes().to_vec();

        let resp = self
            .client
            .sign()
            .key_id(&self.key_id)
            .message(digest_bytes.into())
            .signing_algorithm(SigningAlgorithmSpec::EcdsaSha256)
            .send()
            .await
            .map_err(|e| Error::Kms(format!("Failed to sign: {}", e)))?;

        let signature_der = resp
            .signature()
            .ok_or_else(|| Error::Kms("No signature in response".to_string()))?;

        let sig = K256Signature::from_der(signature_der.as_ref())
            .map_err(|e| Error::Kms(format!("Invalid DER signature: {}", e)))?;

        let recovery_id = self.recover_id(&sig, hash)?;
        
        let v = u8::from(recovery_id) as u64 + 27;

        let r_bytes = sig.r().to_bytes();
        let s_bytes = sig.s().to_bytes();
        let r = U256::from_big_endian(&r_bytes);
        let s = U256::from_big_endian(&s_bytes);

        Ok(Signature { r, s, v })
    }

    fn recover_id(
        &self,
        sig: &K256Signature,
        hash: H256,
    ) -> Result<RecoveryId> {
        let digest = Sha256Proxy::from(hash);
        
        for recovery_id in 0..=1 {
            let recovery_id = RecoveryId::try_from(recovery_id)
                .map_err(|e| Error::Kms(format!("Invalid recovery ID: {}", e)))?;

            if let Ok(recovered_key) = VerifyingKey::recover_from_digest(
                digest.clone(),
                &sig,
                recovery_id,
            ) {
                if recovered_key == self.public_key {
                    return Ok(recovery_id);
                }
            }
        }

        Err(Error::Kms("Failed to recover public key".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_kms_signer() -> Result<()> {
        let key_id = std::env::var("KMS_KEY_ID")
            .expect("KMS_KEY_ID environment variable not set");
        
        let signer = KmsSigner::new(key_id, 421614).await?;
        
        let hash = H256::from([0x42; 32]);
        let signature = signer.sign_hash(hash).await?;
        
        assert_eq!(signature.v, 27u64.saturating_add(0) | (signature.v & 1));
        
        Ok(())
    }
}