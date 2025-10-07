use hft_common::prelude::*;
use tokio::time::{interval, Duration};
use std::f64::consts::PI;

/// Enhanced statistical arbitrage simulator with mean-reverting price series
pub async fn run(
    cfg: Cfg,
    md_tx: mpsc::Sender<Quote>,
    mut od_rx: mpsc::Receiver<Order>,
    fill_tx: mpsc::Sender<Fill>,
) {
    let mut mid: f64 = 100.00;
    let mut fair_price: f64 = 100.00; // The "fair" price that mid reverts to
    let mut clock = interval(Duration::from_millis(cfg.tick_ms));
    let mut ticks: u64 = 0;
    let mut volatility: f64 = 0.01; // Initial volatility
    let mut cycle: f64 = 0.0; // Cyclical component for fair price

    loop {
        tokio::select! {
            _ = clock.tick() => {
                ticks += 1;
                
                // Update volatility
                volatility += (fastrand::f64() - 0.5) * 0.001;
                volatility = volatility.max(0.001).min(0.1);
                
                // Update fair price with cyclical and trend components
                cycle += 0.05;
                let cycle_effect: f64 = (cycle * 0.01).sin() * 0.005;
                let trend: f64 = 0.00001 * (ticks as f64); // Slow upward drift
                fair_price += cycle_effect + trend;
                
                // Mid price mean-reverts to fair price with some noise
                let reversion_speed: f64 = 0.05; // How quickly price reverts
                let noise: f64 = (fastrand::f64() - 0.5) * volatility;
                let reversion: f64 = (fair_price - mid) * reversion_speed;
                mid += reversion + noise * mid;
                
                // Occasionally create arbitrage opportunities (price deviates from fair value)
                let arbitrage_opportunity = if fastrand::usize(0..500) == 0 {
                    // 0.2% chance per tick of arbitrage opportunity
                    (fastrand::f64() - 0.5) * 0.02 // Up to 2% deviation
                } else {
                    0.0
                };
                
                let adjusted_mid: f64 = mid * (1.0 + arbitrage_opportunity);
                
                // Add bid-ask spread that varies with volatility
                let spread: f64 = cfg.tick_sz * (1.0 + volatility * 50.0);
                let bid_ask_adjustment: f64 = spread / 2.0;
                
                let q = Quote {
                    bid: adjusted_mid - bid_ask_adjustment,
                    ask: adjusted_mid + bid_ask_adjustment,
                    ts: std::time::Instant::now(),
                };
                let _ = md_tx.send(q).await;
            },
            Some(o) = od_rx.recv() => {
                // Statistical arbitrage fill simulation
                let slippage: f64 = if fastrand::usize(0..100) < 3 {
                    // 3% chance of significant slippage
                    (fastrand::f64() - 0.5) * 0.005 // Up to 0.5% slippage
                } else {
                    // Normal slippage
                    (o.qty / 500.0) * 0.0005 * if o.side == Side::Buy { 1.0 } else { -1.0 }
                };
                
                let fill_price: f64 = if o.side == Side::Buy {
                    o.px * (1.0 + slippage)
                } else {
                    o.px * (1.0 + slippage)
                };
                
                // Partial fills occasionally
                let fill_qty: f64 = if fastrand::usize(0..100) < 5 {
                    // 5% chance of partial fill
                    o.qty * fastrand::f64()
                } else {
                    o.qty
                };
                
                let f = Fill { 
                    side: o.side, 
                    qty: fill_qty, 
                    px: fill_price, 
                    ts: std::time::Instant::now() 
                };
                let _ = fill_tx.send(f).await;
            },
            else => break,
        }
    }
}