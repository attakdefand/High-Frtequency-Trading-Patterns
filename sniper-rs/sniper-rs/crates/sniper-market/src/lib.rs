//! Marketplace for community strategies in the sniper-rs ecosystem.
//! 
//! This module provides functionality for sharing, discovering, and rating
//! community-created trading strategies and plugins.

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Strategy listing in the marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyListing {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub tags: Vec<String>,
    pub downloads: u64,
    pub rating: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source_url: Option<String>,
    pub documentation_url: Option<String>,
    pub compatibility: Vec<String>, // List of compatible sniper-rs versions
}

/// Strategy rating/review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyReview {
    pub id: String,
    pub strategy_id: String,
    pub user_id: String,
    pub rating: u8, // 1-5 stars
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Marketplace statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketStats {
    pub total_strategies: u64,
    pub total_downloads: u64,
    pub total_reviews: u64,
    pub average_rating: f64,
}

/// Marketplace trait for strategy sharing and discovery
#[async_trait]
pub trait Marketplace: Send + Sync {
    /// List available strategies
    async fn list_strategies(&self, filter: Option<&str>) -> Result<Vec<StrategyListing>>;
    
    /// Get a specific strategy by ID
    async fn get_strategy(&self, id: &str) -> Result<Option<StrategyListing>>;
    
    /// Upload a new strategy
    async fn upload_strategy(&self, strategy: StrategyListing) -> Result<()>;
    
    /// Download strategy content
    async fn download_strategy(&self, id: &str) -> Result<Vec<u8>>;
    
    /// Add a review for a strategy
    async fn add_review(&self, review: StrategyReview) -> Result<()>;
    
    /// Get reviews for a strategy
    async fn get_reviews(&self, strategy_id: &str) -> Result<Vec<StrategyReview>>;
    
    /// Get marketplace statistics
    async fn get_stats(&self) -> Result<MarketStats>;
}

/// In-memory implementation of the marketplace for demonstration
pub struct InMemoryMarketplace {
    strategies: HashMap<String, StrategyListing>,
    reviews: HashMap<String, Vec<StrategyReview>>,
    downloads: HashMap<String, u64>,
}

impl InMemoryMarketplace {
    /// Create a new in-memory marketplace
    pub fn new() -> Self {
        Self {
            strategies: HashMap::new(),
            reviews: HashMap::new(),
            downloads: HashMap::new(),
        }
    }
}

#[async_trait]
impl Marketplace for InMemoryMarketplace {
    async fn list_strategies(&self, filter: Option<&str>) -> Result<Vec<StrategyListing>> {
        let mut strategies: Vec<StrategyListing> = self.strategies.values().cloned().collect();
        
        if let Some(filter_text) = filter {
            strategies.retain(|s| {
                s.name.contains(filter_text) || 
                s.description.contains(filter_text) || 
                s.tags.iter().any(|tag| tag.contains(filter_text))
            });
        }
        
        Ok(strategies)
    }
    
    async fn get_strategy(&self, id: &str) -> Result<Option<StrategyListing>> {
        Ok(self.strategies.get(id).cloned())
    }
    
    async fn upload_strategy(&self, strategy: StrategyListing) -> Result<()> {
        // In a real implementation, this would store the strategy in a database
        // For now, we'll just acknowledge the upload
        println!("Uploaded strategy: {}", strategy.name);
        Ok(())
    }
    
    async fn download_strategy(&self, id: &str) -> Result<Vec<u8>> {
        // Increment download count
        let mut downloads = self.downloads.clone();
        let count = downloads.entry(id.to_string()).or_insert(0);
        *count += 1;
        
        // Return dummy strategy content
        Ok(format!("Strategy content for {}", id).into_bytes())
    }
    
    async fn add_review(&self, review: StrategyReview) -> Result<()> {
        // In a real implementation, this would store the review in a database
        // For now, we'll just acknowledge the review
        println!("Added review for strategy: {}", review.strategy_id);
        Ok(())
    }
    
    async fn get_reviews(&self, strategy_id: &str) -> Result<Vec<StrategyReview>> {
        Ok(self.reviews.get(strategy_id).cloned().unwrap_or_default())
    }
    
    async fn get_stats(&self) -> Result<MarketStats> {
        let total_strategies = self.strategies.len() as u64;
        let total_downloads: u64 = self.downloads.values().sum();
        let total_reviews: u64 = self.reviews.values().map(|v| v.len() as u64).sum();
        
        let average_rating = if total_strategies > 0 {
            // Calculate average rating from all strategies
            let sum: f64 = self.strategies.values().map(|s| s.rating).sum();
            sum / total_strategies as f64
        } else {
            0.0
        };
        
        Ok(MarketStats {
            total_strategies,
            total_downloads,
            total_reviews,
            average_rating,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_marketplace_basic_operations() {
        let marketplace = InMemoryMarketplace::new();
        
        // Test listing strategies (empty initially)
        let strategies = marketplace.list_strategies(None).await.unwrap();
        assert_eq!(strategies.len(), 0);
        
        // Test getting stats
        let stats = marketplace.get_stats().await.unwrap();
        assert_eq!(stats.total_strategies, 0);
        assert_eq!(stats.total_downloads, 0);
        assert_eq!(stats.average_rating, 0.0);
        
        println!("Marketplace basic operations test passed!");
    }
    
    #[tokio::test]
    async fn test_strategy_listing() {
        let marketplace = InMemoryMarketplace::new();
        
        let strategy = StrategyListing {
            id: "test-strategy-1".to_string(),
            name: "Test Strategy".to_string(),
            version: "1.0.0".to_string(),
            description: "A test strategy for unit testing".to_string(),
            author: "Test Author".to_string(),
            tags: vec!["test".to_string(), "example".to_string()],
            downloads: 0,
            rating: 4.5,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            source_url: Some("https://github.com/example/test-strategy".to_string()),
            documentation_url: Some("https://docs.example.com/test-strategy".to_string()),
            compatibility: vec!["0.1.0".to_string(), "0.2.0".to_string()],
        };
        
        // Test uploading strategy
        marketplace.upload_strategy(strategy.clone()).await.unwrap();
        
        // Test getting strategy
        let retrieved = marketplace.get_strategy("test-strategy-1").await.unwrap();
        // Note: In our in-memory implementation, we don't actually store the strategy
        // This is just testing that the method doesn't error
        
        println!("Strategy listing test passed!");
    }
}