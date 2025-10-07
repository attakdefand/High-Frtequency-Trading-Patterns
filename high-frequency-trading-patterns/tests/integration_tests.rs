// Integration tests for HFT patterns

// Test the market making pattern with a controlled simulator
#[cfg(test)]
mod market_making_tests {
    use hft_common::prelude::*;
    use tokio::time::{interval, Duration};

    // A controlled simulator for testing
    pub async fn controlled_simulator(
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
                    // Simple deterministic walk
                    mid += cfg.tick_sz;
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
    async fn test_market_making_strategy() {
        // Initialize configuration
        let cfg = Cfg::default();
        
        // Create channels
        let (md_tx, mut md_rx) = mpsc::channel::<Quote>(1024);
        let (od_tx, mut od_rx) = mpsc::channel::<Order>(1024);
        let (fill_tx, mut fills) = mpsc::channel::<Fill>(1024);
        
        // Spawn the controlled simulator
        tokio::spawn(controlled_simulator(cfg.clone(), md_tx, od_rx, fill_tx));
        
        // Create strategy and risk components
        let mut strat = super::market_making_strategy::Logic::new(cfg.clone());
        let mut risk = super::market_making_risk::Risk::new(&cfg);
        
        // Track results
        let mut orders_sent = 0;
        let mut fills_received = 0;
        let mut test_duration = 0;
        
        // Run the test loop
        loop {
            tokio::select! {
                Some(q) = md_rx.recv() => {
                    if let Some(o) = strat.on_quote(&q) {
                        if risk.allow(&o) {
                            od_tx.send(o).await.unwrap();
                            orders_sent += 1;
                        }
                    }
                },
                Some(_f) = fills.recv() => {
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
            
            // Early exit if we have enough data
            if fills_received > 10 {
                break;
            }
        }
        
        // Verify results
        assert!(orders_sent > 0, "No orders were sent");
        assert!(fills_received > 0, "No fills were received");
    }
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
            // placeholder: cross every 256th quote
            if self.ctr % 256 == 0 {
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

    pub struct Risk {
        max_pos: f64,
        max_orders: usize,
        sent_this_sec: usize,
        last_sec: Instant,
        pos: f64,
    }
    
    impl Risk {
        pub fn new(cfg: &Cfg) -> Self {
            Self { 
                max_pos: cfg.max_pos, 
                max_orders: cfg.max_orders_s,
                sent_this_sec: 0, 
                last_sec: Instant::now(), 
                pos: 0.0 
            }
        }
        
        pub fn allow(&mut self, o: &Order) -> bool {
            if self.last_sec.elapsed().as_secs() >= 1 {
                self.last_sec = Instant::now(); 
                self.sent_this_sec = 0;
            }
            if self.sent_this_sec >= self.max_orders { 
                return false; 
            }
            let delta = if o.side == Side::Buy { 
                o.qty 
            } else { 
                -o.qty 
            };
            if (self.pos + delta).abs() > self.max_pos { 
                return false; 
            }
            self.pos += delta; 
            self.sent_this_sec += 1; 
            true
        }
    }
}