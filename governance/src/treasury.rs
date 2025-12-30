use anyhow::{anyhow, Result};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

/// Treasury management for protocol funds
pub struct Treasury {
    /// Total funds in treasury (lamports)
    balance: u64,
    /// Fee settings
    fee_config: FeeConfig,
    /// Revenue tracking
    revenue: RevenueTracker,
    /// Allocation records
    allocations: HashMap<u64, Allocation>,
    /// Next allocation ID
    next_allocation_id: u64,
}

#[derive(Debug, Clone)]
pub struct FeeConfig {
    /// Transaction fee (basis points, 1 bp = 0.01%)
    pub transaction_fee_bp: u16,
    /// Bridge fee (basis points)
    pub bridge_fee_bp: u16,
    /// Privacy pool fee (basis points)
    pub pool_fee_bp: u16,
    /// Fee recipient
    pub fee_recipient: Pubkey,
}

impl Default for FeeConfig {
    fn default() -> Self {
        Self {
            transaction_fee_bp: 30,  // 0.3%
            bridge_fee_bp: 50,        // 0.5%
            pool_fee_bp: 20,          // 0.2%
            fee_recipient: Pubkey::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RevenueTracker {
    /// Total fees collected
    pub total_fees: u64,
    /// Fees by type
    pub transaction_fees: u64,
    pub bridge_fees: u64,
    pub pool_fees: u64,
}

#[derive(Debug, Clone)]
pub struct Allocation {
    pub id: u64,
    pub recipient: Pubkey,
    pub amount: u64,
    pub purpose: String,
    pub timestamp: i64,
    pub executed: bool,
}

impl Treasury {
    pub fn new() -> Self {
        Self {
            balance: 0,
            fee_config: FeeConfig::default(),
            revenue: RevenueTracker {
                total_fees: 0,
                transaction_fees: 0,
                bridge_fees: 0,
                pool_fees: 0,
            },
            allocations: HashMap::new(),
            next_allocation_id: 1,
        }
    }

    /// Deposit funds to treasury
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        self.balance += amount;
        Ok(())
    }

    /// Calculate transaction fee
    pub fn calculate_transaction_fee(&self, amount: u64) -> u64 {
        (amount * self.fee_config.transaction_fee_bp as u64) / 10_000
    }

    /// Calculate bridge fee
    pub fn calculate_bridge_fee(&self, amount: u64) -> u64 {
        (amount * self.fee_config.bridge_fee_bp as u64) / 10_000
    }

    /// Calculate pool fee
    pub fn calculate_pool_fee(&self, amount: u64) -> u64 {
        (amount * self.fee_config.pool_fee_bp as u64) / 10_000
    }

    /// Collect fee
    pub fn collect_fee(&mut self, amount: u64, fee_type: FeeType) -> Result<()> {
        self.balance += amount;
        self.revenue.total_fees += amount;

        match fee_type {
            FeeType::Transaction => self.revenue.transaction_fees += amount,
            FeeType::Bridge => self.revenue.bridge_fees += amount,
            FeeType::Pool => self.revenue.pool_fees += amount,
        }

        Ok(())
    }

    /// Create a fund allocation
    pub fn create_allocation(
        &mut self,
        recipient: Pubkey,
        amount: u64,
        purpose: String,
    ) -> Result<u64> {
        if amount > self.balance {
            return Err(anyhow!("Insufficient treasury balance"));
        }

        let allocation = Allocation {
            id: self.next_allocation_id,
            recipient,
            amount,
            purpose,
            timestamp: Self::current_timestamp(),
            executed: false,
        };

        self.allocations.insert(self.next_allocation_id, allocation);
        self.next_allocation_id += 1;

        Ok(self.next_allocation_id - 1)
    }

    /// Execute an allocation
    pub fn execute_allocation(&mut self, allocation_id: u64) -> Result<()> {
        let allocation = self.allocations
            .get_mut(&allocation_id)
            .ok_or_else(|| anyhow!("Allocation not found"))?;

        if allocation.executed {
            return Err(anyhow!("Allocation already executed"));
        }

        if allocation.amount > self.balance {
            return Err(anyhow!("Insufficient balance"));
        }

        self.balance -= allocation.amount;
        allocation.executed = true;

        Ok(())
    }

    /// Get treasury balance
    pub fn balance(&self) -> u64 {
        self.balance
    }

    /// Get revenue statistics
    pub fn revenue_stats(&self) -> &RevenueTracker {
        &self.revenue
    }

    /// Update fee configuration
    pub fn update_fees(&mut self, new_config: FeeConfig) -> Result<()> {
        // Validate fee config
        if new_config.transaction_fee_bp > 1000
            || new_config.bridge_fee_bp > 1000
            || new_config.pool_fee_bp > 1000
        {
            return Err(anyhow!("Fees cannot exceed 10%"));
        }

        self.fee_config = new_config;
        Ok(())
    }

    /// Get pending allocations
    pub fn pending_allocations(&self) -> Vec<&Allocation> {
        self.allocations
            .values()
            .filter(|a| !a.executed)
            .collect()
    }

    fn current_timestamp() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FeeType {
    Transaction,
    Bridge,
    Pool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_treasury_deposit() {
        let mut treasury = Treasury::new();

        treasury.deposit(1_000_000).unwrap();
        assert_eq!(treasury.balance(), 1_000_000);
    }

    #[test]
    fn test_fee_calculation() {
        let treasury = Treasury::new();

        let amount = 1_000_000;
        let tx_fee = treasury.calculate_transaction_fee(amount);
        assert_eq!(tx_fee, 3_000); // 0.3% of 1M

        let bridge_fee = treasury.calculate_bridge_fee(amount);
        assert_eq!(bridge_fee, 5_000); // 0.5% of 1M
    }

    #[test]
    fn test_allocation() {
        let mut treasury = Treasury::new();
        treasury.deposit(1_000_000).unwrap();

        let recipient = Pubkey::new_unique();
        let allocation_id = treasury.create_allocation(
            recipient,
            500_000,
            "Development grant".to_string(),
        ).unwrap();

        assert_eq!(allocation_id, 1);

        treasury.execute_allocation(allocation_id).unwrap();
        assert_eq!(treasury.balance(), 500_000);
    }
}
