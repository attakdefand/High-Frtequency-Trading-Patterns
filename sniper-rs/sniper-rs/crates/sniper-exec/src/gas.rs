//! Gas bidding optimization strategies
//! 
//! This module provides functionality for optimizing gas bidding
//! based on network conditions, transaction priority, and cost considerations.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Gas policy for transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasPolicy {
    pub max_fee_gwei: u64,
    pub max_priority_gwei: u64,
}

/// Network congestion level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CongestionLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Gas bidding strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BiddingStrategy {
    Conservative,  // Low priority, cost-focused
    Balanced,      // Medium priority, balanced approach
    Aggressive,    // High priority, speed-focused
    Adaptive,      // Dynamically adjusts based on network conditions
}

/// Gas bid recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasBid {
    pub max_fee_gwei: u64,
    pub max_priority_gwei: u64,
    pub congestion_level: CongestionLevel,
    pub strategy_used: BiddingStrategy,
}

/// Gas bidder that calculates optimal gas bids
pub struct GasBidder {
    // Historical data for adaptive bidding
    history: Arc<RwLock<HashMap<String, Vec<GasBid>>>>,
}

impl GasBidder {
    /// Create a new gas bidder
    pub fn new() -> Self {
        Self {
            history: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Calculate optimal gas bid based on policy and network conditions
    pub async fn calculate_bid(&self, policy: &GasPolicy, network_congestion_pct: u64) -> Result<GasBid> {
        let congestion_level = self.determine_congestion_level(network_congestion_pct);
        let strategy = self.determine_strategy(policy, &congestion_level);
        
        let (max_fee, max_priority) = match strategy {
            BiddingStrategy::Conservative => {
                self.calculate_conservative_bid(policy, &congestion_level)
            }
            BiddingStrategy::Balanced => {
                self.calculate_balanced_bid(policy, &congestion_level)
            }
            BiddingStrategy::Aggressive => {
                self.calculate_aggressive_bid(policy, &congestion_level)
            }
            BiddingStrategy::Adaptive => {
                self.calculate_adaptive_bid(policy, &congestion_level, network_congestion_pct).await
            }
        };
        
        Ok(GasBid {
            max_fee_gwei: max_fee,
            max_priority_gwei: max_priority,
            congestion_level,
            strategy_used: strategy,
        })
    }
    
    /// Determine congestion level based on network metrics
    fn determine_congestion_level(&self, congestion_pct: u64) -> CongestionLevel {
        match congestion_pct {
            0..=25 => CongestionLevel::Low,
            26..=50 => CongestionLevel::Medium,
            51..=75 => CongestionLevel::High,
            _ => CongestionLevel::VeryHigh,
        }
    }
    
    /// Determine bidding strategy based on policy and congestion
    fn determine_strategy(&self, policy: &GasPolicy, congestion: &CongestionLevel) -> BiddingStrategy {
        // If policy is very conservative, use conservative strategy
        if policy.max_fee_gwei <= 30 && policy.max_priority_gwei <= 1 {
            return BiddingStrategy::Conservative;
        }
        
        // If policy is very aggressive, use aggressive strategy
        if policy.max_fee_gwei >= 150 && policy.max_priority_gwei >= 5 {
            return BiddingStrategy::Aggressive;
        }
        
        // For medium policies, adapt based on congestion
        match congestion {
            CongestionLevel::Low | CongestionLevel::Medium => BiddingStrategy::Balanced,
            CongestionLevel::High | CongestionLevel::VeryHigh => BiddingStrategy::Adaptive,
        }
    }
    
    /// Calculate conservative bid (lowest cost)
    fn calculate_conservative_bid(&self, policy: &GasPolicy, congestion: &CongestionLevel) -> (u64, u64) {
        let multiplier = match congestion {
            CongestionLevel::Low => 80,
            CongestionLevel::Medium => 90,
            CongestionLevel::High => 100,
            CongestionLevel::VeryHigh => 110,
        };
        
        let max_fee = (policy.max_fee_gwei * multiplier) / 100;
        let max_priority = (policy.max_priority_gwei * 50) / 100;
        
        (max_fee.max(10), max_priority.max(1))
    }
    
    /// Calculate balanced bid (moderate cost and speed)
    fn calculate_balanced_bid(&self, policy: &GasPolicy, congestion: &CongestionLevel) -> (u64, u64) {
        let multiplier = match congestion {
            CongestionLevel::Low => 90,
            CongestionLevel::Medium => 100,
            CongestionLevel::High => 120,
            CongestionLevel::VeryHigh => 150,
        };
        
        let max_fee = (policy.max_fee_gwei * multiplier) / 100;
        let max_priority = (policy.max_priority_gwei * 80) / 100;
        
        (max_fee.max(15), max_priority.max(1))
    }
    
    /// Calculate aggressive bid (highest speed)
    fn calculate_aggressive_bid(&self, policy: &GasPolicy, congestion: &CongestionLevel) -> (u64, u64) {
        let multiplier = match congestion {
            CongestionLevel::Low => 100,
            CongestionLevel::Medium => 120,
            CongestionLevel::High => 150,
            CongestionLevel::VeryHigh => 200,
        };
        
        let max_fee = (policy.max_fee_gwei * multiplier) / 100;
        let max_priority = (policy.max_priority_gwei * 120) / 100;
        
        (max_fee.max(20), max_priority.max(2))
    }
    
    /// Calculate adaptive bid based on historical data
    async fn calculate_adaptive_bid(&self, policy: &GasPolicy, _congestion: &CongestionLevel, congestion_pct: u64) -> (u64, u64) {
        // Get historical data for this chain
        let history = self.history.read().await;
        let chain_key = format!("chain_{}", congestion_pct / 10); // Group by 10% increments
        
        let historical_avg = if let Some(bids) = history.get(&chain_key) {
            if !bids.is_empty() {
                let sum_fee: u64 = bids.iter().map(|bid| bid.max_fee_gwei).sum();
                let sum_priority: u64 = bids.iter().map(|bid| bid.max_priority_gwei).sum();
                (sum_fee / bids.len() as u64, sum_priority / bids.len() as u64)
            } else {
                (policy.max_fee_gwei, policy.max_priority_gwei)
            }
        } else {
            (policy.max_fee_gwei, policy.max_priority_gwei)
        };
        
        // Adjust based on current congestion
        let adjustment_factor = congestion_pct * 50; // Scale to 0-5000
        let max_fee = (historical_avg.0 * (100 + adjustment_factor) / 100).max(10);
        let max_priority = (historical_avg.1 * (100 + adjustment_factor / 2) / 100).max(1);
        
        // Ensure we don't exceed policy limits
        let max_fee = max_fee.min(policy.max_fee_gwei * 2).max(10);
        let max_priority = max_priority.min(policy.max_priority_gwei * 2).max(1);
        
        (max_fee, max_priority)
    }
    
    /// Record successful bid for future adaptive calculations
    pub async fn record_successful_bid(&self, chain_id: u64, bid: GasBid) -> Result<()> {
        let mut history = self.history.write().await;
        let chain_key = format!("chain_{}", chain_id);
        
        if let Some(bids) = history.get_mut(&chain_key) {
            bids.push(bid);
            // Keep only the last 100 bids for memory efficiency
            if bids.len() > 100 {
                bids.drain(0..bids.len() - 100);
            }
        } else {
            history.insert(chain_key, vec![bid]);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_conservative_bidding() -> Result<()> {
        let bidder = GasBidder::new();
        let policy = GasPolicy {
            max_fee_gwei: 25, // Conservative policy
            max_priority_gwei: 1,
        };
        
        let bid = bidder.calculate_bid(&policy, 20).await?;
        assert_eq!(bid.strategy_used, BiddingStrategy::Conservative);
        assert!(bid.max_fee_gwei <= policy.max_fee_gwei * 110 / 100);
        assert!(bid.max_priority_gwei <= policy.max_priority_gwei);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_aggressive_bidding() -> Result<()> {
        let bidder = GasBidder::new();
        let policy = GasPolicy {
            max_fee_gwei: 200,
            max_priority_gwei: 10,
        };
        
        let bid = bidder.calculate_bid(&policy, 80).await?;
        assert_eq!(bid.strategy_used, BiddingStrategy::Aggressive);
        assert!(bid.max_fee_gwei >= policy.max_fee_gwei * 80 / 100);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_adaptive_bidding() -> Result<()> {
        let bidder = GasBidder::new();
        let policy = GasPolicy {
            max_fee_gwei: 100,
            max_priority_gwei: 5,
        };
        
        // Record some historical data
        let historical_bid = GasBid {
            max_fee_gwei: 80,
            max_priority_gwei: 3,
            congestion_level: CongestionLevel::High,
            strategy_used: BiddingStrategy::Balanced,
        };
        bidder.record_successful_bid(1, historical_bid).await?;
        
        let bid = bidder.calculate_bid(&policy, 70).await?;
        assert_eq!(bid.strategy_used, BiddingStrategy::Adaptive);
        
        Ok(())
    }
    
    #[test]
    fn test_congestion_level_determination() {
        let bidder = GasBidder::new();
        assert_eq!(bidder.determine_congestion_level(10), CongestionLevel::Low);
        assert_eq!(bidder.determine_congestion_level(40), CongestionLevel::Medium);
        assert_eq!(bidder.determine_congestion_level(60), CongestionLevel::High);
        assert_eq!(bidder.determine_congestion_level(90), CongestionLevel::VeryHigh);
    }
}