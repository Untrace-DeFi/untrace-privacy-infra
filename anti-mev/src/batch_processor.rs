use anyhow::Result;
use solana_sdk::instruction::Instruction;
use std::collections::VecDeque;

/// Batch processor for grouping transactions
pub struct BatchProcessor {
    batch_size: u32,
    current_batch: Vec<Instruction>,
    batch_queue: VecDeque<Batch>,
    next_batch_id: u64,
}

#[derive(Debug, Clone)]
pub struct Batch {
    pub id: u64,
    pub instructions: Vec<Instruction>,
    pub created_at: u64,
}

impl BatchProcessor {
    pub fn new(batch_size: u32) -> Self {
        Self {
            batch_size,
            current_batch: Vec::new(),
            batch_queue: VecDeque::new(),
            next_batch_id: 1,
        }
    }

    /// Add instruction to current batch
    pub fn add_to_batch(&mut self, instruction: Instruction) -> Result<()> {
        self.current_batch.push(instruction);

        // If batch is full, seal it and create new batch
        if self.current_batch.len() >= self.batch_size as usize {
            self.seal_batch()?;
        }

        Ok(())
    }

    /// Seal current batch and move to queue
    fn seal_batch(&mut self) -> Result<()> {
        if self.current_batch.is_empty() {
            return Ok(());
        }

        let batch = Batch {
            id: self.next_batch_id,
            instructions: std::mem::take(&mut self.current_batch),
            created_at: Self::current_timestamp(),
        };

        self.batch_queue.push_back(batch);
        self.next_batch_id += 1;

        Ok(())
    }

    /// Process next batch in queue
    pub async fn process_batch(&mut self) -> Result<Vec<Instruction>> {
        if let Some(batch) = self.batch_queue.pop_front() {
            Ok(batch.instructions)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get current batch ID
    pub fn current_batch_id(&self) -> u64 {
        self.next_batch_id
    }

    /// Get number of pending batches
    pub fn pending_batches(&self) -> usize {
        self.batch_queue.len()
    }

    /// Get current batch size
    pub fn current_batch_size(&self) -> usize {
        self.current_batch.len()
    }

    /// Force seal current batch (even if not full)
    pub fn force_seal(&mut self) -> Result<()> {
        self.seal_batch()
    }

    /// Shuffle instructions in batch for privacy
    pub fn shuffle_batch(&mut self) {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        self.current_batch.shuffle(&mut rng);
    }

    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;

    fn create_dummy_instruction() -> Instruction {
        Instruction::new_with_bytes(
            Pubkey::new_unique(),
            &[1, 2, 3],
            vec![],
        )
    }

    #[test]
    fn test_batch_processor() {
        let mut processor = BatchProcessor::new(3);

        processor.add_to_batch(create_dummy_instruction()).unwrap();
        assert_eq!(processor.current_batch_size(), 1);

        processor.add_to_batch(create_dummy_instruction()).unwrap();
        processor.add_to_batch(create_dummy_instruction()).unwrap();

        // Batch should be sealed automatically
        assert_eq!(processor.current_batch_size(), 0);
        assert_eq!(processor.pending_batches(), 1);
    }

    #[test]
    fn test_force_seal() {
        let mut processor = BatchProcessor::new(10);

        processor.add_to_batch(create_dummy_instruction()).unwrap();
        processor.add_to_batch(create_dummy_instruction()).unwrap();

        assert_eq!(processor.pending_batches(), 0);

        processor.force_seal().unwrap();
        assert_eq!(processor.pending_batches(), 1);
    }
}
