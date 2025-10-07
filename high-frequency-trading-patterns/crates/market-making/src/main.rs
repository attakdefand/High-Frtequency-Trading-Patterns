//! pattern crate
mod simulator;
mod strategy;
mod risk;

use anyhow::Result;
use hft_common::prelude::*;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cfg = Cfg::default();
    
    // Create performance monitor
    let monitor = PerformanceMonitor::new("market-making");

    let (md_tx,   mut md_rx) = mpsc::channel::<Quote>(1024);
    let (od_tx,   mut od_rx) = mpsc::channel::<Order>(1024);
    let (fill_tx, mut fills) = mpsc::channel::<Fill>(1024);

    tokio::spawn(simulator::run(cfg.clone(), md_tx, od_rx, fill_tx));

    let mut strat = strategy::Logic::new(cfg.clone());
    let mut risk  = EnhancedRisk::new(&cfg); // Use enhanced risk management

    // Log metrics every 1000 ticks
    let mut tick_count = 0;
    
    loop {
        tokio::select! {
            Some(q) = md_rx.recv() => {
                let start_time = Instant::now();
                
                // Record quote processing
                monitor.record_quote();
                
                // Update risk management with latest quote for circuit breaker checks
                risk.on_quote(&q);
                
                if let Some(o) = strat.on_quote(&q) {
                    if risk.allow(&o) { 
                        od_tx.send(o).await?; 
                        monitor.record_order();
                    } else {
                        info!("Order rejected by risk management");
                    }
                }
                
                // Record latency
                let latency = start_time.elapsed().as_micros();
                monitor.record_latency(latency);
                
                // Log metrics periodically
                tick_count += 1;
                if tick_count % 1000 == 0 {
                    monitor.log_metrics();
                }
            },
            Some(f) = fills.recv() => {
                let start_time = Instant::now();
                
                strat.on_fill(&f);
                risk.on_fill(&f); // Update risk management with fill information
                monitor.record_fill(&f);
                
                info!("FILL {:?} {:.0} @ {:.2}", f.side, f.qty, f.px);
                
                // Record latency for fill processing
                let latency = start_time.elapsed().as_micros();
                monitor.record_latency(latency);
            },
            else => break,
        }
    }
    
    // Log final metrics
    monitor.log_metrics();
    Ok(())
}