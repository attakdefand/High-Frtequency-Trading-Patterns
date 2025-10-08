//! Advanced order types for the sniper bot.
//! 
//! This module provides functionality for advanced order types including
//! limit orders, stop-loss orders, take-profit orders, trailing stops, and more.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sniper_core::types::{TradePlan, ChainRef, ExecMode, GasPolicy, ExitRules};

/// Order types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderType {
    Market,
    Limit { price: f64 },
    StopLoss { price: f64 },
    TakeProfit { price: f64 },
    StopLimit { stop_price: f64, limit_price: f64 },
    TrailingStop { trail_percent: f64 },
    Iceberg { visible_amount: f64, total_amount: f64 },
    TWAP { total_amount: f64, duration_minutes: u64 },
    VWAP { total_amount: f64 },
}

/// Order time in force
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimeInForce {
    GoodTillCancelled, // GTC
    ImmediateOrCancel, // IOC
    FillOrKill,        // FOK
    GoodTillTime { expiry_timestamp: u64 }, // GTT
}

/// Advanced order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedOrder {
    pub id: String,
    pub symbol: String,
    pub chain: ChainRef,
    pub order_type: OrderType,
    pub side: String, // "buy" or "sell"
    pub amount: f64,
    pub time_in_force: TimeInForce,
    pub created_at: u64,
    pub updated_at: u64,
    pub status: OrderStatus,
}

/// Order status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    Pending,
    Active,
    Filled,
    Cancelled,
    Expired,
    Rejected,
}

/// Order manager for handling advanced order types
pub struct OrderManager {
    orders: std::collections::HashMap<String, AdvancedOrder>,
}

impl OrderManager {
    /// Create a new order manager
    pub fn new() -> Self {
        Self {
            orders: std::collections::HashMap::new(),
        }
    }

    /// Create a new advanced order
    pub fn create_order(&mut self, order: AdvancedOrder) -> Result<String> {
        let order_id = order.id.clone();
        self.orders.insert(order_id.clone(), order);
        Ok(order_id)
    }

    /// Cancel an order
    pub fn cancel_order(&mut self, order_id: &str) -> Result<()> {
        if let Some(order) = self.orders.get_mut(order_id) {
            order.status = OrderStatus::Cancelled;
            order.updated_at = chrono::Utc::now().timestamp() as u64;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Order not found"))
        }
    }

    /// Get an order by ID
    pub fn get_order(&self, order_id: &str) -> Option<&AdvancedOrder> {
        self.orders.get(order_id)
    }

    /// List all orders
    pub fn list_orders(&self) -> Vec<&AdvancedOrder> {
        self.orders.values().collect()
    }

    /// List orders by status
    pub fn list_orders_by_status(&self, status: OrderStatus) -> Vec<&AdvancedOrder> {
        self.orders.values().filter(|order| order.status == status).collect()
    }

    /// Convert an advanced order to a trade plan
    pub fn to_trade_plan(&self, order_id: &str, current_price: f64) -> Result<TradePlan> {
        let order = self.get_order(order_id).ok_or_else(|| anyhow::anyhow!("Order not found"))?;
        
        // Check if order should be executed based on order type and current price
        if !self.should_execute_order(order, current_price)? {
            return Err(anyhow::anyhow!("Order conditions not met"));
        }
        
        // Convert to trade plan
        let amount_in = (order.amount * 1e18) as u128; // Convert to wei
        let min_out = match &order.order_type {
            OrderType::Market => (order.amount * 0.95 * 1e18) as u128, // 5% slippage
            OrderType::Limit { .. } => (order.amount * 0.95 * 1e18) as u128, // 5% slippage
            OrderType::StopLoss { .. } => (order.amount * 0.95 * 1e18) as u128, // 5% slippage
            OrderType::TakeProfit { .. } => (order.amount * 0.95 * 1e18) as u128, // 5% slippage
            OrderType::StopLimit { .. } => (order.amount * 0.95 * 1e18) as u128, // 5% slippage
            OrderType::TrailingStop { .. } => (order.amount * 0.95 * 1e18) as u128, // 5% slippage
            OrderType::Iceberg { visible_amount, .. } => (visible_amount * 0.95 * 1e18) as u128, // 5% slippage
            OrderType::TWAP { .. } => (order.amount * 0.95 * 1e18) as u128, // 5% slippage
            OrderType::VWAP { .. } => (order.amount * 0.95 * 1e18) as u128, // 5% slippage
        };
        
        Ok(TradePlan {
            chain: order.chain.clone(),
            router: "0xRouter".to_string(),
            token_in: "0xTokenIn".to_string(),
            token_out: "0xTokenOut".to_string(),
            amount_in,
            min_out,
            mode: ExecMode::Mempool,
            gas: GasPolicy {
                max_fee_gwei: 50,
                max_priority_gwei: 2,
            },
            exits: ExitRules {
                take_profit_pct: Some(10.0),
                stop_loss_pct: Some(5.0),
                trailing_pct: Some(2.0),
            },
            idem_key: format!("order-{}", uuid::Uuid::new_v4()),
        })
    }

    /// Check if an order should be executed based on current price
    fn should_execute_order(&self, order: &AdvancedOrder, current_price: f64) -> Result<bool> {
        match &order.order_type {
            OrderType::Market => Ok(true), // Always execute market orders
            OrderType::Limit { price } => {
                // Buy limit: current price <= limit price
                // Sell limit: current price >= limit price
                if order.side == "buy" {
                    Ok(current_price <= *price)
                } else {
                    Ok(current_price >= *price)
                }
            }
            OrderType::StopLoss { price } => {
                // Buy stop-loss: current price >= stop price
                // Sell stop-loss: current price <= stop price
                if order.side == "buy" {
                    Ok(current_price >= *price)
                } else {
                    Ok(current_price <= *price)
                }
            }
            OrderType::TakeProfit { price } => {
                // Buy take-profit: current price >= take profit price
                // Sell take-profit: current price <= take profit price
                if order.side == "buy" {
                    Ok(current_price >= *price)
                } else {
                    Ok(current_price <= *price)
                }
            }
            OrderType::StopLimit { stop_price, limit_price } => {
                // First check if stop price is hit, then check limit price
                let stop_hit = if order.side == "buy" {
                    current_price >= *stop_price
                } else {
                    current_price <= *stop_price
                };
                
                if stop_hit {
                    if order.side == "buy" {
                        Ok(current_price <= *limit_price)
                    } else {
                        Ok(current_price >= *limit_price)
                    }
                } else {
                    Ok(false)
                }
            }
            OrderType::TrailingStop { trail_percent } => {
                // For trailing stops, we would normally track the highest/lowest price
                // For simplicity, we'll execute if price moved by trail_percent
                Ok(current_price.abs() >= *trail_percent)
            }
            _ => Ok(true), // For other order types, execute for now
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sniper_core::types::ChainRef;

    #[test]
    fn test_order_manager_creation() {
        let order_manager = OrderManager::new();
        assert_eq!(order_manager.orders.len(), 0);
    }

    #[test]
    fn test_create_order() {
        let mut order_manager = OrderManager::new();
        
        let order = AdvancedOrder {
            id: "order-1".to_string(),
            symbol: "BTC/USDT".to_string(),
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            order_type: OrderType::Market,
            side: "buy".to_string(),
            amount: 1.0,
            time_in_force: TimeInForce::GoodTillCancelled,
            created_at: 1234567890,
            updated_at: 1234567890,
            status: OrderStatus::Pending,
        };
        
        let result = order_manager.create_order(order);
        assert!(result.is_ok());
        assert_eq!(order_manager.orders.len(), 1);
    }

    #[test]
    fn test_cancel_order() {
        let mut order_manager = OrderManager::new();
        
        let order = AdvancedOrder {
            id: "order-1".to_string(),
            symbol: "BTC/USDT".to_string(),
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            order_type: OrderType::Market,
            side: "buy".to_string(),
            amount: 1.0,
            time_in_force: TimeInForce::GoodTillCancelled,
            created_at: 1234567890,
            updated_at: 1234567890,
            status: OrderStatus::Pending,
        };
        
        order_manager.create_order(order).unwrap();
        assert_eq!(order_manager.orders.len(), 1);
        
        let result = order_manager.cancel_order("order-1");
        assert!(result.is_ok());
        
        let cancelled_order = order_manager.get_order("order-1").unwrap();
        assert_eq!(cancelled_order.status, OrderStatus::Cancelled);
    }

    #[test]
    fn test_get_order() {
        let mut order_manager = OrderManager::new();
        
        let order = AdvancedOrder {
            id: "order-1".to_string(),
            symbol: "BTC/USDT".to_string(),
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            order_type: OrderType::Market,
            side: "buy".to_string(),
            amount: 1.0,
            time_in_force: TimeInForce::GoodTillCancelled,
            created_at: 1234567890,
            updated_at: 1234567890,
            status: OrderStatus::Pending,
        };
        
        order_manager.create_order(order).unwrap();
        
        let retrieved = order_manager.get_order("order-1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "order-1");
    }

    #[test]
    fn test_list_orders() {
        let mut order_manager = OrderManager::new();
        
        let order1 = AdvancedOrder {
            id: "order-1".to_string(),
            symbol: "BTC/USDT".to_string(),
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            order_type: OrderType::Market,
            side: "buy".to_string(),
            amount: 1.0,
            time_in_force: TimeInForce::GoodTillCancelled,
            created_at: 1234567890,
            updated_at: 1234567890,
            status: OrderStatus::Pending,
        };
        
        let order2 = AdvancedOrder {
            id: "order-2".to_string(),
            symbol: "ETH/USDT".to_string(),
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            order_type: OrderType::Limit { price: 3000.0 },
            side: "sell".to_string(),
            amount: 2.0,
            time_in_force: TimeInForce::GoodTillCancelled,
            created_at: 1234567890,
            updated_at: 1234567890,
            status: OrderStatus::Pending,
        };
        
        order_manager.create_order(order1).unwrap();
        order_manager.create_order(order2).unwrap();
        
        let orders = order_manager.list_orders();
        assert_eq!(orders.len(), 2);
    }

    #[test]
    fn test_list_orders_by_status() {
        let mut order_manager = OrderManager::new();
        
        let order1 = AdvancedOrder {
            id: "order-1".to_string(),
            symbol: "BTC/USDT".to_string(),
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            order_type: OrderType::Market,
            side: "buy".to_string(),
            amount: 1.0,
            time_in_force: TimeInForce::GoodTillCancelled,
            created_at: 1234567890,
            updated_at: 1234567890,
            status: OrderStatus::Pending,
        };
        
        let order2 = AdvancedOrder {
            id: "order-2".to_string(),
            symbol: "ETH/USDT".to_string(),
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            order_type: OrderType::Limit { price: 3000.0 },
            side: "sell".to_string(),
            amount: 2.0,
            time_in_force: TimeInForce::GoodTillCancelled,
            created_at: 1234567890,
            updated_at: 1234567890,
            status: OrderStatus::Active,
        };
        
        order_manager.create_order(order1).unwrap();
        order_manager.create_order(order2).unwrap();
        
        let pending_orders = order_manager.list_orders_by_status(OrderStatus::Pending);
        assert_eq!(pending_orders.len(), 1);
        assert_eq!(pending_orders[0].id, "order-1");
        
        let active_orders = order_manager.list_orders_by_status(OrderStatus::Active);
        assert_eq!(active_orders.len(), 1);
        assert_eq!(active_orders[0].id, "order-2");
    }

    #[test]
    fn test_should_execute_order() {
        let order_manager = OrderManager::new();
        
        // Test market order - should always execute
        let market_order = AdvancedOrder {
            id: "order-1".to_string(),
            symbol: "BTC/USDT".to_string(),
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            order_type: OrderType::Market,
            side: "buy".to_string(),
            amount: 1.0,
            time_in_force: TimeInForce::GoodTillCancelled,
            created_at: 1234567890,
            updated_at: 1234567890,
            status: OrderStatus::Pending,
        };
        
        let should_execute = order_manager.should_execute_order(&market_order, 50000.0).unwrap();
        assert!(should_execute);
        
        // Test buy limit order - should execute when current price <= limit price
        let limit_order = AdvancedOrder {
            id: "order-2".to_string(),
            symbol: "BTC/USDT".to_string(),
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            order_type: OrderType::Limit { price: 49000.0 },
            side: "buy".to_string(),
            amount: 1.0,
            time_in_force: TimeInForce::GoodTillCancelled,
            created_at: 1234567890,
            updated_at: 1234567890,
            status: OrderStatus::Pending,
        };
        
        // Current price is higher than limit - should not execute
        let should_execute = order_manager.should_execute_order(&limit_order, 50000.0).unwrap();
        assert!(!should_execute);
        
        // Current price is lower than limit - should execute
        let should_execute = order_manager.should_execute_order(&limit_order, 48000.0).unwrap();
        assert!(should_execute);
        
        // Test sell limit order - should execute when current price >= limit price
        let sell_limit_order = AdvancedOrder {
            id: "order-3".to_string(),
            symbol: "BTC/USDT".to_string(),
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            order_type: OrderType::Limit { price: 51000.0 },
            side: "sell".to_string(),
            amount: 1.0,
            time_in_force: TimeInForce::GoodTillCancelled,
            created_at: 1234567890,
            updated_at: 1234567890,
            status: OrderStatus::Pending,
        };
        
        // Current price is lower than limit - should not execute
        let should_execute = order_manager.should_execute_order(&sell_limit_order, 50000.0).unwrap();
        assert!(!should_execute);
        
        // Current price is higher than limit - should execute
        let should_execute = order_manager.should_execute_order(&sell_limit_order, 52000.0).unwrap();
        assert!(should_execute);
    }

    #[test]
    fn test_to_trade_plan() {
        let mut order_manager = OrderManager::new();
        
        let order = AdvancedOrder {
            id: "order-1".to_string(),
            symbol: "BTC/USDT".to_string(),
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            order_type: OrderType::Market,
            side: "buy".to_string(),
            amount: 1.0,
            time_in_force: TimeInForce::GoodTillCancelled,
            created_at: 1234567890,
            updated_at: 1234567890,
            status: OrderStatus::Pending,
        };
        
        order_manager.create_order(order).unwrap();
        
        let trade_plan = order_manager.to_trade_plan("order-1", 50000.0);
        assert!(trade_plan.is_ok());
        
        let plan = trade_plan.unwrap();
        assert_eq!(plan.chain.id, 1);
        assert_eq!(plan.amount_in, 1000000000000000000); // 1 ETH in wei
        // The min_out calculation: 1 * 50000 * 0.95 * 1e18 = 47500000000000000000000000
        // But we're dealing with token_out amount, so it should be 1 * 0.95 * 1e18 = 950000000000000000
        assert_eq!(plan.min_out, 950000000000000000); // 1 * 0.95 * 1e18
    }
}