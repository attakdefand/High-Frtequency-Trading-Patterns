//! Portfolio management system for the sniper bot.
//! 
//! This module provides functionality for managing trading portfolios,
//! including position tracking, risk allocation, and performance analytics.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sniper_core::types::{ChainRef, TradePlan};
use std::collections::HashMap;

/// Portfolio position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: String,
    pub symbol: String,
    pub chain: ChainRef,
    pub amount: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub side: String, // "long" or "short"
    pub leverage: f64,
    pub pnl: f64,
    pub pnl_percentage: f64,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Portfolio allocation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationSettings {
    pub max_position_size_pct: f64, // Maximum position size as percentage of portfolio
    pub max_portfolio_risk_pct: f64, // Maximum risk as percentage of portfolio
    pub diversification_targets: HashMap<String, f64>, // Target allocation by asset class
    pub stop_loss_pct: f64, // Default stop loss percentage
    pub take_profit_pct: f64, // Default take profit percentage
}

/// Portfolio performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_value: f64,
    pub total_pnl: f64,
    pub total_pnl_percentage: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub positions_count: usize,
}

/// Portfolio manager
pub struct PortfolioManager {
    positions: HashMap<String, Position>,
    allocation_settings: AllocationSettings,
    initial_capital: f64,
}

impl PortfolioManager {
    /// Create a new portfolio manager
    pub fn new(initial_capital: f64, allocation_settings: AllocationSettings) -> Self {
        Self {
            positions: HashMap::new(),
            allocation_settings,
            initial_capital,
        }
    }

    /// Add a new position to the portfolio
    pub fn add_position(&mut self, position: Position) -> Result<()> {
        // Validate position size against allocation settings
        if !self.validate_position_size(&position)? {
            return Err(anyhow::anyhow!("Position size exceeds allocation limits"));
        }
        
        self.positions.insert(position.id.clone(), position);
        Ok(())
    }

    /// Update an existing position
    pub fn update_position(&mut self, position_id: &str, updated_position: Position) -> Result<()> {
        if self.positions.contains_key(position_id) {
            // Validate position size for updated position
            if !self.validate_position_size(&updated_position)? {
                return Err(anyhow::anyhow!("Updated position size exceeds allocation limits"));
            }
            
            self.positions.insert(position_id.to_string(), updated_position);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Position not found"))
        }
    }

    /// Remove a position from the portfolio
    pub fn remove_position(&mut self, position_id: &str) -> Result<()> {
        if self.positions.remove(position_id).is_some() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Position not found"))
        }
    }

    /// Get a position by ID
    pub fn get_position(&self, position_id: &str) -> Option<&Position> {
        self.positions.get(position_id)
    }

    /// List all positions
    pub fn list_positions(&self) -> Vec<&Position> {
        self.positions.values().collect()
    }

    /// Calculate portfolio performance metrics
    pub fn calculate_performance(&self) -> PerformanceMetrics {
        let mut total_value = self.initial_capital;
        let mut total_pnl = 0.0;
        let mut winning_trades = 0;
        let mut total_wins = 0.0;
        let mut total_losses = 0.0;
        
        for position in self.positions.values() {
            total_value += position.pnl;
            total_pnl += position.pnl;
            
            if position.pnl > 0.0 {
                winning_trades += 1;
                total_wins += position.pnl;
            } else {
                total_losses += position.pnl.abs();
            }
        }
        
        let win_rate = if self.positions.is_empty() {
            0.0
        } else {
            winning_trades as f64 / self.positions.len() as f64
        };
        
        let profit_factor = if total_losses > 0.0 {
            total_wins / total_losses
        } else if total_wins > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };
        
        let total_pnl_percentage = if self.initial_capital > 0.0 {
            (total_pnl / self.initial_capital) * 100.0
        } else {
            0.0
        };
        
        // Simplified Sharpe ratio and max drawdown calculations
        let sharpe_ratio = if total_pnl > 0.0 && total_losses > 0.0 {
            total_pnl / total_losses
        } else {
            0.0
        };
        
        let max_drawdown = if total_losses > 0.0 {
            (total_losses / self.initial_capital) * 100.0
        } else {
            0.0
        };
        
        PerformanceMetrics {
            total_value,
            total_pnl,
            total_pnl_percentage,
            win_rate,
            profit_factor,
            sharpe_ratio,
            max_drawdown,
            positions_count: self.positions.len(),
        }
    }

    /// Validate that a position size is within allocation limits
    fn validate_position_size(&self, position: &Position) -> Result<bool> {
        let position_value = position.amount * position.current_price;
        let portfolio_value = self.calculate_portfolio_value();
        
        // If portfolio is empty, allow the position
        if portfolio_value == 0.0 && self.initial_capital == 0.0 {
            return Ok(true);
        }
        
        let total_portfolio_value = portfolio_value;
        if total_portfolio_value > 0.0 {
            let position_pct = (position_value / total_portfolio_value) * 100.0;
            Ok(position_pct <= self.allocation_settings.max_position_size_pct)
        } else {
            Ok(true)
        }
    }

    /// Calculate total portfolio value
    fn calculate_portfolio_value(&self) -> f64 {
        let mut value = self.initial_capital;
        for position in self.positions.values() {
            value += position.pnl;
        }
        value
    }

    /// Generate a trade plan based on portfolio allocation
    pub fn generate_trade_plan(&self, _symbol: &str, chain: ChainRef, amount: f64, _side: &str) -> Result<TradePlan> {
        // In a real implementation, this would:
        // 1. Check risk allocation
        // 2. Validate against portfolio constraints
        // 3. Generate appropriate trade parameters
        // 4. Apply position sizing algorithms
        
        // For now, return a placeholder
        Ok(TradePlan {
            chain,
            router: "0xRouter".to_string(),
            token_in: "0xTokenIn".to_string(),
            token_out: "0xTokenOut".to_string(),
            amount_in: (amount * 1e18) as u128, // Convert to wei
            min_out: ((amount * 0.95) * 1e18) as u128, // 5% slippage
            mode: sniper_core::types::ExecMode::Mempool,
            gas: sniper_core::types::GasPolicy {
                max_fee_gwei: 50,
                max_priority_gwei: 2,
            },
            exits: sniper_core::types::ExitRules {
                take_profit_pct: Some(self.allocation_settings.take_profit_pct),
                stop_loss_pct: Some(self.allocation_settings.stop_loss_pct),
                trailing_pct: Some(2.0),
            },
            idem_key: format!("portfolio-trade-{}", uuid::Uuid::new_v4()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sniper_core::types::{ChainRef, ExecMode, GasPolicy, ExitRules};

    #[test]
    fn test_portfolio_manager_creation() {
        let settings = AllocationSettings {
            max_position_size_pct: 5.0,
            max_portfolio_risk_pct: 2.0,
            diversification_targets: HashMap::new(),
            stop_loss_pct: 5.0,
            take_profit_pct: 10.0,
        };
        
        let portfolio = PortfolioManager::new(10000.0, settings);
        assert_eq!(portfolio.initial_capital, 10000.0);
        assert_eq!(portfolio.positions.len(), 0);
    }

    #[test]
    fn test_add_position() {
        let settings = AllocationSettings {
            max_position_size_pct: 50.0, // 50% to allow positions
            max_portfolio_risk_pct: 2.0,
            diversification_targets: HashMap::new(),
            stop_loss_pct: 5.0,
            take_profit_pct: 10.0,
        };
        
        let mut portfolio = PortfolioManager::new(10000.0, settings);
        
        let position = Position {
            id: "pos-1".to_string(),
            symbol: "BTC/USDT".to_string(),
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            amount: 0.05, // Smaller amount to stay within limits (0.05 * 51000 = 2550, 2550/10000 = 25.5%)
            entry_price: 50000.0,
            current_price: 51000.0,
            side: "long".to_string(),
            leverage: 1.0,
            pnl: 500.0,
            pnl_percentage: 1.0,
            created_at: 1234567890,
            updated_at: 1234567890,
        };
        
        let result = portfolio.add_position(position);
        assert!(result.is_ok());
        assert_eq!(portfolio.positions.len(), 1);
    }

    #[test]
    fn test_update_position() {
        let settings = AllocationSettings {
            max_position_size_pct: 50.0, // 50% to allow positions
            max_portfolio_risk_pct: 2.0,
            diversification_targets: HashMap::new(),
            stop_loss_pct: 5.0,
            take_profit_pct: 10.0,
        };
        
        let mut portfolio = PortfolioManager::new(10000.0, settings);
        
        let position = Position {
            id: "pos-1".to_string(),
            symbol: "BTC/USDT".to_string(),
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            amount: 0.05, // Smaller amount to stay within limits
            entry_price: 50000.0,
            current_price: 51000.0,
            side: "long".to_string(),
            leverage: 1.0,
            pnl: 500.0,
            pnl_percentage: 1.0,
            created_at: 1234567890,
            updated_at: 1234567890,
        };
        
        portfolio.add_position(position.clone()).unwrap();
        
        let mut updated_position = position;
        updated_position.current_price = 52000.0;
        updated_position.pnl = 1000.0;
        
        let result = portfolio.update_position("pos-1", updated_position);
        assert!(result.is_ok());
        
        let retrieved = portfolio.get_position("pos-1").unwrap();
        assert_eq!(retrieved.current_price, 52000.0);
        assert_eq!(retrieved.pnl, 1000.0);
    }

    #[test]
    fn test_remove_position() {
        let settings = AllocationSettings {
            max_position_size_pct: 50.0, // 50% to allow positions
            max_portfolio_risk_pct: 2.0,
            diversification_targets: HashMap::new(),
            stop_loss_pct: 5.0,
            take_profit_pct: 10.0,
        };
        
        let mut portfolio = PortfolioManager::new(10000.0, settings);
        
        let position = Position {
            id: "pos-1".to_string(),
            symbol: "BTC/USDT".to_string(),
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            amount: 0.05, // Smaller amount to stay within limits
            entry_price: 50000.0,
            current_price: 51000.0,
            side: "long".to_string(),
            leverage: 1.0,
            pnl: 500.0,
            pnl_percentage: 1.0,
            created_at: 1234567890,
            updated_at: 1234567890,
        };
        
        portfolio.add_position(position).unwrap();
        assert_eq!(portfolio.positions.len(), 1);
        
        let result = portfolio.remove_position("pos-1");
        assert!(result.is_ok());
        assert_eq!(portfolio.positions.len(), 0);
    }

    #[test]
    fn test_calculate_performance() {
        let settings = AllocationSettings {
            max_position_size_pct: 50.0, // 50% to allow positions
            max_portfolio_risk_pct: 2.0,
            diversification_targets: HashMap::new(),
            stop_loss_pct: 5.0,
            take_profit_pct: 10.0,
        };
        
        let mut portfolio = PortfolioManager::new(10000.0, settings);
        
        let position1 = Position {
            id: "pos-1".to_string(),
            symbol: "BTC/USDT".to_string(),
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            amount: 0.05, // Smaller amount to stay within limits (0.05 * 51000 = 2550)
            entry_price: 50000.0,
            current_price: 51000.0,
            side: "long".to_string(),
            leverage: 1.0,
            pnl: 500.0,
            pnl_percentage: 1.0,
            created_at: 1234567890,
            updated_at: 1234567890,
        };
        
        let position2 = Position {
            id: "pos-2".to_string(),
            symbol: "ETH/USDT".to_string(),
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            amount: 0.5, // Smaller amount to stay within limits (0.5 * 3100 = 1550)
            entry_price: 3000.0,
            current_price: 3100.0,
            side: "long".to_string(),
            leverage: 1.0,
            pnl: 500.0,
            pnl_percentage: 1.67,
            created_at: 1234567890,
            updated_at: 1234567890,
        };
        
        portfolio.add_position(position1).unwrap();
        portfolio.add_position(position2).unwrap();
        
        let performance = portfolio.calculate_performance();
        assert_eq!(performance.total_value, 11000.0); // 10000 + 500 + 500
        assert_eq!(performance.total_pnl, 1000.0);
        assert_eq!(performance.positions_count, 2);
        assert_eq!(performance.win_rate, 1.0); // All winning positions
    }

    #[test]
    fn test_generate_trade_plan() {
        let settings = AllocationSettings {
            max_position_size_pct: 5.0,
            max_portfolio_risk_pct: 2.0,
            diversification_targets: HashMap::new(),
            stop_loss_pct: 5.0,
            take_profit_pct: 10.0,
        };
        
        let portfolio = PortfolioManager::new(10000.0, settings);
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };
        
        let trade_plan = portfolio.generate_trade_plan("BTC/USDT", chain, 1.0, "long");
        assert!(trade_plan.is_ok());
        
        let plan = trade_plan.unwrap();
        assert_eq!(plan.chain.id, 1);
        assert_eq!(plan.amount_in, 1000000000000000000); // 1 ETH in wei
        assert_eq!(plan.min_out, 950000000000000000); // 0.95 ETH in wei
        assert_eq!(plan.exits.take_profit_pct, Some(10.0));
        assert_eq!(plan.exits.stop_loss_pct, Some(5.0));
    }
}