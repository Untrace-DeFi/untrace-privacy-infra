use anyhow::Result;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signature,
    system_program,
};
use untrace_common::{crypto, PrivacyLevel};

use crate::UntraceClient;

pub struct PrivateTransferClient<'a> {
    client: &'a UntraceClient,
}

impl<'a> PrivateTransferClient<'a> {
    pub fn new(client: &'a UntraceClient) -> Self {
        Self { client }
    }

    /// Execute a private transfer
    pub async fn transfer(
        &self,
        recipient: &Pubkey,
        amount: u64,
        privacy_level: PrivacyLevel,
    ) -> Result<Signature> {
        let transfer_account = Pubkey::new_unique();

        // Generate recipient's ephemeral key
        let mut recipient_ephemeral = [0u8; 32];
        rand::Rng::fill(&mut rand::thread_rng(), &mut recipient_ephemeral);

        // Encrypt amount
        let amount_bytes = amount.to_le_bytes();
        let mut shared_secret = [0u8; 32];
        rand::Rng::fill(&mut rand::thread_rng(), &mut shared_secret);
        let nonce = [0u8; 12];

        let (encrypted_amount, _) = crypto::encrypt_data(&amount_bytes, &shared_secret, &nonce)
            .map_err(|e| anyhow::anyhow!(e))?;

        // Encrypt recipient
        let recipient_bytes = recipient.to_bytes();
        let (encrypted_recipient, _) = crypto::encrypt_data(&recipient_bytes, &shared_secret, &nonce)
            .map_err(|e| anyhow::anyhow!(e))?;

        // Generate ZK proof
        let commitment = crypto::generate_commitment(&recipient_bytes, amount, &shared_secret);
        let nullifier = crypto::generate_nullifier(&shared_secret, &commitment);
        let zk_proof = crypto::generate_zk_proof(&commitment, &nullifier, &shared_secret);

        let privacy_level_u8 = match privacy_level {
            PrivacyLevel::Basic => 0u8,
            PrivacyLevel::Enhanced => 1u8,
            PrivacyLevel::Maximum => 2u8,
        };

        let mut data = vec![3u8]; // Instruction discriminator
        data.extend_from_slice(&(encrypted_amount.len() as u32).to_le_bytes());
        data.extend_from_slice(&encrypted_amount);
        data.extend_from_slice(&(encrypted_recipient.len() as u32).to_le_bytes());
        data.extend_from_slice(&encrypted_recipient);
        data.extend_from_slice(&(zk_proof.len() as u32).to_le_bytes());
        data.extend_from_slice(&zk_proof);
        data.push(privacy_level_u8);

        let instruction = Instruction {
            program_id: self.client.program_id,
            accounts: vec![
                AccountMeta::new(transfer_account, false),
                AccountMeta::new(self.client.payer.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data,
        };

        self.client.send_transaction(vec![instruction]).await
    }

    /// Execute a batch of private transfers for better anonymity
    pub async fn batch_transfer(
        &self,
        transfers: Vec<(Pubkey, u64)>,
        privacy_level: PrivacyLevel,
    ) -> Result<Vec<Signature>> {
        let mut signatures = Vec::new();

        for (recipient, amount) in transfers {
            let sig = self.transfer(&recipient, amount, privacy_level).await?;
            signatures.push(sig);
        }

        Ok(signatures)
    }

    /// Query transfer status
    pub async fn get_transfer_status(&self, transfer_account: &Pubkey) -> Result<TransferStatus> {
        let account = self.client.rpc_client.get_account(transfer_account)?;

        if account.data.is_empty() {
            return Ok(TransferStatus::NotFound);
        }

        Ok(TransferStatus::Completed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferStatus {
    NotFound,
    Pending,
    Completed,
    Failed,
}
