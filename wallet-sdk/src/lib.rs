use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use std::collections::HashMap;
use untrace_common::PrivacyLevel;
use untrace_privacy_client::{UntraceClient, PrivateTransferClient};

pub mod adapter;
pub mod storage;

pub use adapter::WalletAdapter;
pub use storage::SecureStorage;

/// UntraceOS Wallet - Privacy-focused Web3 wallet
#[derive(Debug)]
pub struct UntraceWallet {
    /// Wallet keypair
    keypair: Keypair,
    /// Privacy client
    privacy_client: Option<UntraceClient>,
    /// Connected adapters (Phantom, Solflare, etc.)
    adapters: HashMap<String, Box<dyn WalletAdapter>>,
    /// Wallet configuration
    config: WalletConfig,
    /// Secure storage for keys and secrets
    storage: SecureStorage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    /// Default privacy level for transactions
    pub default_privacy_level: PrivacyLevel,
    /// Enable anti-MEV protection
    pub anti_mev_enabled: bool,
    /// RPC endpoint URL
    pub rpc_url: String,
    /// Program ID for privacy protocol
    pub program_id: String,
    /// Auto-mix transactions in privacy pool
    pub auto_mix_enabled: bool,
    /// Minimum pool size before withdrawal
    pub min_pool_size: u64,
}

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            default_privacy_level: PrivacyLevel::Enhanced,
            anti_mev_enabled: true,
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            program_id: "UnTrAcE1111111111111111111111111111111111111".to_string(),
            auto_mix_enabled: true,
            min_pool_size: 10,
        }
    }
}

impl UntraceWallet {
    /// Create a new wallet
    pub fn new(config: WalletConfig) -> Result<Self> {
        let keypair = Keypair::new();
        let storage = SecureStorage::new()?;

        Ok(Self {
            keypair,
            privacy_client: None,
            adapters: HashMap::new(),
            config,
            storage,
        })
    }

    /// Create wallet from existing keypair
    pub fn from_keypair(keypair: Keypair, config: WalletConfig) -> Result<Self> {
        let storage = SecureStorage::new()?;

        Ok(Self {
            keypair,
            privacy_client: None,
            adapters: HashMap::new(),
            config,
            storage,
        })
    }

    /// Initialize privacy client
    pub fn init_privacy_client(&mut self) -> Result<()> {
        let program_id = self.config.program_id.parse::<Pubkey>()
            .map_err(|e| anyhow!("Invalid program ID: {}", e))?;

        let client = UntraceClient::new(
            &self.config.rpc_url,
            program_id,
            Keypair::from_bytes(&self.keypair.to_bytes()).unwrap(),
        );

        self.privacy_client = Some(client);
        Ok(())
    }

    /// Connect to external wallet adapter (Phantom, Solflare, etc.)
    pub fn connect_adapter(&mut self, name: String, adapter: Box<dyn WalletAdapter>) -> Result<()> {
        adapter.connect()?;
        self.adapters.insert(name, adapter);
        Ok(())
    }

    /// Get public key
    pub fn public_key(&self) -> Pubkey {
        self.keypair.pubkey()
    }

    /// Send private transaction
    pub async fn send_private_transaction(
        &self,
        recipient: &Pubkey,
        amount: u64,
        privacy_level: Option<PrivacyLevel>,
    ) -> Result<String> {
        let client = self.privacy_client.as_ref()
            .ok_or_else(|| anyhow!("Privacy client not initialized"))?;

        let level = privacy_level.unwrap_or(self.config.default_privacy_level);

        let signature = client
            .private_transfer()
            .transfer(recipient, amount, level)
            .await?;

        Ok(signature.to_string())
    }

    /// Send cross-chain private transfer
    pub async fn send_cross_chain_transfer(
        &self,
        dest_chain: u16,
        recipient: &str,
        amount: u64,
        token: &str,
    ) -> Result<String> {
        let client = self.privacy_client.as_ref()
            .ok_or_else(|| anyhow!("Privacy client not initialized"))?;

        use untrace_privacy_client::cross_chain::SupportedChain;

        let source = SupportedChain::Solana;
        let dest = match dest_chain {
            1 => SupportedChain::Ethereum,
            2 => SupportedChain::BinanceSmartChain,
            3 => SupportedChain::Polygon,
            _ => return Err(anyhow!("Unsupported chain")),
        };

        let signature = client
            .cross_chain()
            .bridge_transfer(source, dest, recipient, amount, token)
            .await?;

        Ok(signature.to_string())
    }

    /// Deposit to privacy pool
    pub async fn deposit_to_pool(
        &self,
        pool_id: u64,
        recipient: &Pubkey,
        amount: u64,
    ) -> Result<(String, [u8; 32], [u8; 32])> {
        let client = self.privacy_client.as_ref()
            .ok_or_else(|| anyhow!("Privacy client not initialized"))?;

        let (signature, commitment, randomness) = client
            .privacy_pool()
            .deposit(pool_id, recipient, amount)
            .await?;

        // Store commitment and randomness in secure storage
        self.storage.store_commitment(&commitment, &randomness)?;

        Ok((signature.to_string(), commitment, randomness))
    }

    /// Withdraw from privacy pool
    pub async fn withdraw_from_pool(
        &self,
        pool_id: u64,
        commitment: &[u8; 32],
        recipient: &Pubkey,
    ) -> Result<String> {
        let client = self.privacy_client.as_ref()
            .ok_or_else(|| anyhow!("Privacy client not initialized"))?;

        // Retrieve secret from secure storage
        let secret = self.storage.get_secret(commitment)?;

        let signature = client
            .privacy_pool()
            .withdraw(pool_id, commitment, &secret, recipient)
            .await?;

        Ok(signature.to_string())
    }

    /// Get wallet balance
    pub async fn get_balance(&self) -> Result<u64> {
        let client = self.privacy_client.as_ref()
            .ok_or_else(|| anyhow!("Privacy client not initialized"))?;

        let balance = client.rpc_client.get_balance(&self.keypair.pubkey())?;
        Ok(balance)
    }

    /// Export wallet (encrypted)
    pub fn export_encrypted(&self, password: &str) -> Result<String> {
        self.storage.export_wallet(&self.keypair, password)
    }

    /// Import wallet (encrypted)
    pub fn import_encrypted(encrypted: &str, password: &str, config: WalletConfig) -> Result<Self> {
        let storage = SecureStorage::new()?;
        let keypair = storage.import_wallet(encrypted, password)?;

        Ok(Self {
            keypair,
            privacy_client: None,
            adapters: HashMap::new(),
            config,
            storage,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let config = WalletConfig::default();
        let wallet = UntraceWallet::new(config).unwrap();

        let pubkey = wallet.public_key();
        assert_ne!(pubkey, Pubkey::default());
    }

    #[test]
    fn test_wallet_export_import() {
        let config = WalletConfig::default();
        let wallet = UntraceWallet::new(config.clone()).unwrap();
        let original_pubkey = wallet.public_key();

        let password = "test_password_123";
        let encrypted = wallet.export_encrypted(password).unwrap();

        let imported_wallet = UntraceWallet::import_encrypted(&encrypted, password, config).unwrap();
        let imported_pubkey = imported_wallet.public_key();

        assert_eq!(original_pubkey, imported_pubkey);
    }
}
