use hft_common::prelude::*;
use tokio::time::{interval, Duration};

pub async fn run(
    cfg: Cfg,
    md_tx: mpsc::Sender<Quote>,
    mut od_rx: mpsc::Receiver<Order>,
    fill_tx: mpsc::Sender<Fill>,
) {
    let mut mid = 100.00;
    let mut clock = interval(Duration::from_millis(cfg.tick_ms));

    loop {
        tokio::select! {
            _ = clock.tick() => {
                // simple deterministic walk (no RNG -> Send-safe)
                mid += cfg.tick_sz;
                let q = Quote {
                    bid: mid - cfg.tick_sz/2.0,
                    ask: mid + cfg.tick_sz/2.0,
                    ts:  std::time::Instant::now(),
                };
                let _ = md_tx.send(q).await;
            },
            Some(o) = od_rx.recv() => {
                let f = Fill { side: o.side, qty: o.qty, px: o.px, ts: std::time::Instant::now() };
                let _ = fill_tx.send(f).await;
            },
            else => break,
        }
    }
}
