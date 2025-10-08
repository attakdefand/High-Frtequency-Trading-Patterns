//! Nonce management for transaction sequencing
//! 
//! This module provides functionality for managing transaction nonces
//! to ensure proper sequencing and avoid nonce gaps.

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Nonce manager for tracking account nonces
pub struct NonceManager {
    // Map of address to current nonce
    nonces: Arc<RwLock<HashMap<String, u64>>>,
}

impl NonceManager {
    /// Create a new nonce manager
    pub fn new() -> Self {
        Self {
            nonces: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Get the next nonce for an address
    pub async fn get_next_nonce(&self, address: &str) -> Result<u64> {
        let mut nonces = self.nonces.write().await;
        let nonce = nonces.entry(address.to_string()).or_insert(0);
        let current = *nonce;
        *nonce += 1;
        Ok(current)
    }
    
    /// Reset nonce for an address (useful after reorgs or errors)
    pub async fn reset_nonce(&self, address: &str, nonce: u64) -> Result<()> {
        let mut nonces = self.nonces.write().await;
        nonces.insert(address.to_string(), nonce);
        Ok(())
    }
    
    /// Get current nonce without incrementing
    pub async fn get_current_nonce(&self, address: &str) -> Result<u64> {
        let nonces = self.nonces.read().await;
        Ok(*nonces.get(address).unwrap_or(&0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nonce_management() -> Result<()> {
        let manager = NonceManager::new();
        let address = "0xTestAddress";
        
        // Test getting next nonce
        let nonce1 = manager.get_next_nonce(address).await?;
        assert_eq!(nonce1, 0);
        
        let nonce2 = manager.get_next_nonce(address).await?;
        assert_eq!(nonce2, 1);
        
        // Test getting current nonce without incrementing
        let current = manager.get_current_nonce(address).await?;
        assert_eq!(current, 2);
        
        // Test resetting nonce
        manager.reset_nonce(address, 5).await?;
        let current = manager.get_current_nonce(address).await?;
        assert_eq!(current, 5);
        
        Ok(())
    }
}