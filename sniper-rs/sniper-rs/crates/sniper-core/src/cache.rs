//! Caching mechanisms for frequently accessed data
//! 
//! This module provides functionality for caching data to improve performance
//! and reduce redundant computations.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Cache entry with expiration
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: Instant,
    pub ttl: Duration,
}

impl<T> CacheEntry<T> {
    /// Create a new cache entry
    pub fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            created_at: Instant::now(),
            ttl,
        }
    }
    
    /// Check if the entry has expired
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// Generic cache implementation
pub struct Cache<K, V> {
    entries: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    default_ttl: Duration,
    max_size: usize,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Create a new cache with default TTL and max size
    pub fn new(default_ttl: Duration, max_size: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
            max_size,
        }
    }
    
    /// Get a value from the cache
    pub async fn get(&self, key: &K) -> Option<V> {
        let entries = self.entries.read().await;
        if let Some(entry) = entries.get(key) {
            if !entry.is_expired() {
                Some(entry.value.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Insert a value into the cache
    pub async fn insert(&self, key: K, value: V) -> Result<()> {
        self.insert_with_ttl(key, value, self.default_ttl).await
    }
    
    /// Insert a value into the cache with custom TTL
    pub async fn insert_with_ttl(&self, key: K, value: V, ttl: Duration) -> Result<()> {
        let mut entries = self.entries.write().await;
        
        // Evict oldest entries if we're at max size
        if entries.len() >= self.max_size {
            self.evict_oldest(&mut entries).await;
        }
        
        entries.insert(key, CacheEntry::new(value, ttl));
        Ok(())
    }
    
    /// Remove a value from the cache
    pub async fn remove(&self, key: &K) -> Result<Option<V>> {
        let mut entries = self.entries.write().await;
        Ok(entries.remove(key).map(|entry| entry.value))
    }
    
    /// Check if a key exists in the cache
    pub async fn contains_key(&self, key: &K) -> bool {
        let entries = self.entries.read().await;
        entries.contains_key(key)
    }
    
    /// Get the number of entries in the cache
    pub async fn len(&self) -> usize {
        let entries = self.entries.read().await;
        entries.len()
    }
    
    /// Check if the cache is empty
    pub async fn is_empty(&self) -> bool {
        let entries = self.entries.read().await;
        entries.is_empty()
    }
    
    /// Clear all entries from the cache
    pub async fn clear(&self) -> Result<()> {
        let mut entries = self.entries.write().await;
        entries.clear();
        Ok(())
    }
    
    /// Evict expired entries
    pub async fn evict_expired(&self) -> Result<usize> {
        let mut entries = self.entries.write().await;
        let initial_size = entries.len();
        entries.retain(|_, entry| !entry.is_expired());
        Ok(initial_size - entries.len())
    }
    
    /// Evict the oldest entry
    async fn evict_oldest(&self, entries: &mut HashMap<K, CacheEntry<V>>) {
        if let Some(oldest_key) = entries
            .iter()
            .min_by_key(|(_, entry)| entry.created_at)
            .map(|(key, _)| key.clone())
        {
            entries.remove(&oldest_key);
        }
    }
}

/// Specialized cache for AMM quotes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AmmQuote {
    pub expected_output: u128,
    pub price_impact: f64,
    pub gas_estimate: u64,
    pub timestamp: u64,
}

/// Cache for frequently accessed AMM data
pub struct AmmCache {
    quotes: Cache<String, AmmQuote>,
    routes: Cache<String, Vec<String>>,
}

impl AmmCache {
    /// Create a new AMM cache
    pub fn new() -> Self {
        Self {
            quotes: Cache::new(Duration::from_secs(30), 1000), // 30 second TTL, max 1000 entries
            routes: Cache::new(Duration::from_secs(60), 500),  // 60 second TTL, max 500 entries
        }
    }
    
    /// Get a quote from the cache
    pub async fn get_quote(&self, key: &str) -> Option<AmmQuote> {
        self.quotes.get(&key.to_string()).await
    }
    
    /// Store a quote in the cache
    pub async fn store_quote(&self, key: String, quote: AmmQuote) -> Result<()> {
        self.quotes.insert(key, quote).await
    }
    
    /// Get routes from the cache
    pub async fn get_routes(&self, key: &str) -> Option<Vec<String>> {
        self.routes.get(&key.to_string()).await
    }
    
    /// Store routes in the cache
    pub async fn store_routes(&self, key: String, routes: Vec<String>) -> Result<()> {
        self.routes.insert(key, routes).await
    }
    
    /// Evict expired entries from all caches
    pub async fn evict_expired(&self) -> Result<(usize, usize)> {
        let quote_evicted = self.quotes.evict_expired().await?;
        let route_evicted = self.routes.evict_expired().await?;
        Ok((quote_evicted, route_evicted))
    }
    
    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        CacheStats {
            quotes_count: self.quotes.len().await,
            routes_count: self.routes.len().await,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub quotes_count: usize,
    pub routes_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration as StdDuration;

    #[tokio::test]
    async fn test_cache_basic_operations() -> Result<()> {
        let cache: Cache<String, i32> = Cache::new(Duration::from_secs(1), 100);
        
        // Test insert and get
        cache.insert("key1".to_string(), 42).await?;
        let value = cache.get(&"key1".to_string()).await;
        assert_eq!(value, Some(42));
        
        // Test contains_key
        assert!(cache.contains_key(&"key1".to_string()).await);
        assert!(!cache.contains_key(&"key2".to_string()).await);
        
        // Test remove
        let removed = cache.remove(&"key1".to_string()).await?;
        assert_eq!(removed, Some(42));
        assert!(cache.get(&"key1".to_string()).await.is_none());
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_cache_expiration() -> Result<()> {
        let cache: Cache<String, i32> = Cache::new(Duration::from_millis(10), 100);
        
        cache.insert("key1".to_string(), 42).await?;
        assert!(cache.get(&"key1".to_string()).await.is_some());
        
        // Wait for expiration
        thread::sleep(StdDuration::from_millis(20));
        
        assert!(cache.get(&"key1".to_string()).await.is_none());
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_cache_size_limit() -> Result<()> {
        let cache: Cache<i32, String> = Cache::new(Duration::from_secs(1), 2);
        
        cache.insert(1, "value1".to_string()).await?;
        cache.insert(2, "value2".to_string()).await?;
        cache.insert(3, "value3".to_string()).await?;
        
        // One entry should have been evicted
        assert_eq!(cache.len().await, 2);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_amm_cache() -> Result<()> {
        let amm_cache = AmmCache::new();
        
        let quote = AmmQuote {
            expected_output: 1000000000000000000,
            price_impact: 0.5,
            gas_estimate: 150000,
            timestamp: 1234567890,
        };
        
        amm_cache.store_quote("quote-key".to_string(), quote.clone()).await?;
        let retrieved = amm_cache.get_quote("quote-key").await;
        assert_eq!(retrieved, Some(quote));
        
        let stats = amm_cache.stats().await;
        assert_eq!(stats.quotes_count, 1);
        
        Ok(())
    }
}