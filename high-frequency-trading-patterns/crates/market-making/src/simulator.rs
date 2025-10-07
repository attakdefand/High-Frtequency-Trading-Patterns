use hft_common::prelude::*;
use tokio::time::{interval, Duration};
use std::f64::consts::PI;

/// Enhanced market simulator with realistic price dynamics
pub async fn run(
    cfg: Cfg,
    md_tx: mpsc::Sender<Quote>,
    mut od_rx: mpsc::Receiver<Order>,
    fill_tx: mpsc::Sender<Fill>,
) {
    let mut mid: f64 = 100.00;
    let mut clock = interval(Duration::from_millis(cfg.tick_ms));
    let mut ticks: u64 = 0;
    let mut volatility: f64 = 0.01; // Initial volatility
    let mut trend: f64 = 0.0; // Market trend component
    let mut cycle: f64 = 0.0; // Cyclical component

    loop {
        tokio::select! {
            _ = clock.tick() => {
                ticks += 1;
                
                // Update volatility based on market conditions (random walk)
                volatility += (fastrand::f64() - 0.5) * 0.001;
                volatility = volatility.max(0.001).min(0.1); // Keep within reasonable bounds
                
                // Update trend component (slow drift)
                trend += (fastrand::f64() - 0.5) * 0.0001;
                trend = trend.max(-0.01).min(0.01); // Limit trend strength
                
                // Update cyclical component (short-term cycles)
                cycle += 0.1;
                let cycle_effect: f64 = (cycle * 0.01).sin() * 0.001;
                
                // Brownian motion with trend and volatility
                let price_change: f64 = (fastrand::f64() - 0.5) * volatility + trend + cycle_effect;
                mid += price_change * mid; // Percentage change
                
                // Add bid-ask spread that varies with volatility
                let spread: f64 = cfg.tick_sz * (1.0 + volatility * 100.0);
                let bid_ask_adjustment: f64 = spread / 2.0;
                
                // Occasionally add market impact (large price moves)
                let impact: f64 = if fastrand::usize(0..1000) == 0 {
                    (fastrand::f64() - 0.5) * 0.05 // 5% impact
                } else {
                    0.0
                };
                
                let adjusted_mid: f64 = mid * (1.0 + impact);
                
                let q = Quote {
                    bid: adjusted_mid - bid_ask_adjustment,
                    ask: adjusted_mid + bid_ask_adjustment,
                    ts: std::time::Instant::now(),
                };
                let _ = md_tx.send(q).await;
            },
            Some(o) = od_rx.recv() => {
                // More realistic fill simulation with slippage and partial fills
                let slippage: f64 = if fastrand::usize(0..100) < 5 {
                    // 5% chance of significant slippage
                    (fastrand::f64() - 0.5) * 0.01 // Up to 1% slippage
                } else {
                    // Normal slippage based on order size relative to market
                    (o.qty / 1000.0) * 0.001 * if o.side == Side::Buy { 1.0 } else { -1.0 }
                };
                
                let fill_price: f64 = if o.side == Side::Buy {
                    o.px * (1.0 + slippage)
                } else {
                    o.px * (1.0 + slippage)
                };
                
                // Partial fills occasionally
                let fill_qty: f64 = if fastrand::usize(0..100) < 10 {
                    // 10% chance of partial fill
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