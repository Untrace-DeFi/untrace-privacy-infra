use anyhow::Result;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signature,
    system_program,
    sysvar::clock,
};
use untrace_common::crypto;

use crate::UntraceClient;

pub struct PrivacyPoolClient<'a> {
    client: &'a UntraceClient,
}

impl<'a> PrivacyPoolClient<'a> {
    pub fn new(client: &'a UntraceClient) -> Self {
        Self { client }
    }

    /// Initialize a new privacy pool
    pub async fn initialize_pool(
        &self,
        pool_id: u64,
        min_pool_size: u64,
    ) -> Result<Signature> {
        let (pool_pda, _bump) = Pubkey::find_program_address(
            &[b"privacy_pool", &pool_id.to_le_bytes()],
            &self.client.program_id,
        );

        let mut data = vec![0u8]; // Instruction discriminator
        data.extend_from_slice(&pool_id.to_le_bytes());
        data.extend_from_slice(&min_pool_size.to_le_bytes());

        let instruction = Instruction {
            program_id: self.client.program_id,
            accounts: vec![
                AccountMeta::new(pool_pda, false),
                AccountMeta::new(self.client.payer.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data,
        };

        self.client.send_transaction(vec![instruction]).await
    }

    /// Deposit funds into privacy pool
    pub async fn deposit(
        &self,
        pool_id: u64,
        recipient: &Pubkey,
        amount: u64,
    ) -> Result<(Signature, [u8; 32], [u8; 32])> {
        let (commitment, randomness) = self.client.generate_commitment(recipient, amount);

        let (pool_pda, _) = Pubkey::find_program_address(
            &[b"privacy_pool", &pool_id.to_le_bytes()],
            &self.client.program_id,
        );

        let commitment_account = Pubkey::new_unique();

        // Encrypt the deposit data
        let mut plaintext = Vec::new();
        plaintext.extend_from_slice(&recipient.to_bytes());
        plaintext.extend_from_slice(&amount.to_le_bytes());

        let mut shared_secret = randomness;
        let nonce = [0u8; 12];
        let (encrypted_data, _tag) = crypto::encrypt_data(&plaintext, &shared_secret, &nonce)
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut data = vec![1u8]; // Instruction discriminator
        data.extend_from_slice(&commitment);
        data.extend_from_slice(&(encrypted_data.len() as u32).to_le_bytes());
        data.extend_from_slice(&encrypted_data);

        let instruction = Instruction {
            program_id: self.client.program_id,
            accounts: vec![
                AccountMeta::new(pool_pda, false),
                AccountMeta::new(commitment_account, false),
                AccountMeta::new(self.client.payer.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data,
        };

        let signature = self.client.send_transaction(vec![instruction]).await?;

        Ok((signature, commitment, randomness))
    }

    /// Withdraw funds from privacy pool
    pub async fn withdraw(
        &self,
        pool_id: u64,
        commitment: &[u8; 32],
        secret: &[u8],
        recipient: &Pubkey,
    ) -> Result<Signature> {
        let nullifier = self.client.generate_nullifier(secret, commitment);

        let (pool_pda, _) = Pubkey::find_program_address(
            &[b"privacy_pool", &pool_id.to_le_bytes()],
            &self.client.program_id,
        );

        let nullifier_account = Pubkey::new_unique();

        // Generate ZK proof
        let mut secret_hash = [0u8; 32];
        secret_hash[..secret.len().min(32)].copy_from_slice(&secret[..secret.len().min(32)]);
        let zk_proof = crypto::generate_zk_proof(commitment, &nullifier, &secret_hash);

        // Generate merkle proof (simplified)
        let merkle_proof = vec![[0u8; 32]; 10];

        let mut data = vec![2u8]; // Instruction discriminator
        data.extend_from_slice(&nullifier);
        data.extend_from_slice(&recipient.to_bytes());
        data.extend_from_slice(&(zk_proof.len() as u32).to_le_bytes());
        data.extend_from_slice(&zk_proof);
        data.extend_from_slice(&(merkle_proof.len() as u32).to_le_bytes());
        for proof_element in merkle_proof {
            data.extend_from_slice(&proof_element);
        }

        let instruction = Instruction {
            program_id: self.client.program_id,
            accounts: vec![
                AccountMeta::new(pool_pda, false),
                AccountMeta::new(nullifier_account, false),
                AccountMeta::new(self.client.payer.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data,
        };

        self.client.send_transaction(vec![instruction]).await
    }
}
