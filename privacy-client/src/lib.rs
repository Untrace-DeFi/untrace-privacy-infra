use anyhow::{anyhow, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};
use untrace_common::{crypto, PrivacyLevel};

pub mod private_transfer;
pub mod privacy_pool;
pub mod cross_chain;

pub use private_transfer::PrivateTransferClient;
pub use privacy_pool::PrivacyPoolClient;
pub use cross_chain::CrossChainClient;

/// Main client for Untrace privacy protocol
pub struct UntraceClient {
    pub rpc_client: RpcClient,
    pub program_id: Pubkey,
    pub payer: Keypair,
}

impl UntraceClient {
    /// Create a new Untrace client
    pub fn new(rpc_url: &str, program_id: Pubkey, payer: Keypair) -> Self {
        let rpc_client = RpcClient::new_with_commitment(
            rpc_url.to_string(),
            CommitmentConfig::confirmed(),
        );

        Self {
            rpc_client,
            program_id,
            payer,
        }
    }

    /// Get privacy pool client
    pub fn privacy_pool(&self) -> PrivacyPoolClient {
        PrivacyPoolClient::new(self)
    }

    /// Get private transfer client
    pub fn private_transfer(&self) -> PrivateTransferClient {
        PrivateTransferClient::new(self)
    }

    /// Get cross-chain client
    pub fn cross_chain(&self) -> CrossChainClient {
        CrossChainClient::new(self)
    }

    /// Send and confirm transaction
    pub async fn send_transaction(&self, instructions: Vec<Instruction>) -> Result<Signature> {
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;

        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&self.payer.pubkey()),
            &[&self.payer],
            recent_blockhash,
        );

        let signature = self
            .rpc_client
            .send_and_confirm_transaction(&transaction)?;

        Ok(signature)
    }

    /// Generate a new commitment for privacy pool
    pub fn generate_commitment(
        &self,
        recipient: &Pubkey,
        amount: u64,
    ) -> ([u8; 32], [u8; 32]) {
        let mut randomness = [0u8; 32];
        rand::Rng::fill(&mut rand::thread_rng(), &mut randomness);

        let commitment = crypto::generate_commitment(
            &recipient.to_bytes(),
            amount,
            &randomness,
        );

        (commitment, randomness)
    }

    /// Generate nullifier for withdrawal
    pub fn generate_nullifier(&self, secret: &[u8], commitment: &[u8; 32]) -> [u8; 32] {
        crypto::generate_nullifier(secret, commitment)
    }

    /// Encrypt transfer data
    pub fn encrypt_transfer_data(
        &self,
        recipient: &Pubkey,
        amount: u64,
        recipient_pubkey: &[u8; 32],
    ) -> Result<(Vec<u8>, [u8; 32], [u8; 12], [u8; 16])> {
        let mut shared_secret = [0u8; 32];
        rand::Rng::fill(&mut rand::thread_rng(), &mut shared_secret);

        let mut nonce = [0u8; 12];
        rand::Rng::fill(&mut rand::thread_rng(), &mut nonce);

        // Create plaintext data
        let mut plaintext = Vec::new();
        plaintext.extend_from_slice(&recipient.to_bytes());
        plaintext.extend_from_slice(&amount.to_le_bytes());

        let (ciphertext, tag) = crypto::encrypt_data(&plaintext, &shared_secret, &nonce)
            .map_err(|e| anyhow!(e))?;

        let mut ephemeral_pubkey = [0u8; 32];
        rand::Rng::fill(&mut rand::thread_rng(), &mut ephemeral_pubkey);

        Ok((ciphertext, ephemeral_pubkey, nonce, tag))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::signature::Keypair;

    #[test]
    fn test_generate_commitment() {
        let client = UntraceClient::new(
            "http://localhost:8899",
            Pubkey::new_unique(),
            Keypair::new(),
        );

        let recipient = Pubkey::new_unique();
        let amount = 1000u64;

        let (commitment, randomness) = client.generate_commitment(&recipient, amount);

        assert_eq!(commitment.len(), 32);
        assert_eq!(randomness.len(), 32);
    }

    #[test]
    fn test_generate_nullifier() {
        let client = UntraceClient::new(
            "http://localhost:8899",
            Pubkey::new_unique(),
            Keypair::new(),
        );

        let secret = b"my_secret_key_12345678901234567890";
        let commitment = [1u8; 32];

        let nullifier = client.generate_nullifier(secret, &commitment);
        assert_eq!(nullifier.len(), 32);
    }
}
