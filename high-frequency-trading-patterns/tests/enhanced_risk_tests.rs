// Integration tests for enhanced risk management features

use hft_common::prelude::*;
use tokio::time::{interval, Duration};

// A controlled simulator for testing risk management
pub async fn risk_test_simulator(
    cfg: Cfg,
    md_tx: mpsc::Sender<Quote>,
    mut od_rx: mpsc::Receiver<Order>,
    fill_tx: mpsc::Sender<Fill>,
) {
    let mut mid = 100.00;
    let mut clock = interval(Duration::from_millis(cfg.tick_ms));
    let mut ticks = 0;

    loop {
        tokio::select! {
            _ = clock.tick() => {
                ticks += 1;
                
                // Create volatile price movements to test circuit breaker
                if ticks == 500 {
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
                
                // Stop after 1000 ticks to prevent infinite test
                if ticks > 1000 {
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
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
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
    let (md_tx, mut md_rx) = mpsc::channel::<Quote>(1024);
    let (od_tx, mut od_rx) = mpsc::channel::<Order>(1024);
    let (fill_tx, mut fills) = mpsc::channel::<Fill>(1024);
    
    // Spawn the risk test simulator
    tokio::spawn(risk_test_simulator(cfg.clone(), md_tx, od_rx, fill_tx));
    
    // Create strategy and enhanced risk components
    let mut strat = super::market_making_strategy::Logic::new(cfg.clone());
    let mut risk = super::market_making_risk::EnhancedRisk::new(&cfg);
    
    // Track results
    let mut orders_sent = 0;
    let mut fills_received = 0;
    let mut circuit_breaker_activated = false;
    let mut test_duration = 0;
    
    // Run the test loop
    loop {
        tokio::select! {
            Some(q) = md_rx.recv() => {
                // Update risk management with latest quote
                risk.on_quote(&q);
                
                if let Some(o) = strat.on_quote(&q) {
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
                strat.on_fill(&f);
                risk.on_fill(&f);
                fills_received += 1;
            },
            _ = tokio::time::sleep(Duration::from_millis(200)) => {
                // Test timeout
                test_duration += 1;
                if test_duration > 50 {
                    break;
                }
            }
        }
        
        // Early exit if we have enough data or circuit breaker activated
        if fills_received > 10 || circuit_breaker_activated {
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

// Module definitions for market making components (copied from the actual implementation)
mod market_making_strategy {
    use hft_common::prelude::*;

    pub struct Logic { 
        cfg: Cfg, 
        ctr: u64 
    }
    
    impl Logic {
        pub fn new(cfg: Cfg) -> Self { 
            Self { cfg, ctr: 0 } 
        }

        pub fn on_quote(&mut self, q: &Quote) -> Option<Order> {
            self.ctr += 1;
            // placeholder: cross every 10th quote (more frequent for testing)
            if self.ctr % 10 == 0 {
                Some(Order { 
                    side: Side::Buy, 
                    qty: 100.0, 
                    px: q.ask 
                })
            } else { 
                None 
            }
        }
        
        pub fn on_fill(&mut self, _f: &Fill) {}
    }
}

mod market_making_risk {
    use hft_common::prelude::*;
    use std::time::Instant;

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
        pnl: f64,
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
                last_price: 100.0,
                circuit_breaker_active: false,
                circuit_breaker_end: Instant::now(),
                pnl: 0.0,
            }
        }

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

        fn is_circuit_breaker_active(&self) -> bool {
            self.circuit_breaker_active && self.circuit_breaker_end.elapsed().as_secs() < self.circuit_breaker_duration
        }

        fn reset_rate_limit(&mut self) {
            if self.last_sec.elapsed().as_secs() >= 1 {
                self.last_sec = Instant::now();
                self.sent_this_sec = 0;
            }
        }

        fn check_rate_limit(&self) -> bool {
            self.sent_this_sec < self.max_orders
        }

        fn check_position_limit(&self, o: &Order) -> bool {
            let delta = if o.side == Side::Buy { o.qty } else { -o.qty };
            (self.pos + delta).abs() <= self.max_pos
        }

        fn check_order_value(&self, o: &Order) -> bool {
            let order_value = o.qty * o.px;
            order_value <= self.max_order_value
        }

        fn check_drawdown(&self) -> bool {
            self.pnl >= -self.max_drawdown
        }

        pub fn on_fill(&mut self, f: &Fill) {
            let value = f.qty * f.px;
            self.pnl += if f.side == Side::Buy { -value } else { value };
            
            // Check drawdown after updating PnL
            if !self.check_drawdown() {
                self.activate_circuit_breaker();
            }
        }

        pub fn on_quote(&mut self, q: &Quote) {
            let mid_price = (q.bid + q.ask) / 2.0;
            
            // Check for significant price changes that would trigger circuit breaker
            let price_change_pct = ((mid_price - self.last_price) / self.last_price).abs() * 100.0;
            if price_change_pct >= self.circuit_breaker_pct {
                self.activate_circuit_breaker();
            }
            
            self.last_price = mid_price;
        }

        fn activate_circuit_breaker(&mut self) {
            self.circuit_breaker_active = true;
            self.circuit_breaker_end = Instant::now();
        }

        pub fn position(&self) -> f64 {
            self.pos
        }

        pub fn get_pnl(&self) -> f64 {
            self.pnl
        }

        pub fn is_circuit_breaker_activated(&self) -> bool {
            self.circuit_breaker_active
        }
    }
}