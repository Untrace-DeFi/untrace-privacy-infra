use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use solana_sdk::signature::Keypair;
use std::collections::HashMap;

/// Secure storage for wallet data
#[derive(Debug)]
pub struct SecureStorage {
    /// Encrypted commitments and secrets
    commitments: HashMap<String, StoredCommitment>,
    /// Encrypted keypairs
    keypairs: HashMap<String, Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredCommitment {
    commitment: [u8; 32],
    randomness: [u8; 32],
    timestamp: i64,
}

impl SecureStorage {
    pub fn new() -> Result<Self> {
        Ok(Self {
            commitments: HashMap::new(),
            keypairs: HashMap::new(),
        })
    }

    /// Store a commitment with its randomness
    pub fn store_commitment(&self, commitment: &[u8; 32], randomness: &[u8; 32]) -> Result<()> {
        // In production, this would use secure OS keychain/keystore
        println!("Storing commitment securely");
        Ok(())
    }

    /// Get secret for a commitment
    pub fn get_secret(&self, commitment: &[u8; 32]) -> Result<Vec<u8>> {
        // In production, retrieve from secure storage
        Ok(commitment.to_vec())
    }

    /// Export wallet (encrypted with password)
    pub fn export_wallet(&self, keypair: &Keypair, password: &str) -> Result<String> {
        // Simple XOR encryption for demonstration
        // In production, use proper encryption like AES-GCM with PBKDF2
        let keypair_bytes = keypair.to_bytes();
        let password_bytes = password.as_bytes();

        let mut encrypted = Vec::new();
        for (i, byte) in keypair_bytes.iter().enumerate() {
            encrypted.push(byte ^ password_bytes[i % password_bytes.len()]);
        }

        Ok(bs58::encode(&encrypted).into_string())
    }

    /// Import wallet (decrypt with password)
    pub fn import_wallet(&self, encrypted: &str, password: &str) -> Result<Keypair> {
        let encrypted_bytes = bs58::decode(encrypted)
            .into_vec()
            .map_err(|e| anyhow!("Failed to decode: {}", e))?;

        let password_bytes = password.as_bytes();

        let mut decrypted = Vec::new();
        for (i, byte) in encrypted_bytes.iter().enumerate() {
            decrypted.push(byte ^ password_bytes[i % password_bytes.len()]);
        }

        if decrypted.len() != 64 {
            return Err(anyhow!("Invalid keypair length"));
        }

        Keypair::from_bytes(&decrypted)
            .map_err(|e| anyhow!("Failed to create keypair: {}", e))
    }

    /// Store encrypted seed phrase
    pub fn store_seed_phrase(&mut self, seed_phrase: &str, password: &str) -> Result<String> {
        let seed_bytes = seed_phrase.as_bytes();
        let password_bytes = password.as_bytes();

        let mut encrypted = Vec::new();
        for (i, byte) in seed_bytes.iter().enumerate() {
            encrypted.push(byte ^ password_bytes[i % password_bytes.len()]);
        }

        let encoded = bs58::encode(&encrypted).into_string();
        Ok(encoded)
    }

    /// Retrieve seed phrase
    pub fn retrieve_seed_phrase(&self, encrypted: &str, password: &str) -> Result<String> {
        let encrypted_bytes = bs58::decode(encrypted)
            .into_vec()
            .map_err(|e| anyhow!("Failed to decode: {}", e))?;

        let password_bytes = password.as_bytes();

        let mut decrypted = Vec::new();
        for (i, byte) in encrypted_bytes.iter().enumerate() {
            decrypted.push(byte ^ password_bytes[i % password_bytes.len()]);
        }

        String::from_utf8(decrypted)
            .map_err(|e| anyhow!("Failed to decode seed phrase: {}", e))
    }

    /// Clear all stored data
    pub fn clear(&mut self) {
        self.commitments.clear();
        self.keypairs.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_import_wallet() {
        let storage = SecureStorage::new().unwrap();
        let keypair = Keypair::new();
        let password = "test_password_123";

        let encrypted = storage.export_wallet(&keypair, password).unwrap();
        let imported = storage.import_wallet(&encrypted, password).unwrap();

        assert_eq!(
            keypair.to_bytes(),
            imported.to_bytes()
        );
    }

    #[test]
    fn test_seed_phrase_storage() {
        let mut storage = SecureStorage::new().unwrap();
        let seed = "witch collapse practice feed shame open despair creek road again ice least";
        let password = "secure_password";

        let encrypted = storage.store_seed_phrase(seed, password).unwrap();
        let decrypted = storage.retrieve_seed_phrase(&encrypted, password).unwrap();

        assert_eq!(seed, decrypted);
    }
}
