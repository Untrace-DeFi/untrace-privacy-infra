use anyhow::{anyhow, Result};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use untrace_common::{Proposal, ProposalStatus};

pub mod token;
pub mod voting;
pub mod treasury;

pub use token::GovernanceToken;
pub use voting::VotingSystem;
pub use treasury::Treasury;

/// Decentralized governance system for Untrace protocol
pub struct GovernanceSystem {
    /// Governance token
    token: GovernanceToken,
    /// Voting system
    voting: VotingSystem,
    /// Treasury management
    treasury: Treasury,
    /// Active proposals
    proposals: HashMap<u64, Proposal>,
    /// Next proposal ID
    next_proposal_id: u64,
}

impl GovernanceSystem {
    pub fn new(
        token_supply: u64,
        voting_period: i64,
        quorum_threshold: u64,
    ) -> Self {
        Self {
            token: GovernanceToken::new(token_supply),
            voting: VotingSystem::new(voting_period, quorum_threshold),
            treasury: Treasury::new(),
            proposals: HashMap::new(),
            next_proposal_id: 1,
        }
    }

    /// Create a new governance proposal
    pub fn create_proposal(
        &mut self,
        proposer: Pubkey,
        description: String,
        start_time: i64,
        end_time: i64,
    ) -> Result<u64> {
        // Check proposer has minimum token balance
        let min_tokens = 1_000_000; // 1M tokens to propose
        if self.token.balance_of(&proposer) < min_tokens {
            return Err(anyhow!("Insufficient tokens to create proposal"));
        }

        let description_hash = Self::hash_description(&description);

        let proposal = Proposal {
            id: self.next_proposal_id,
            proposer,
            description_hash,
            start_time,
            end_time,
            yes_votes: 0,
            no_votes: 0,
            status: ProposalStatus::Active,
        };

        self.proposals.insert(self.next_proposal_id, proposal);
        self.next_proposal_id += 1;

        Ok(self.next_proposal_id - 1)
    }

    /// Vote on a proposal
    pub fn vote(
        &mut self,
        proposal_id: u64,
        voter: Pubkey,
        vote_yes: bool,
    ) -> Result<()> {
        let proposal = self.proposals
            .get_mut(&proposal_id)
            .ok_or_else(|| anyhow!("Proposal not found"))?;

        // Check proposal is active
        if proposal.status != ProposalStatus::Active {
            return Err(anyhow!("Proposal is not active"));
        }

        // Get voter's token balance (voting power)
        let voting_power = self.token.balance_of(&voter);

        if voting_power == 0 {
            return Err(anyhow!("No voting power"));
        }

        // Cast vote
        self.voting.cast_vote(proposal_id, voter, voting_power, vote_yes)?;

        // Update proposal vote counts
        if vote_yes {
            proposal.yes_votes += voting_power;
        } else {
            proposal.no_votes += voting_power;
        }

        Ok(())
    }

    /// Execute a proposal if it passed
    pub fn execute_proposal(&mut self, proposal_id: u64) -> Result<()> {
        let proposal = self.proposals
            .get_mut(&proposal_id)
            .ok_or_else(|| anyhow!("Proposal not found"))?;

        // Check if voting period ended
        let current_time = Self::current_timestamp();
        if current_time < proposal.end_time {
            return Err(anyhow!("Voting period not ended"));
        }

        // Check if proposal passed
        if !self.voting.has_passed(proposal_id, proposal.yes_votes, proposal.no_votes)? {
            proposal.status = ProposalStatus::Failed;
            return Err(anyhow!("Proposal did not pass"));
        }

        proposal.status = ProposalStatus::Executed;

        Ok(())
    }

    /// Get proposal details
    pub fn get_proposal(&self, proposal_id: u64) -> Option<&Proposal> {
        self.proposals.get(&proposal_id)
    }

    /// Get all active proposals
    pub fn get_active_proposals(&self) -> Vec<&Proposal> {
        self.proposals
            .values()
            .filter(|p| p.status == ProposalStatus::Active)
            .collect()
    }

    /// Delegate voting power
    pub fn delegate_votes(&mut self, delegator: Pubkey, delegatee: Pubkey) -> Result<()> {
        let voting_power = self.token.balance_of(&delegator);
        self.voting.delegate(delegator, delegatee, voting_power)
    }

    /// Get voting power for an address
    pub fn get_voting_power(&self, address: &Pubkey) -> u64 {
        self.voting.get_voting_power(address)
    }

    fn hash_description(description: &str) -> [u8; 32] {
        use sha3::{Digest, Sha3_256};
        let mut hasher = Sha3_256::new();
        hasher.update(description.as_bytes());
        let result = hasher.finalize();

        let mut output = [0u8; 32];
        output.copy_from_slice(&result);
        output
    }

    fn current_timestamp() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_governance_system() {
        let mut gov = GovernanceSystem::new(
            1_000_000_000, // 1B token supply
            86400,         // 24 hour voting period
            100_000_000,   // 100M quorum
        );

        let proposer = Pubkey::new_unique();

        // Mint tokens to proposer
        gov.token.mint(proposer, 10_000_000).unwrap();

        let proposal_id = gov.create_proposal(
            proposer,
            "Test proposal".to_string(),
            0,
            86400,
        ).unwrap();

        assert_eq!(proposal_id, 1);

        let proposal = gov.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.proposer, proposer);
    }

    #[test]
    fn test_voting() {
        let mut gov = GovernanceSystem::new(
            1_000_000_000,
            86400,
            100_000_000,
        );

        let proposer = Pubkey::new_unique();
        let voter = Pubkey::new_unique();

        gov.token.mint(proposer, 10_000_000).unwrap();
        gov.token.mint(voter, 50_000_000).unwrap();

        let proposal_id = gov.create_proposal(
            proposer,
            "Test proposal".to_string(),
            0,
            86400,
        ).unwrap();

        gov.vote(proposal_id, voter, true).unwrap();

        let proposal = gov.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.yes_votes, 50_000_000);
    }
}
