use anyhow::Result;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signature,
    system_program,
};
use untrace_common::crypto;

use crate::UntraceClient;

#[derive(Debug, Clone, Copy)]
pub enum SupportedChain {
    Ethereum = 1,
    BinanceSmartChain = 2,
    Polygon = 3,
    Avalanche = 4,
    Arbitrum = 5,
    Optimism = 6,
    Solana = 7,
}

impl SupportedChain {
    pub fn to_u16(&self) -> u16 {
        *self as u16
    }
}

pub struct CrossChainClient<'a> {
    client: &'a UntraceClient,
}

impl<'a> CrossChainClient<'a> {
    pub fn new(client: &'a UntraceClient) -> Self {
        Self { client }
    }

    /// Initiate a cross-chain private transfer
    pub async fn bridge_transfer(
        &self,
        source_chain: SupportedChain,
        dest_chain: SupportedChain,
        recipient: &str,
        amount: u64,
        token: &str,
    ) -> Result<Signature> {
        let bridge_account = Pubkey::new_unique();

        // Prepare transfer data
        let mut transfer_data = Vec::new();
        transfer_data.extend_from_slice(recipient.as_bytes());
        transfer_data.extend_from_slice(&amount.to_le_bytes());
        transfer_data.extend_from_slice(token.as_bytes());

        // Encrypt the transfer data
        let mut shared_secret = [0u8; 32];
        rand::Rng::fill(&mut rand::thread_rng(), &mut shared_secret);

        let mut nonce = [0u8; 12];
        rand::Rng::fill(&mut rand::thread_rng(), &mut nonce);

        let (encrypted_data, tag) = crypto::encrypt_data(&transfer_data, &shared_secret, &nonce)
            .map_err(|e| anyhow::anyhow!(e))?;

        // Generate ephemeral public key
        let mut ephemeral_pubkey = [0u8; 32];
        rand::Rng::fill(&mut rand::thread_rng(), &mut ephemeral_pubkey);

        let mut data = vec![4u8]; // Instruction discriminator
        data.extend_from_slice(&source_chain.to_u16().to_le_bytes());
        data.extend_from_slice(&dest_chain.to_u16().to_le_bytes());
        data.extend_from_slice(&(encrypted_data.len() as u32).to_le_bytes());
        data.extend_from_slice(&encrypted_data);
        data.extend_from_slice(&ephemeral_pubkey);
        data.extend_from_slice(&nonce);
        data.extend_from_slice(&tag);

        let instruction = Instruction {
            program_id: self.client.program_id,
            accounts: vec![
                AccountMeta::new(bridge_account, false),
                AccountMeta::new(self.client.payer.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data,
        };

        self.client.send_transaction(vec![instruction]).await
    }

    /// Query bridge transfer status
    pub async fn get_bridge_status(&self, bridge_account: &Pubkey) -> Result<BridgeStatus> {
        let account = self.client.rpc_client.get_account(bridge_account)?;

        if account.data.is_empty() {
            return Ok(BridgeStatus::NotFound);
        }

        // Parse status from account data (simplified)
        if account.data.len() > 100 {
            let status_byte = account.data[account.data.len() - 1];
            match status_byte {
                0 => Ok(BridgeStatus::Pending),
                1 => Ok(BridgeStatus::Completed),
                2 => Ok(BridgeStatus::Failed),
                _ => Ok(BridgeStatus::Unknown),
            }
        } else {
            Ok(BridgeStatus::Unknown)
        }
    }

    /// Estimate bridge fees
    pub fn estimate_bridge_fee(
        &self,
        source_chain: SupportedChain,
        dest_chain: SupportedChain,
        amount: u64,
    ) -> u64 {
        // Base fee + percentage
        let base_fee = 1_000_000; // 0.001 SOL
        let percentage_fee = amount / 1000; // 0.1%

        // Chain-specific multipliers
        let chain_multiplier = match (source_chain, dest_chain) {
            (SupportedChain::Solana, _) | (_, SupportedChain::Solana) => 1,
            (SupportedChain::Ethereum, _) | (_, SupportedChain::Ethereum) => 3,
            _ => 2,
        };

        base_fee + (percentage_fee * chain_multiplier)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeStatus {
    NotFound,
    Pending,
    Completed,
    Failed,
    Unknown,
}
