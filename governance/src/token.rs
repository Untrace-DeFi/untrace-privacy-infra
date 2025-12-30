use anyhow::{anyhow, Result};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

/// Governance token for voting and fees
pub struct GovernanceToken {
    /// Total supply
    total_supply: u64,
    /// Circulating supply
    circulating_supply: u64,
    /// Token balances
    balances: HashMap<Pubkey, u64>,
    /// Token metadata
    metadata: TokenMetadata,
}

#[derive(Debug, Clone)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

impl GovernanceToken {
    pub fn new(total_supply: u64) -> Self {
        Self {
            total_supply,
            circulating_supply: 0,
            balances: HashMap::new(),
            metadata: TokenMetadata {
                name: "Untrace Governance Token".to_string(),
                symbol: "UNT".to_string(),
                decimals: 9,
            },
        }
    }

    /// Mint tokens to an address
    pub fn mint(&mut self, to: Pubkey, amount: u64) -> Result<()> {
        if self.circulating_supply + amount > self.total_supply {
            return Err(anyhow!("Would exceed total supply"));
        }

        let balance = self.balances.entry(to).or_insert(0);
        *balance += amount;
        self.circulating_supply += amount;

        Ok(())
    }

    /// Transfer tokens
    pub fn transfer(&mut self, from: Pubkey, to: Pubkey, amount: u64) -> Result<()> {
        let from_balance = self.balances.get_mut(&from)
            .ok_or_else(|| anyhow!("From address has no balance"))?;

        if *from_balance < amount {
            return Err(anyhow!("Insufficient balance"));
        }

        *from_balance -= amount;

        let to_balance = self.balances.entry(to).or_insert(0);
        *to_balance += amount;

        Ok(())
    }

    /// Get balance of address
    pub fn balance_of(&self, address: &Pubkey) -> u64 {
        self.balances.get(address).copied().unwrap_or(0)
    }

    /// Get total supply
    pub fn total_supply(&self) -> u64 {
        self.total_supply
    }

    /// Get circulating supply
    pub fn circulating_supply(&self) -> u64 {
        self.circulating_supply
    }

    /// Burn tokens
    pub fn burn(&mut self, from: Pubkey, amount: u64) -> Result<()> {
        let balance = self.balances.get_mut(&from)
            .ok_or_else(|| anyhow!("Address has no balance"))?;

        if *balance < amount {
            return Err(anyhow!("Insufficient balance to burn"));
        }

        *balance -= amount;
        self.circulating_supply -= amount;

        Ok(())
    }

    /// Get token metadata
    pub fn metadata(&self) -> &TokenMetadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_mint() {
        let mut token = GovernanceToken::new(1_000_000_000);

        let address = Pubkey::new_unique();
        token.mint(address, 1_000_000).unwrap();

        assert_eq!(token.balance_of(&address), 1_000_000);
        assert_eq!(token.circulating_supply(), 1_000_000);
    }

    #[test]
    fn test_token_transfer() {
        let mut token = GovernanceToken::new(1_000_000_000);

        let from = Pubkey::new_unique();
        let to = Pubkey::new_unique();

        token.mint(from, 1_000_000).unwrap();
        token.transfer(from, to, 500_000).unwrap();

        assert_eq!(token.balance_of(&from), 500_000);
        assert_eq!(token.balance_of(&to), 500_000);
    }

    #[test]
    fn test_token_burn() {
        let mut token = GovernanceToken::new(1_000_000_000);

        let address = Pubkey::new_unique();
        token.mint(address, 1_000_000).unwrap();
        token.burn(address, 300_000).unwrap();

        assert_eq!(token.balance_of(&address), 700_000);
        assert_eq!(token.circulating_supply(), 700_000);
    }
}
