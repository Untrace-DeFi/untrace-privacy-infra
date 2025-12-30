use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use serde::{Deserialize, Serialize};

pub mod crypto;
pub mod error;

pub use error::UntraceError;

/// Privacy levels supported by the protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum PrivacyLevel {
    /// Basic privacy - transaction amounts hidden
    Basic,
    /// Enhanced privacy - amounts and recipient hidden
    Enhanced,
    /// Maximum privacy - full transaction obfuscation with ZK proofs
    Maximum,
}

/// Encrypted transaction data
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct EncryptedTransaction {
    /// Encrypted payload
    pub ciphertext: Vec<u8>,
    /// Ephemeral public key for decryption
    pub ephemeral_pubkey: [u8; 32],
    /// Nonce for encryption
    pub nonce: [u8; 12],
    /// Authentication tag
    pub tag: [u8; 16],
}

/// Private transfer instruction data
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct PrivateTransfer {
    /// Amount (encrypted)
    pub encrypted_amount: Vec<u8>,
    /// Recipient (encrypted)
    pub encrypted_recipient: Vec<u8>,
    /// Zero-knowledge proof of valid transfer
    pub zk_proof: Vec<u8>,
    /// Privacy level
    pub privacy_level: PrivacyLevel,
}

/// Cross-chain bridge data
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct CrossChainTransfer {
    /// Source chain identifier
    pub source_chain: u16,
    /// Destination chain identifier
    pub dest_chain: u16,
    /// Encrypted transfer data
    pub encrypted_data: EncryptedTransaction,
    /// Privacy merkle root
    pub merkle_root: [u8; 32],
}

/// Privacy pool for mixing transactions
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct PrivacyPool {
    /// Pool identifier
    pub pool_id: u64,
    /// Merkle tree root of commitments
    pub commitment_root: [u8; 32],
    /// Number of commitments in pool
    pub commitment_count: u64,
    /// Minimum pool size before withdrawals
    pub min_pool_size: u64,
}

/// Commitment for privacy pool
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Commitment {
    /// Commitment hash
    pub commitment: [u8; 32],
    /// Nullifier hash (spent indicator)
    pub nullifier: [u8; 32],
    /// Timestamp
    pub timestamp: i64,
}

/// Governance proposal
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Proposal {
    /// Proposal ID
    pub id: u64,
    /// Proposer public key
    pub proposer: Pubkey,
    /// Proposal description hash
    pub description_hash: [u8; 32],
    /// Voting start time
    pub start_time: i64,
    /// Voting end time
    pub end_time: i64,
    /// Yes votes
    pub yes_votes: u64,
    /// No votes
    pub no_votes: u64,
    /// Proposal status
    pub status: ProposalStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum ProposalStatus {
    Active,
    Passed,
    Failed,
    Executed,
}

/// Anti-MEV configuration
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct AntiMevConfig {
    /// Time-locked transactions enabled
    pub time_lock_enabled: bool,
    /// Minimum time lock duration (slots)
    pub min_time_lock: u64,
    /// Transaction batching enabled
    pub batching_enabled: bool,
    /// Batch size
    pub batch_size: u32,
}
