use thiserror::Error;
use solana_program::program_error::ProgramError;

#[derive(Error, Debug)]
pub enum UntraceError {
    #[error("Invalid instruction")]
    InvalidInstruction,

    #[error("Invalid privacy level")]
    InvalidPrivacyLevel,

    #[error("Encryption failed")]
    EncryptionFailed,

    #[error("Decryption failed")]
    DecryptionFailed,

    #[error("Invalid zero-knowledge proof")]
    InvalidZKProof,

    #[error("Insufficient pool size")]
    InsufficientPoolSize,

    #[error("Commitment already exists")]
    CommitmentExists,

    #[error("Nullifier already used")]
    NullifierUsed,

    #[error("Invalid merkle proof")]
    InvalidMerkleProof,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Proposal not found")]
    ProposalNotFound,

    #[error("Voting period ended")]
    VotingEnded,

    #[error("Already voted")]
    AlreadyVoted,

    #[error("MEV protection violated")]
    MevProtectionViolated,

    #[error("Time lock not expired")]
    TimeLockNotExpired,
}

impl From<UntraceError> for ProgramError {
    fn from(e: UntraceError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
