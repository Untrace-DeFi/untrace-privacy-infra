use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use std::fmt::Debug;

/// Trait for wallet adapters (Phantom, Solflare, etc.)
pub trait WalletAdapter: Debug + Send + Sync {
    /// Connect to the wallet
    fn connect(&self) -> Result<()>;

    /// Disconnect from the wallet
    fn disconnect(&self) -> Result<()>;

    /// Get the connected wallet's public key
    fn get_public_key(&self) -> Result<Pubkey>;

    /// Check if wallet is connected
    fn is_connected(&self) -> bool;

    /// Sign a transaction
    fn sign_transaction(&self, transaction: &[u8]) -> Result<Vec<u8>>;

    /// Sign a message
    fn sign_message(&self, message: &[u8]) -> Result<Vec<u8>>;
}

/// Phantom wallet adapter
#[derive(Debug)]
pub struct PhantomAdapter {
    connected: bool,
    public_key: Option<Pubkey>,
}

impl PhantomAdapter {
    pub fn new() -> Self {
        Self {
            connected: false,
            public_key: None,
        }
    }
}

impl WalletAdapter for PhantomAdapter {
    fn connect(&self) -> Result<()> {
        // In a real implementation, this would use browser APIs
        // to connect to the Phantom wallet extension
        println!("Connecting to Phantom wallet...");
        Ok(())
    }

    fn disconnect(&self) -> Result<()> {
        println!("Disconnecting from Phantom wallet...");
        Ok(())
    }

    fn get_public_key(&self) -> Result<Pubkey> {
        self.public_key
            .ok_or_else(|| anyhow::anyhow!("Wallet not connected"))
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn sign_transaction(&self, transaction: &[u8]) -> Result<Vec<u8>> {
        // In production, this would call Phantom's sign API
        Ok(transaction.to_vec())
    }

    fn sign_message(&self, message: &[u8]) -> Result<Vec<u8>> {
        // In production, this would call Phantom's sign message API
        Ok(message.to_vec())
    }
}

/// Solflare wallet adapter
#[derive(Debug)]
pub struct SolflareAdapter {
    connected: bool,
    public_key: Option<Pubkey>,
}

impl SolflareAdapter {
    pub fn new() -> Self {
        Self {
            connected: false,
            public_key: None,
        }
    }
}

impl WalletAdapter for SolflareAdapter {
    fn connect(&self) -> Result<()> {
        println!("Connecting to Solflare wallet...");
        Ok(())
    }

    fn disconnect(&self) -> Result<()> {
        println!("Disconnecting from Solflare wallet...");
        Ok(())
    }

    fn get_public_key(&self) -> Result<Pubkey> {
        self.public_key
            .ok_or_else(|| anyhow::anyhow!("Wallet not connected"))
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn sign_transaction(&self, transaction: &[u8]) -> Result<Vec<u8>> {
        Ok(transaction.to_vec())
    }

    fn sign_message(&self, message: &[u8]) -> Result<Vec<u8>> {
        Ok(message.to_vec())
    }
}

/// Generic Web3 wallet adapter
#[derive(Debug)]
pub struct Web3Adapter {
    wallet_type: String,
    connected: bool,
    public_key: Option<Pubkey>,
}

impl Web3Adapter {
    pub fn new(wallet_type: String) -> Self {
        Self {
            wallet_type,
            connected: false,
            public_key: None,
        }
    }
}

impl WalletAdapter for Web3Adapter {
    fn connect(&self) -> Result<()> {
        println!("Connecting to {} wallet...", self.wallet_type);
        Ok(())
    }

    fn disconnect(&self) -> Result<()> {
        println!("Disconnecting from {} wallet...", self.wallet_type);
        Ok(())
    }

    fn get_public_key(&self) -> Result<Pubkey> {
        self.public_key
            .ok_or_else(|| anyhow::anyhow!("Wallet not connected"))
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn sign_transaction(&self, transaction: &[u8]) -> Result<Vec<u8>> {
        Ok(transaction.to_vec())
    }

    fn sign_message(&self, message: &[u8]) -> Result<Vec<u8>> {
        Ok(message.to_vec())
    }
}
