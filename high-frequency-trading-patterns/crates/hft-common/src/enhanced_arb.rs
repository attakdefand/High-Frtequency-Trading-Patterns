use crate::prelude::*;

/// Enhanced Arbitrage Strategy for various arbitrage patterns
pub struct EnhancedArbitrage {
    cfg: Cfg,
    quotes_processed: u64,
    // Performance tracking
    total_pnl: f64,
    trades_count: u64,
    // Arbitrage-specific fields
    min_profit_threshold: f64,
    max_position: f64,
    current_position: f64,
    // Latency tracking
    last_quote_time: Option<std::time::Instant>,
    latency_stats: LatencyStats,
}

/// Statistics for tracking latency
struct LatencyStats {
    total_latency: u128,
    quotes_count: u64,
    min_latency: u128,
    max_latency: u128,
}

/// Types of arbitrage strategies
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArbitrageType {
    Statistical,
    IndexEtfBasis,
    TriangularCrossExchange,
    Latency,
}

impl EnhancedArbitrage {
    pub fn new(cfg: Cfg, arb_type: ArbitrageType) -> Self {
        Self {
            cfg: cfg.clone(),
            quotes_processed: 0,
            total_pnl: 0.0,
            trades_count: 0,
            min_profit_threshold: 0.01, // Minimum profit threshold
            max_position: cfg.max_pos,
            current_position: 0.0,
            last_quote_time: None,
            latency_stats: LatencyStats {
                total_latency: 0,
                quotes_count: 0,
                min_latency: u128::MAX,
                max_latency: 0,
            },
        }
    }

    /// Process quotes for statistical arbitrage
    pub fn on_statistical_arbitrage_quote(&mut self, q: &Quote, fair_price: f64) -> Option<Order> {
        self.quotes_processed += 1;
        self.update_latency_stats();
        
        // Calculate spread from fair price
        let mid_price = (q.bid + q.ask) / 2.0;
        let price_diff = mid_price - fair_price;
        
        // Check if there's a profitable opportunity
        if price_diff.abs() > self.min_profit_threshold {
            // Determine order side based on price deviation
            let side = if price_diff > 0.0 {
                // Price is above fair value, sell
                Side::Sell
            } else {
                // Price is below fair value, buy
                Side::Buy
            };
            
            // Calculate order size based on deviation and position limits
            let size = self.calculate_statistical_arb_size(price_diff);
            
            // Check position limits
            if self.check_position_limit(side, size) {
                Some(Order {
                    side,
                    qty: size,
                    px: if side == Side::Buy { q.ask } else { q.bid },
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Process quotes for index/ETF basis arbitrage
    pub fn on_index_etf_basis_quote(&mut self, etf_quote: &Quote, index_value: f64, creation_fee: f64, redemption_fee: f64) -> Option<Vec<Order>> {
        self.quotes_processed += 1;
        self.update_latency_stats();
        
        let etf_mid = (etf_quote.bid + etf_quote.ask) / 2.0;
        
        // Calculate basis (ETF price - Index value)
        let basis = etf_mid - index_value;
        
        let mut orders = Vec::new();
        
        // Check for creation arbitrage (buy index, sell ETF)
        let creation_cost = index_value + creation_fee;
        if etf_mid > creation_cost + self.min_profit_threshold {
            // Sell ETF, buy index components
            if self.check_position_limit(Side::Sell, 100.0) {
                orders.push(Order {
                    side: Side::Sell,
                    qty: 100.0,
                    px: etf_quote.bid,
                });
            }
        }
        
        // Check for redemption arbitrage (buy ETF, sell index)
        let redemption_value = index_value - redemption_fee;
        if etf_mid < redemption_value - self.min_profit_threshold {
            // Buy ETF, sell index components
            if self.check_position_limit(Side::Buy, 100.0) {
                orders.push(Order {
                    side: Side::Buy,
                    qty: 100.0,
                    px: etf_quote.ask,
                });
            }
        }
        
        if orders.is_empty() {
            None
        } else {
            Some(orders)
        }
    }

    /// Process quotes for triangular cross-exchange arbitrage
    pub fn on_triangular_arb_quote(&mut self, quote1: &Quote, quote2: &Quote, quote3: &Quote) -> Option<Vec<Order>> {
        self.quotes_processed += 1;
        self.update_latency_stats();
        
        // Calculate triangular arbitrage opportunity
        // This is a simplified example - real implementation would be more complex
        let rate1 = quote1.ask; // Buy asset 1 with asset 2
        let rate2 = quote2.ask; // Buy asset 2 with asset 3
        let rate3 = quote3.bid; // Sell asset 3 for asset 1
        
        // Calculate round-trip profit (assuming 1 unit of asset 1)
        let round_trip = (1.0 / rate1) / rate2 * rate3;
        let profit = round_trip - 1.0;
        
        let mut orders = Vec::new();
        
        if profit > self.min_profit_threshold {
            // Execute triangular arbitrage
            if self.check_position_limit(Side::Buy, 100.0) {
                orders.push(Order {
                    side: Side::Buy,
                    qty: 100.0,
                    px: quote1.ask,
                });
                orders.push(Order {
                    side: Side::Buy,
                    qty: 100.0 / rate1,
                    px: quote2.ask,
                });
                orders.push(Order {
                    side: Side::Sell,
                    qty: (100.0 / rate1) / rate2,
                    px: quote3.bid,
                });
            }
        }
        
        if orders.is_empty() {
            None
        } else {
            Some(orders)
        }
    }

    /// Process fills to update PnL and position
    pub fn on_fill(&mut self, f: &Fill) {
        // Update position
        match f.side {
            Side::Buy => self.current_position += f.qty,
            Side::Sell => self.current_position -= f.qty,
        }
        
        // Update PnL
        let value = f.qty * f.px;
        match f.side {
            Side::Buy => self.total_pnl -= value,
            Side::Sell => self.total_pnl += value,
        }
        
        self.trades_count += 1;
    }

    /// Calculate order size for statistical arbitrage
    fn calculate_statistical_arb_size(&self, price_diff: f64) -> f64 {
        // Base size
        let base_size = 100.0;
        
        // Adjust size based on deviation magnitude
        let deviation_factor = (price_diff.abs() / self.min_profit_threshold).min(5.0);
        
        // Adjust size based on available position
        let position_factor = (self.max_position - self.current_position.abs()) / self.max_position;
        
        base_size * deviation_factor * position_factor.max(0.1)
    }

    /// Check if order is within position limits
    fn check_position_limit(&self, side: Side, size: f64) -> bool {
        let new_position = match side {
            Side::Buy => self.current_position + size,
            Side::Sell => self.current_position - size,
        };
        
        new_position.abs() <= self.max_position
    }

    /// Update latency statistics
    fn update_latency_stats(&mut self) {
        let now = std::time::Instant::now();
        
        if let Some(last_time) = self.last_quote_time {
            let latency = now.duration_since(last_time).as_micros();
            self.latency_stats.total_latency += latency;
            self.latency_stats.quotes_count += 1;
            self.latency_stats.min_latency = self.latency_stats.min_latency.min(latency);
            self.latency_stats.max_latency = self.latency_stats.max_latency.max(latency);
        }
        
        self.last_quote_time = Some(now);
    }

    /// Get current PnL
    pub fn pnl(&self) -> f64 {
        self.total_pnl
    }

    /// Get current position
    pub fn position(&self) -> f64 {
        self.current_position
    }

    /// Get quotes processed count
    pub fn quotes_processed(&self) -> u64 {
        self.quotes_processed
    }

    /// Get average latency in microseconds
    pub fn average_latency(&self) -> f64 {
        if self.latency_stats.quotes_count > 0 {
            self.latency_stats.total_latency as f64 / self.latency_stats.quotes_count as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_arbitrage_initialization() {
        let cfg = Cfg::default();
        let arb = EnhancedArbitrage::new(cfg.clone(), ArbitrageType::Statistical);
        
        assert_eq!(arb.pnl(), 0.0);
        assert_eq!(arb.position(), 0.0);
        assert_eq!(arb.quotes_processed(), 0);
        assert_eq!(arb.average_latency(), 0.0);
    }

    #[test]
    fn test_statistical_arbitrage() {
        let cfg = Cfg::default();
        let mut arb = EnhancedArbitrage::new(cfg, ArbitrageType::Statistical);
        
        let quote = Quote {
            bid: 99.50,
            ask: 100.50,
            ts: std::time::Instant::now(),
        };
        
        // Test when price is above fair value
        let order = arb.on_statistical_arbitrage_quote(&quote, 99.0);
        assert!(order.is_some());
        assert_eq!(order.unwrap().side, Side::Sell);
        
        // Test when price is below fair value
        let order = arb.on_statistical_arbitrage_quote(&quote, 101.0);
        assert!(order.is_some());
        assert_eq!(order.unwrap().side, Side::Buy);
        
        // Test when price is at fair value (no opportunity)
        let order = arb.on_statistical_arbitrage_quote(&quote, 100.0);
        assert!(order.is_none());
    }

    #[test]
    fn test_position_management() {
        let mut cfg = Cfg::default();
        cfg.max_pos = 100.0;
        let mut arb = EnhancedArbitrage::new(cfg, ArbitrageType::Statistical);
        
        // Test buying
        let buy_fill = Fill {
            side: Side::Buy,
            qty: 50.0,
            px: 100.0,
            ts: std::time::Instant::now(),
        };
        arb.on_fill(&buy_fill);
        assert_eq!(arb.position(), 50.0);
        
        // Test selling
        let sell_fill = Fill {
            side: Side::Sell,
            qty: 25.0,
            px: 101.0,
            ts: std::time::Instant::now(),
        };
        arb.on_fill(&sell_fill);
        assert_eq!(arb.position(), 25.0);
    }

    #[test]
    fn test_pnl_calculation() {
        let cfg = Cfg::default();
        let mut arb = EnhancedArbitrage::new(cfg, ArbitrageType::Statistical);
        
        // Buy 100 shares at $100
        let buy_fill = Fill {
            side: Side::Buy,
            qty: 100.0,
            px: 100.0,
            ts: std::time::Instant::now(),
        };
        arb.on_fill(&buy_fill);
        assert_eq!(arb.pnl(), -10000.0); // -$10,000 (cost of purchase)
        
        // Sell 100 shares at $101
        let sell_fill = Fill {
            side: Side::Sell,
            qty: 100.0,
            px: 101.0,
            ts: std::time::Instant::now(),
        };
        arb.on_fill(&sell_fill);
        assert_eq!(arb.pnl(), 100.0); // +$100 profit
    }
}