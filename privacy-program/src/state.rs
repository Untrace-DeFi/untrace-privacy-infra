use anchor_lang::prelude::*;
use untrace_common::PrivacyLevel;

#[account]
pub struct PrivacyPoolAccount {
    pub pool_id: u64,
    pub commitment_root: [u8; 32],
    pub commitment_count: u64,
    pub min_pool_size: u64,
    pub authority: Pubkey,
}

#[account]
pub struct CommitmentAccount {
    pub commitment: [u8; 32],
    pub nullifier: [u8; 32],
    pub timestamp: i64,
    pub pool_id: u64,
}

#[account]
pub struct NullifierAccount {
    pub nullifier: [u8; 32],
    pub is_used: bool,
    pub timestamp: i64,
}

#[account]
pub struct PrivateTransferAccount {
    pub encrypted_amount: Vec<u8>,
    pub encrypted_recipient: Vec<u8>,
    pub zk_proof: Vec<u8>,
    pub privacy_level: PrivacyLevel,
    pub sender: Pubkey,
    pub timestamp: i64,
}

#[account]
pub struct CrossChainBridgeAccount {
    pub source_chain: u16,
    pub dest_chain: u16,
    pub encrypted_data: Vec<u8>,
    pub ephemeral_pubkey: [u8; 32],
    pub nonce: [u8; 12],
    pub tag: [u8; 16],
    pub sender: Pubkey,
    pub timestamp: i64,
    pub status: u8, // 0=pending, 1=completed, 2=failed
}

impl PrivacyPoolAccount {
    pub const LEN: usize = 8 + // discriminator
        8 + // pool_id
        32 + // commitment_root
        8 + // commitment_count
        8 + // min_pool_size
        32; // authority
}

impl CommitmentAccount {
    pub const LEN: usize = 8 + // discriminator
        32 + // commitment
        32 + // nullifier
        8 + // timestamp
        8; // pool_id
}

impl NullifierAccount {
    pub const LEN: usize = 8 + // discriminator
        32 + // nullifier
        1 + // is_used
        8; // timestamp
}
