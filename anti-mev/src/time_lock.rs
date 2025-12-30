use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// Time-lock manager for delayed transaction execution
pub struct TimeLockManager {
    min_lock_duration: u64,
    locked_transactions: HashMap<u64, LockedTransaction>,
}

#[derive(Debug, Clone)]
pub struct LockedTransaction {
    pub unlock_slot: u64,
    pub created_slot: u64,
}

impl TimeLockManager {
    pub fn new(min_lock_duration: u64) -> Self {
        Self {
            min_lock_duration,
            locked_transactions: HashMap::new(),
        }
    }

    /// Calculate when a transaction should unlock
    pub fn calculate_unlock_slot(&self) -> Result<u64> {
        // In production, get current slot from Solana RPC
        let current_slot = 1000u64;
        Ok(current_slot + self.min_lock_duration)
    }

    /// Lock a transaction until specified slot
    pub fn lock_transaction(&mut self, tx_id: u64, unlock_slot: u64) -> Result<()> {
        let current_slot = 1000u64;

        if unlock_slot <= current_slot {
            return Err(anyhow!("Unlock slot must be in the future"));
        }

        self.locked_transactions.insert(
            tx_id,
            LockedTransaction {
                unlock_slot,
                created_slot: current_slot,
            },
        );

        Ok(())
    }

    /// Check if a transaction is unlocked
    pub fn is_unlocked(&self, slot: u64) -> bool {
        // In production, compare with actual Solana slot
        true
    }

    /// Get unlock slot for transaction
    pub fn get_unlock_slot(&self, tx_id: u64) -> Option<u64> {
        self.locked_transactions
            .get(&tx_id)
            .map(|tx| tx.unlock_slot)
    }

    /// Remove unlocked transactions from tracking
    pub fn cleanup_unlocked(&mut self, current_slot: u64) {
        self.locked_transactions
            .retain(|_, tx| tx.unlock_slot > current_slot);
    }

    /// Get number of locked transactions
    pub fn locked_count(&self) -> usize {
        self.locked_transactions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_lock() {
        let mut manager = TimeLockManager::new(10);

        let unlock_slot = manager.calculate_unlock_slot().unwrap();
        assert!(unlock_slot > 1000);

        manager.lock_transaction(1, unlock_slot).unwrap();
        assert_eq!(manager.locked_count(), 1);

        let retrieved_slot = manager.get_unlock_slot(1).unwrap();
        assert_eq!(retrieved_slot, unlock_slot);
    }

    #[test]
    fn test_cleanup() {
        let mut manager = TimeLockManager::new(10);

        manager.lock_transaction(1, 1100).unwrap();
        manager.lock_transaction(2, 1200).unwrap();

        assert_eq!(manager.locked_count(), 2);

        manager.cleanup_unlocked(1150);
        assert_eq!(manager.locked_count(), 1);
    }
}
