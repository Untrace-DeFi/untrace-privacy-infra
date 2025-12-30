use anyhow::{anyhow, Result};
use solana_sdk::pubkey::Pubkey;
use std::collections::{HashMap, HashSet};

/// Voting system for governance proposals
pub struct VotingSystem {
    /// Voting period duration (seconds)
    voting_period: i64,
    /// Minimum votes required (quorum)
    quorum_threshold: u64,
    /// Vote records per proposal
    votes: HashMap<u64, ProposalVotes>,
    /// Vote delegation
    delegations: HashMap<Pubkey, Pubkey>,
    /// Voting power cache
    voting_power: HashMap<Pubkey, u64>,
}

#[derive(Debug, Clone)]
pub struct ProposalVotes {
    /// Addresses that voted yes
    yes_voters: HashSet<Pubkey>,
    /// Addresses that voted no
    no_voters: HashSet<Pubkey>,
    /// Total yes votes
    yes_count: u64,
    /// Total no votes
    no_count: u64,
}

impl VotingSystem {
    pub fn new(voting_period: i64, quorum_threshold: u64) -> Self {
        Self {
            voting_period,
            quorum_threshold,
            votes: HashMap::new(),
            delegations: HashMap::new(),
            voting_power: HashMap::new(),
        }
    }

    /// Cast a vote on a proposal
    pub fn cast_vote(
        &mut self,
        proposal_id: u64,
        voter: Pubkey,
        voting_power: u64,
        vote_yes: bool,
    ) -> Result<()> {
        let votes = self.votes.entry(proposal_id).or_insert(ProposalVotes {
            yes_voters: HashSet::new(),
            no_voters: HashSet::new(),
            yes_count: 0,
            no_count: 0,
        });

        // Check if already voted
        if votes.yes_voters.contains(&voter) || votes.no_voters.contains(&voter) {
            return Err(anyhow!("Already voted"));
        }

        // Apply delegation if exists
        let effective_voter = self.delegations.get(&voter).copied().unwrap_or(voter);

        if vote_yes {
            votes.yes_voters.insert(effective_voter);
            votes.yes_count += voting_power;
        } else {
            votes.no_voters.insert(effective_voter);
            votes.no_count += voting_power;
        }

        Ok(())
    }

    /// Check if a proposal has passed
    pub fn has_passed(
        &self,
        proposal_id: u64,
        yes_votes: u64,
        no_votes: u64,
    ) -> Result<bool> {
        let total_votes = yes_votes + no_votes;

        // Check quorum
        if total_votes < self.quorum_threshold {
            return Ok(false);
        }

        // Simple majority
        Ok(yes_votes > no_votes)
    }

    /// Delegate voting power to another address
    pub fn delegate(
        &mut self,
        delegator: Pubkey,
        delegatee: Pubkey,
        voting_power: u64,
    ) -> Result<()> {
        self.delegations.insert(delegator, delegatee);

        // Update voting power
        let delegatee_power = self.voting_power.entry(delegatee).or_insert(0);
        *delegatee_power += voting_power;

        Ok(())
    }

    /// Remove delegation
    pub fn undelegate(&mut self, delegator: Pubkey) -> Result<()> {
        if let Some(delegatee) = self.delegations.remove(&delegator) {
            // Could update voting power here
            Ok(())
        } else {
            Err(anyhow!("No delegation found"))
        }
    }

    /// Get voting power for an address (including delegations)
    pub fn get_voting_power(&self, address: &Pubkey) -> u64 {
        self.voting_power.get(address).copied().unwrap_or(0)
    }

    /// Get vote statistics for a proposal
    pub fn get_vote_stats(&self, proposal_id: u64) -> Option<VoteStats> {
        self.votes.get(&proposal_id).map(|votes| VoteStats {
            yes_votes: votes.yes_count,
            no_votes: votes.no_count,
            yes_voters: votes.yes_voters.len(),
            no_voters: votes.no_voters.len(),
            total_votes: votes.yes_count + votes.no_count,
            participation_rate: self.calculate_participation(votes),
        })
    }

    fn calculate_participation(&self, votes: &ProposalVotes) -> f64 {
        let total_votes = votes.yes_count + votes.no_count;
        if self.quorum_threshold == 0 {
            return 0.0;
        }
        (total_votes as f64 / self.quorum_threshold as f64) * 100.0
    }
}

#[derive(Debug, Clone)]
pub struct VoteStats {
    pub yes_votes: u64,
    pub no_votes: u64,
    pub yes_voters: usize,
    pub no_voters: usize,
    pub total_votes: u64,
    pub participation_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voting() {
        let mut voting = VotingSystem::new(86400, 100_000_000);

        let voter1 = Pubkey::new_unique();
        let voter2 = Pubkey::new_unique();

        voting.cast_vote(1, voter1, 50_000_000, true).unwrap();
        voting.cast_vote(1, voter2, 60_000_000, false).unwrap();

        let passed = voting.has_passed(1, 50_000_000, 60_000_000).unwrap();
        assert!(!passed);

        let stats = voting.get_vote_stats(1).unwrap();
        assert_eq!(stats.yes_votes, 50_000_000);
        assert_eq!(stats.no_votes, 60_000_000);
    }

    #[test]
    fn test_delegation() {
        let mut voting = VotingSystem::new(86400, 100_000_000);

        let delegator = Pubkey::new_unique();
        let delegatee = Pubkey::new_unique();

        voting.delegate(delegator, delegatee, 10_000_000).unwrap();

        assert_eq!(voting.get_voting_power(&delegatee), 10_000_000);
    }
}
