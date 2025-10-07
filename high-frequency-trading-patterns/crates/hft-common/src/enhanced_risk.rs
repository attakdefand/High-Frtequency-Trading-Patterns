use crate::prelude::*;
use std::time::Instant;

/// Enhanced Risk Management System
pub struct EnhancedRisk {
    max_pos: f64,
    max_orders: usize,
    sent_this_sec: usize,
    last_sec: Instant,
    pos: f64,
    // New risk management features
    max_drawdown: f64,
    max_order_value: f64,
    circuit_breaker_pct: f64,
    circuit_breaker_duration: u64,
    last_price: f64,
    circuit_breaker_active: bool,
    circuit_breaker_end: Instant,
    pnl: f64, // Profit and loss tracker
}

impl EnhancedRisk {
    pub fn new(cfg: &Cfg) -> Self {
        Self {
            max_pos: cfg.max_pos,
            max_orders: cfg.max_orders_s,
            sent_this_sec: 0,
            last_sec: Instant::now(),
            pos: 0.0,
            // New risk management features
            max_drawdown: cfg.max_drawdown,
            max_order_value: cfg.max_order_value,
            circuit_breaker_pct: cfg.circuit_breaker_pct,
            circuit_breaker_duration: cfg.circuit_breaker_duration,
            last_price: 100.0, // Default initial price
            circuit_breaker_active: false,
            circuit_breaker_end: Instant::now(),
            pnl: 0.0,
        }
    }

    /// Main function to check if an order is allowed
    pub fn allow(&mut self, o: &Order) -> bool {
        // Check if circuit breaker is active
        if self.is_circuit_breaker_active() {
            return false;
        }

        // Reset rate limit counter if needed
        self.reset_rate_limit();

        // Check all risk controls
        if !self.check_rate_limit() {
            return false;
        }

        if !self.check_position_limit(o) {
            return false;
        }

        if !self.check_order_value(o) {
            return false;
        }

        // Update position and counters if order is allowed
        let delta = if o.side == Side::Buy { o.qty } else { -o.qty };
        self.pos += delta;
        self.sent_this_sec += 1;

        true
    }

    /// Check if circuit breaker is active
    fn is_circuit_breaker_active(&self) -> bool {
        self.circuit_breaker_active && self.circuit_breaker_end.elapsed().as_secs() < self.circuit_breaker_duration
    }

    /// Reset rate limit counter if a second has passed
    fn reset_rate_limit(&mut self) {
        if self.last_sec.elapsed().as_secs() >= 1 {
            self.last_sec = Instant::now();
            self.sent_this_sec = 0;
        }
    }

    /// Check rate limiting
    fn check_rate_limit(&self) -> bool {
        self.sent_this_sec < self.max_orders
    }

    /// Check position limit
    fn check_position_limit(&self, o: &Order) -> bool {
        let delta = if o.side == Side::Buy { o.qty } else { -o.qty };
        (self.pos + delta).abs() <= self.max_pos
    }

    /// Check maximum order value
    fn check_order_value(&self, o: &Order) -> bool {
        let order_value = o.qty * o.px;
        order_value <= self.max_order_value
    }

    /// Check drawdown limit
    fn check_drawdown(&self) -> bool {
        self.pnl >= -self.max_drawdown
    }

    /// Update PnL based on fills
    pub fn on_fill(&mut self, f: &Fill) {
        let value = f.qty * f.px;
        self.pnl += if f.side == Side::Buy { -value } else { value };
        
        // Check drawdown after updating PnL
        if !self.check_drawdown() {
            self.activate_circuit_breaker();
        }
    }

    /// Update price for circuit breaker checks
    pub fn on_quote(&mut self, q: &Quote) {
        let mid_price = (q.bid + q.ask) / 2.0;
        
        // Check for significant price changes that would trigger circuit breaker
        let price_change_pct = ((mid_price - self.last_price) / self.last_price).abs() * 100.0;
        if price_change_pct >= self.circuit_breaker_pct {
            self.activate_circuit_breaker();
        }
        
        self.last_price = mid_price;
    }

    /// Activate circuit breaker
    fn activate_circuit_breaker(&mut self) {
        self.circuit_breaker_active = true;
        self.circuit_breaker_end = Instant::now();
    }

    /// Get current position
    pub fn position(&self) -> f64 {
        self.pos
    }

    /// Get current PnL
    pub fn get_pnl(&self) -> f64 {
        self.pnl
    }

    /// Get circuit breaker status
    pub fn is_circuit_breaker_activated(&self) -> bool {
        self.circuit_breaker_active
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_enhanced_risk_initialization() {
        let cfg = Cfg::default();
        let risk = EnhancedRisk::new(&cfg);
        
        assert_eq!(risk.position(), 0.0);
        assert_eq!(risk.get_pnl(), 0.0);
        assert!(!risk.is_circuit_breaker_activated());
    }

    #[test]
    fn test_rate_limiting() {
        let mut cfg = Cfg::default();
        cfg.max_orders_s = 2; // Set low limit for testing
        let mut risk = EnhancedRisk::new(&cfg);
        
        let order1 = Order { side: Side::Buy, qty: 100.0, px: 100.0 };
        let order2 = Order { side: Side::Buy, qty: 100.0, px: 100.0 };
        let order3 = Order { side: Side::Buy, qty: 100.0, px: 100.0 };
        
        // First two orders should be allowed
        assert!(risk.allow(&order1));
        assert!(risk.allow(&order2));
        
        // Third order should be rejected due to rate limit
        assert!(!risk.allow(&order3));
    }

    #[test]
    fn test_position_limit() {
        let mut cfg = Cfg::default();
        cfg.max_pos = 100.0; // Set low limit for testing
        let mut risk = EnhancedRisk::new(&cfg);
        
        let buy_order = Order { side: Side::Buy, qty: 150.0, px: 100.0 }; // Exceeds position limit
        let sell_order = Order { side: Side::Sell, qty: 150.0, px: 100.0 }; // Exceeds position limit
        
        // Both orders should be rejected due to position limit
        assert!(!risk.allow(&buy_order));
        assert!(!risk.allow(&sell_order));
    }

    #[test]
    fn test_order_value_limit() {
        let mut cfg = Cfg::default();
        cfg.max_order_value = 5000.0; // Set low limit for testing
        let mut risk = EnhancedRisk::new(&cfg);
        
        let large_order = Order { side: Side::Buy, qty: 100.0, px: 100.0 }; // Value = 10,000 (exceeds limit)
        let small_order = Order { side: Side::Buy, qty: 10.0, px: 100.0 }; // Value = 1,000 (within limit)
        
        // Large order should be rejected
        assert!(!risk.allow(&large_order));
        
        // Small order should be allowed
        assert!(risk.allow(&small_order));
    }

    #[test]
    fn test_circuit_breaker_activation_by_price_change() {
        let cfg = Cfg::default();
        let mut risk = EnhancedRisk::new(&cfg);
        
        // Initial quote
        let quote1 = Quote { bid: 100.0, ask: 101.0, ts: Instant::now() };
        risk.on_quote(&quote1);
        
        // Quote with large price change (10% change should trigger 5% circuit breaker)
        let quote2 = Quote { bid: 110.0, ask: 111.0, ts: Instant::now() };
        risk.on_quote(&quote2);
        
        // Circuit breaker should now be activated
        assert!(risk.is_circuit_breaker_activated());
        
        // Orders should be rejected when circuit breaker is active
        let order = Order { side: Side::Buy, qty: 100.0, px: 100.0 };
        assert!(!risk.allow(&order));
    }

    #[test]
    fn test_circuit_breaker_activation_by_drawdown() {
        let cfg = Cfg::default();
        let mut risk = EnhancedRisk::new(&cfg);
        
        // Simulate a large loss that exceeds drawdown limit
        let large_loss_fill = Fill { side: Side::Buy, qty: 100.0, px: 100.0, ts: Instant::now() }; // Buy at 100
        risk.on_fill(&large_loss_fill);
        
        let large_loss_fill2 = Fill { side: Side::Sell, qty: 100.0, px: 50.0, ts: Instant::now() }; // Sell at 50 = $5000 loss
        risk.on_fill(&large_loss_fill2);
        
        // Circuit breaker should now be activated due to drawdown
        assert!(risk.is_circuit_breaker_activated());
    }

    #[test]
    fn test_pnl_calculation() {
        let cfg = Cfg::default();
        let mut risk = EnhancedRisk::new(&cfg);
        
        // Buy 100 shares at $100
        let buy_fill = Fill { side: Side::Buy, qty: 100.0, px: 100.0, ts: Instant::now() };
        risk.on_fill(&buy_fill);
        assert_eq!(risk.get_pnl(), -10000.0); // -$10,000 (cost of purchase)
        
        // Sell 100 shares at $110
        let sell_fill = Fill { side: Side::Sell, qty: 100.0, px: 110.0, ts: Instant::now() };
        risk.on_fill(&sell_fill);
        assert_eq!(risk.get_pnl(), 1000.0); // +$1,000 profit
    }
}

// Integration tests
#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::{interval, sleep};

    // A controlled simulator for testing risk management
    async fn risk_test_simulator(
        cfg: Cfg,
        md_tx: tokio::sync::mpsc::Sender<Quote>,
        mut od_rx: tokio::sync::mpsc::Receiver<Order>, // Kept mut because we need to call recv()
        fill_tx: tokio::sync::mpsc::Sender<Fill>,
    ) {
        let mut mid = 100.00;
        let mut clock = interval(Duration::from_millis(cfg.tick_ms));
        let mut ticks = 0;

        loop {
            tokio::select! {
                _ = clock.tick() => {
                    ticks += 1;
                    
                    // Create volatile price movements to test circuit breaker
                    if ticks == 50 {
                        // Large price jump to trigger circuit breaker
                        mid = 150.00;
                    } else {
                        // Normal price movement
                        mid += cfg.tick_sz;
                    }
                    
                    let q = Quote {
                        bid: mid - cfg.tick_sz/2.0,
                        ask: mid + cfg.tick_sz/2.0,
                        ts: std::time::Instant::now(),
                    };
                    let _ = md_tx.send(q).await;
                    
                    // Stop after 100 ticks to prevent infinite test
                    if ticks > 100 {
                        break;
                    }
                },
                Some(o) = od_rx.recv() => {
                    // Immediately fill all orders for testing
                    let f = Fill { 
                        side: o.side, 
                        qty: o.qty, 
                        px: o.px, 
                        ts: std::time::Instant::now() 
                    };
                    let _ = fill_tx.send(f).await;
                },
                _ = sleep(Duration::from_millis(100)) => {
                    // Timeout to prevent infinite loop
                    break;
                }
            }
        }
    }

    #[tokio::test]
    async fn test_enhanced_risk_integration() {
        // Initialize configuration with tight risk limits for testing
        let mut cfg = Cfg::default();
        cfg.max_orders_s = 5; // Very low limit for testing
        cfg.circuit_breaker_pct = 1.0; // 1% price change triggers circuit breaker
        
        // Create channels
        let (md_tx, mut md_rx) = tokio::sync::mpsc::channel::<Quote>(1024);
        let (od_tx, mut od_rx) = tokio::sync::mpsc::channel::<Order>(1024);
        let (fill_tx, mut fills) = tokio::sync::mpsc::channel::<Fill>(1024);
        
        // Spawn the risk test simulator
        tokio::spawn(risk_test_simulator(cfg.clone(), md_tx, od_rx, fill_tx));
        
        // Create enhanced risk component
        let mut risk = EnhancedRisk::new(&cfg);
        
        // Track results
        let mut orders_sent = 0;
        let mut fills_received = 0;
        let mut circuit_breaker_activated = false;
        let mut test_duration = 0;
        
        // Simple strategy that sends orders
        let mut counter = 0;
        
        // Run the test loop
        loop {
            tokio::select! {
                Some(q) = md_rx.recv() => {
                    // Update risk management with latest quote
                    risk.on_quote(&q);
                    
                    counter += 1;
                    // Send an order every 5 ticks
                    if counter % 5 == 0 {
                        let o = Order { side: Side::Buy, qty: 100.0, px: q.ask };
                        if risk.allow(&o) {
                            od_tx.send(o).await.unwrap();
                            orders_sent += 1;
                        } else {
                            // Check if circuit breaker was activated
                            if risk.is_circuit_breaker_activated() {
                                circuit_breaker_activated = true;
                            }
                        }
                    }
                },
                Some(f) = fills.recv() => {
                    risk.on_fill(&f);
                    fills_received += 1;
                },
                _ = sleep(Duration::from_millis(200)) => {
                    // Test timeout
                    test_duration += 1;
                    if test_duration > 10 {
                        break;
                    }
                }
            }
            
            // Early exit if we have enough data or circuit breaker activated
            if fills_received > 5 || circuit_breaker_activated {
                break;
            }
        }
        
        // Verify results
        assert!(orders_sent > 0, "No orders were sent");
        assert!(fills_received > 0, "No fills were received");
        
        // With the tight rate limit, we should hit the rate limit
        assert!(orders_sent <= 5, "Rate limit was not enforced");
        
        // With the large price jump, circuit breaker should be activated
        assert!(circuit_breaker_activated, "Circuit breaker was not activated");
    }
}