//! pattern crate
mod simulator;
mod strategy;
mod risk;

use anyhow::Result;
use hft_common::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cfg = Cfg::default();

    let (md_tx,   mut md_rx) = mpsc::channel::<Quote>(1024);
    let (od_tx,   mut od_rx) = mpsc::channel::<Order>(1024);
    let (fill_tx, mut fills) = mpsc::channel::<Fill>(1024);

    tokio::spawn(simulator::run(cfg.clone(), md_tx, od_rx, fill_tx));

    let mut strat = strategy::Logic::new(cfg.clone());
    let mut risk  = risk::Risk::new(&cfg);

    loop {
        tokio::select! {
            Some(q) = md_rx.recv() => {
                if let Some(o) = strat.on_quote(&q) {
                    if risk.allow(&o) { od_tx.send(o).await?; }
                }
            },
            Some(f) = fills.recv() => {
                strat.on_fill(&f);
                info!("FILL {:?} {:.0} @ {:.2}", f.side, f.qty, f.px);
            },
            else => break,
        }
    }
    Ok(())
}
