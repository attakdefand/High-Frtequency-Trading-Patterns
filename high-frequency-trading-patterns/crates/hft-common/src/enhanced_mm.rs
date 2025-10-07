use crate::prelude::*;
use std::collections::VecDeque;

/// Enhanced Market Making Strategy with advanced features
pub struct EnhancedMarketMaking {
    cfg: Cfg,
    inventory: f64, // Current inventory position
    quotes_received: u64,
    // Inventory management
    target_inventory: f64,
    max_inventory: f64,
    min_inventory: f64,
    // Spread optimization
    base_spread: f64,
    min_spread: f64,
    spread_multiplier: f64,
    // Queue position tracking
    queue_positions: VecDeque<f64>,
    queue_window_size: usize,
    // Performance tracking
    total_pnl: f64,
    trades_count: u64,
}

impl EnhancedMarketMaking {
    pub fn new(cfg: Cfg) -> Self {
        Self {
            cfg: cfg.clone(),
            inventory: 0.0,
            quotes_received: 0,
            // Inventory management
            target_inventory: 0.0,
            max_inventory: cfg.max_pos,
            min_inventory: -cfg.max_pos,
            // Spread optimization
            base_spread: cfg.tick_sz * 2.0, // Start with 2 ticks
            min_spread: cfg.tick_sz,        // Minimum 1 tick
            spread_multiplier: 1.0,
            // Queue position tracking
            queue_positions: VecDeque::with_capacity(100),
            queue_window_size: 100,
            // Performance tracking
            total_pnl: 0.0,
            trades_count: 0,
        }
    }

    /// Main function to process quotes and generate orders
    pub fn on_quote(&mut self, q: &Quote) -> Vec<Order> {
        self.quotes_received += 1;
        
        // Update queue position tracking
        self.update_queue_position(q);
        
        // Adjust spreads based on market conditions
        let adjusted_spread = self.calculate_dynamic_spread(q);
        
        // Calculate inventory-adjusted quotes
        let (bid_price, ask_price) = self.calculate_inventory_adjusted_quotes(q, adjusted_spread);
        
        // Generate orders
        let mut orders = Vec::new();
        
        // Add bid order if within inventory limits
        if self.inventory > self.min_inventory {
            orders.push(Order {
                side: Side::Buy,
                qty: self.calculate_order_size(Side::Buy),
                px: bid_price,
            });
        }
        
        // Add ask order if within inventory limits
        if self.inventory < self.max_inventory {
            orders.push(Order {
                side: Side::Sell,
                qty: self.calculate_order_size(Side::Sell),
                px: ask_price,
            });
        }
        
        orders
    }

    /// Update queue position tracking based on quote changes
    fn update_queue_position(&mut self, q: &Quote) {
        let mid_price = (q.bid + q.ask) / 2.0;
        self.queue_positions.push_back(mid_price);
        
        // Maintain window size
        if self.queue_positions.len() > self.queue_window_size {
            self.queue_positions.pop_front();
        }
    }

    /// Calculate dynamic spread based on market volatility
    fn calculate_dynamic_spread(&self, q: &Quote) -> f64 {
        // Calculate volatility from recent price movements
        let volatility = self.calculate_volatility();
        
        // Adjust spread based on volatility
        let vol_multiplier = 1.0 + volatility * self.spread_multiplier;
        let dynamic_spread = self.base_spread * vol_multiplier;
        
        // Ensure spread doesn't go below minimum
        dynamic_spread.max(self.min_spread)
    }

    /// Calculate volatility from recent price movements
    fn calculate_volatility(&self) -> f64 {
        if self.queue_positions.len() < 2 {
            return 0.0;
        }
        
        let prices: Vec<f64> = self.queue_positions.iter().copied().collect();
        let mean: f64 = prices.iter().sum::<f64>() / prices.len() as f64;
        
        let variance = prices
            .iter()
            .map(|price| (price - mean).powi(2))
            .sum::<f64>() / prices.len() as f64;
            
        variance.sqrt() // Standard deviation
    }

    /// Calculate inventory-adjusted quotes
    fn calculate_inventory_adjusted_quotes(&self, q: &Quote, spread: f64) -> (f64, f64) {
        // Adjust quotes based on inventory position
        let inventory_skew = self.calculate_inventory_skew();
        
        let bid_price = q.bid - spread / 2.0 - inventory_skew;
        let ask_price = q.ask + spread / 2.0 + inventory_skew;
        
        (bid_price, ask_price)
    }

    /// Calculate inventory skew to adjust quotes
    fn calculate_inventory_skew(&self) -> f64 {
        // Normalize inventory position to [-1, 1] range
        let normalized_inventory = (self.inventory - self.target_inventory) / self.max_inventory;
        
        // Apply skew factor (adjust based on how far we are from target)
        normalized_inventory * self.base_spread * 2.0
    }

    /// Calculate order size based on side and current conditions
    fn calculate_order_size(&self, side: Side) -> f64 {
        // Base size
        let base_size = 100.0;
        
        // Adjust size based on inventory position
        let inventory_factor = self.calculate_inventory_size_factor(side);
        
        // Adjust size based on recent fill rate
        let fill_rate_factor = self.calculate_fill_rate_factor();
        
        base_size * inventory_factor * fill_rate_factor
    }

    /// Calculate inventory size factor
    fn calculate_inventory_size_factor(&self, side: Side) -> f64 {
        let normalized_inventory = (self.inventory - self.target_inventory) / self.max_inventory;
        
        match side {
            Side::Buy => {
                // Reduce buy size when inventory is high
                1.0 - normalized_inventory.abs().min(1.0) * 0.5
            }
            Side::Sell => {
                // Reduce sell size when inventory is low
                1.0 - normalized_inventory.abs().min(1.0) * 0.5
            }
        }
    }

    /// Calculate fill rate factor based on recent trading activity
    fn calculate_fill_rate_factor(&self) -> f64 {
        // For now, return neutral factor
        // In a more advanced implementation, this would track recent fill rates
        1.0
    }

    /// Process fills to update inventory and PnL
    pub fn on_fill(&mut self, f: &Fill) {
        // Update inventory
        match f.side {
            Side::Buy => self.inventory += f.qty,
            Side::Sell => self.inventory -= f.qty,
        }
        
        // Update PnL
        let value = f.qty * f.px;
        match f.side {
            Side::Buy => self.total_pnl -= value,
            Side::Sell => self.total_pnl += value,
        }
        
        self.trades_count += 1;
    }

    /// Get current inventory position
    pub fn inventory(&self) -> f64 {
        self.inventory
    }

    /// Get current PnL
    pub fn pnl(&self) -> f64 {
        self.total_pnl
    }

    /// Get quotes received count
    pub fn quotes_received(&self) -> u64 {
        self.quotes_received
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_mm_initialization() {
        let cfg = Cfg::default();
        let mm = EnhancedMarketMaking::new(cfg.clone());
        
        assert_eq!(mm.inventory(), 0.0);
        assert_eq!(mm.pnl(), 0.0);
        assert_eq!(mm.quotes_received(), 0);
        assert_eq!(mm.max_inventory, cfg.max_pos);
        assert_eq!(mm.min_inventory, -cfg.max_pos);
    }

    #[test]
    fn test_inventory_management() {
        let cfg = Cfg::default();
        let mut mm = EnhancedMarketMaking::new(cfg);
        
        // Test buying
        let buy_fill = Fill {
            side: Side::Buy,
            qty: 100.0,
            px: 100.0,
            ts: std::time::Instant::now(),
        };
        mm.on_fill(&buy_fill);
        assert_eq!(mm.inventory(), 100.0);
        
        // Test selling
        let sell_fill = Fill {
            side: Side::Sell,
            qty: 50.0,
            px: 101.0,
            ts: std::time::Instant::now(),
        };
        mm.on_fill(&sell_fill);
        assert_eq!(mm.inventory(), 50.0);
    }

    #[test]
    fn test_pnl_calculation() {
        let cfg = Cfg::default();
        let mut mm = EnhancedMarketMaking::new(cfg);
        
        // Buy 100 shares at $100
        let buy_fill = Fill {
            side: Side::Buy,
            qty: 100.0,
            px: 100.0,
            ts: std::time::Instant::now(),
        };
        mm.on_fill(&buy_fill);
        assert_eq!(mm.pnl(), -10000.0); // -$10,000 (cost of purchase)
        
        // Sell 100 shares at $101
        let sell_fill = Fill {
            side: Side::Sell,
            qty: 100.0,
            px: 101.0,
            ts: std::time::Instant::now(),
        };
        mm.on_fill(&sell_fill);
        assert_eq!(mm.pnl(), 100.0); // +$100 profit
    }

    #[test]
    fn test_quote_processing() {
        let cfg = Cfg::default();
        let mut mm = EnhancedMarketMaking::new(cfg);
        
        let quote = Quote {
            bid: 99.50,
            ask: 100.50,
            ts: std::time::Instant::now(),
        };
        
        let orders = mm.on_quote(&quote);
        assert_eq!(orders.len(), 2); // Should generate bid and ask orders
        assert_eq!(mm.quotes_received(), 1);
    }

    #[test]
    fn test_inventory_limits() {
        let mut cfg = Cfg::default();
        cfg.max_pos = 100.0;
        let mut mm = EnhancedMarketMaking::new(cfg);
        
        // Set inventory to maximum
        mm.inventory = 100.0;
        
        let quote = Quote {
            bid: 99.50,
            ask: 100.50,
            ts: std::time::Instant::now(),
        };
        
        let orders = mm.on_quote(&quote);
        // Should only generate bid order since we're at max inventory
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].side, Side::Buy);
    }

    #[test]
    fn test_volatility_calculation() {
        let cfg = Cfg::default();
        let mm = EnhancedMarketMaking::new(cfg);
        
        // Test with no data
        let volatility = mm.calculate_volatility();
        assert_eq!(volatility, 0.0);
    }

    #[test]
    fn test_inventory_skew() {
        let cfg = Cfg::default();
        let mm = EnhancedMarketMaking::new(cfg);
        
        // Test with zero inventory (no skew)
        let skew = mm.calculate_inventory_skew();
        assert_eq!(skew, 0.0);
    }
}