//! Core module for the sniper bot.
//! 
//! This module provides core functionality shared across all sniper components.

pub mod types;
pub mod bus;
pub mod config;
pub mod errors;
pub mod env;
pub mod prelude;
pub mod cache;

use anyhow::Result;

/// Core functionality for the sniper bot
pub struct Core {
    // In a real implementation, this would contain core services
}

impl Core {
    /// Create a new core instance
    pub fn new() -> Self {
        Self {}
    }
    
    /// Initialize core services
    pub async fn init(&self) -> Result<()> {
        // Placeholder for core initialization
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_creation() {
        let core = Core::new();
        assert!(true); // Just testing that we can create a core instance
    }
    
    #[tokio::test]
    async fn test_core_initialization() -> Result<()> {
        let core = Core::new();
        core.init().await?;
        Ok(())
    }
}

#[cfg(test)]
mod phase3_tests {
    use super::*;
    use crate::cache::{Cache, AmmCache, AmmQuote};
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_caching_mechanisms() {
        // Test generic cache
        let cache: Cache<String, i32> = Cache::new(Duration::from_secs(1), 100);
        
        // Test insert and get
        cache.insert("key1".to_string(), 42).await.unwrap();
        let value = cache.get(&"key1".to_string()).await;
        assert_eq!(value, Some(42));
        
        // Test contains_key
        assert!(cache.contains_key(&"key1".to_string()).await);
        assert!(!cache.contains_key(&"key2".to_string()).await);
        
        // Test remove
        let removed = cache.remove(&"key1".to_string()).await.unwrap();
        assert_eq!(removed, Some(42));
        assert!(cache.get(&"key1".to_string()).await.is_none());
        
        // Test AMM cache
        let amm_cache = AmmCache::new();
        
        let quote = AmmQuote {
            expected_output: 1000000000000000000,
            price_impact: 0.5,
            gas_estimate: 150000,
            timestamp: 1234567890,
        };
        
        amm_cache.store_quote("quote-key".to_string(), quote.clone()).await.unwrap();
        let retrieved = amm_cache.get_quote("quote-key").await;
        assert_eq!(retrieved, Some(quote));
        
        let stats = amm_cache.stats().await;
        assert_eq!(stats.quotes_count, 1);
        
        println!("Caching mechanisms tests passed!");
    }

    #[tokio::test]
    async fn test_resource_usage_optimization() {
        // Test cache size limits and eviction
        let cache: Cache<i32, String> = Cache::new(Duration::from_secs(1), 2);
        
        cache.insert(1, "value1".to_string()).await.unwrap();
        cache.insert(2, "value2".to_string()).await.unwrap();
        cache.insert(3, "value3".to_string()).await.unwrap();
        
        // One entry should have been evicted due to size limit
        assert_eq!(cache.len().await, 2);
        
        // Test cache expiration
        let expiring_cache: Cache<String, i32> = Cache::new(Duration::from_millis(10), 100);
        
        expiring_cache.insert("key1".to_string(), 42).await.unwrap();
        assert!(expiring_cache.get(&"key1".to_string()).await.is_some());
        
        // Wait for expiration
        sleep(Duration::from_millis(20)).await;
        
        assert!(expiring_cache.get(&"key1".to_string()).await.is_none());
        
        // Test AMM cache eviction
        let amm_cache = AmmCache::new();
        
        // Add many entries to test eviction
        for i in 0..1005 {
            let quote = AmmQuote {
                expected_output: 1000000000000000000 + i as u128,
                price_impact: 0.1 + (i as f64 * 0.001),
                gas_estimate: 100000 + i,
                timestamp: 1234567890 + i,
            };
            
            amm_cache.store_quote(format!("quote-{}", i), quote).await.unwrap();
        }
        
        // Cache should respect size limits
        let stats = amm_cache.stats().await;
        assert!(stats.quotes_count <= 1000); // Max size is 1000
        
        println!("Resource usage optimization tests passed!");
    }
}