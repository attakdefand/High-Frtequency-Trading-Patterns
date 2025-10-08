//! Cross-protocol liquidity aggregation for the sniper-rs ecosystem.
//! 
//! This module provides functionality to aggregate liquidity across multiple
//! DeFi protocols and chains to find the best trading opportunities.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sniper_core::types::ChainRef;
use std::collections::HashMap;

/// Token pair information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TokenPair {
    pub token0: String,
    pub token1: String,
}

/// Liquidity source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquiditySource {
    pub protocol: String,
    pub chain: ChainRef,
    pub pair: TokenPair,
    pub reserve0: u128,
    pub reserve1: u128,
    pub fee: f64,
    pub timestamp: u64,
}

/// Aggregated liquidity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedLiquidity {
    pub pair: TokenPair,
    pub sources: Vec<LiquiditySource>,
    pub total_liquidity: u128,
    pub best_price: f64,
    pub price_impact: f64,
    pub timestamp: u64,
}

/// Liquidity aggregation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityConfig {
    pub chains: Vec<String>,
    pub protocols: Vec<String>,
    pub min_liquidity: u128,
    pub max_price_impact: f64,
}

/// Liquidity aggregator
pub struct LiquidityAggregator {
    config: LiquidityConfig,
    liquidity_sources: HashMap<String, Vec<LiquiditySource>>,
}

impl LiquidityAggregator {
    /// Create a new liquidity aggregator
    pub fn new(config: LiquidityConfig) -> Self {
        Self {
            config,
            liquidity_sources: HashMap::new(),
        }
    }
    
    /// Add liquidity source
    pub fn add_liquidity_source(&mut self, source_id: String, source: LiquiditySource) {
        self.liquidity_sources
            .entry(source_id)
            .or_insert_with(Vec::new)
            .push(source);
    }
    
    /// Remove liquidity source
    pub fn remove_liquidity_source(&mut self, source_id: &str) {
        self.liquidity_sources.remove(source_id);
    }
    
    /// Get all liquidity sources for a token pair
    pub fn get_liquidity_sources(&self, pair: &TokenPair) -> Vec<&LiquiditySource> {
        self.liquidity_sources
            .values()
            .flatten()
            .filter(|source| {
                &source.pair == pair
            })
            .collect()
    }
    
    /// Aggregate liquidity for a token pair across all sources
    pub fn aggregate_liquidity(&self, pair: &TokenPair) -> Result<AggregatedLiquidity> {
        let sources = self.get_liquidity_sources(pair);
        
        if sources.is_empty() {
            return Err(anyhow::anyhow!("No liquidity sources found for pair"));
        }
        
        // Calculate total liquidity
        let total_liquidity: u128 = sources.iter().map(|s| s.reserve0 + s.reserve1).sum();
        
        // Find best price (simple implementation)
        let best_price = sources
            .iter()
            .map(|s| s.reserve1 as f64 / s.reserve0 as f64)
            .fold(0.0/0.0, f64::max); // NaN initial value, will be replaced by first value
        
        // Calculate average price impact (simplified)
        let price_impact = sources
            .iter()
            .map(|s| s.fee)
            .sum::<f64>() / sources.len() as f64;
        
        Ok(AggregatedLiquidity {
            pair: pair.clone(),
            sources: sources.into_iter().cloned().collect(),
            total_liquidity,
            best_price,
            price_impact,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }
    
    /// Find the best route for a trade
    pub fn find_best_route(
        &self,
        token_in: &str,
        token_out: &str,
        amount_in: u128,
    ) -> Result<Option<TradeRoute>> {
        // This is a simplified implementation
        // In a real implementation, this would use pathfinding algorithms
        
        // For now, we'll just look for direct pairs
        let pair = TokenPair {
            token0: token_in.to_string(),
            token1: token_out.to_string(),
        };
        
        match self.aggregate_liquidity(&pair) {
            Ok(liquidity) => {
                // Check if liquidity is sufficient
                if liquidity.total_liquidity > amount_in && liquidity.price_impact < self.config.max_price_impact {
                    Ok(Some(TradeRoute {
                        path: vec![pair],
                        expected_output: (amount_in as f64 * liquidity.best_price) as u128,
                        price_impact: liquidity.price_impact,
                        sources: liquidity.sources,
                    }))
                } else {
                    Ok(None)
                }
            },
            Err(_) => Ok(None),
        }
    }
}

/// Trade route information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRoute {
    pub path: Vec<TokenPair>,
    pub expected_output: u128,
    pub price_impact: f64,
    pub sources: Vec<LiquiditySource>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_liquidity_aggregator_creation() {
        let config = LiquidityConfig {
            chains: vec!["ethereum".to_string(), "bsc".to_string()],
            protocols: vec!["uniswap".to_string(), "pancakeswap".to_string()],
            min_liquidity: 1000000,
            max_price_impact: 0.05,
        };
        
        let aggregator = LiquidityAggregator::new(config);
        assert!(aggregator.liquidity_sources.is_empty());
    }
    
    #[tokio::test]
    async fn test_liquidity_source_management() {
        let config = LiquidityConfig {
            chains: vec!["ethereum".to_string()],
            protocols: vec!["uniswap".to_string()],
            min_liquidity: 1000000,
            max_price_impact: 0.05,
        };
        
        let mut aggregator = LiquidityAggregator::new(config);
        
        let pair = TokenPair {
            token0: "WETH".to_string(),
            token1: "USDC".to_string(),
        };
        
        let source = LiquiditySource {
            protocol: "uniswap".to_string(),
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            pair: pair.clone(),
            reserve0: 1000000000000000000000, // 1000 WETH
            reserve1: 2000000000000, // 2,000,000 USDC
            fee: 0.003,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        // Add liquidity source
        aggregator.add_liquidity_source("uniswap_ethereum".to_string(), source.clone());
        assert_eq!(aggregator.liquidity_sources.len(), 1);
        
        // Get liquidity sources
        let sources = aggregator.get_liquidity_sources(&pair);
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].protocol, "uniswap");
        
        // Remove liquidity source
        aggregator.remove_liquidity_source("uniswap_ethereum");
        assert_eq!(aggregator.liquidity_sources.len(), 0);
        
        println!("Liquidity source management test passed!");
    }
    
    #[tokio::test]
    async fn test_liquidity_aggregation() -> Result<()> {
        let config = LiquidityConfig {
            chains: vec!["ethereum".to_string(), "bsc".to_string()],
            protocols: vec!["uniswap".to_string(), "pancakeswap".to_string()],
            min_liquidity: 1000000,
            max_price_impact: 0.05,
        };
        
        let mut aggregator = LiquidityAggregator::new(config);
        
        let pair = TokenPair {
            token0: "WETH".to_string(),
            token1: "USDC".to_string(),
        };
        
        // Add multiple liquidity sources
        let uniswap_source = LiquiditySource {
            protocol: "uniswap".to_string(),
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            pair: pair.clone(),
            reserve0: 1000000000000000000000, // 1000 WETH
            reserve1: 2000000000000, // 2,000,000 USDC
            fee: 0.003,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        let pancakeswap_source = LiquiditySource {
            protocol: "pancakeswap".to_string(),
            chain: ChainRef {
                name: "bsc".to_string(),
                id: 56,
            },
            pair: pair.clone(),
            reserve0: 500000000000000000000, // 500 WETH
            reserve1: 1000000000000, // 1,000,000 USDC
            fee: 0.0025,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        aggregator.add_liquidity_source("uniswap_ethereum".to_string(), uniswap_source);
        aggregator.add_liquidity_source("pancakeswap_bsc".to_string(), pancakeswap_source);
        
        // Aggregate liquidity
        let aggregated = aggregator.aggregate_liquidity(&pair)?;
        
        assert_eq!(aggregated.pair.token0, "WETH");
        assert_eq!(aggregated.pair.token1, "USDC");
        assert_eq!(aggregated.sources.len(), 2);
        assert!(aggregated.total_liquidity > 0);
        assert!(aggregated.best_price > 0.0);
        
        println!("Liquidity aggregation test passed!");
        Ok(())
    }
}