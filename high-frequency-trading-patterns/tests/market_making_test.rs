// Integration tests for the market making pattern

use hft_common::prelude::*;

#[tokio::test]
async fn test_market_making_basic_functionality() {
    // Initialize the configuration
    let cfg = Cfg::default();
    
    // Create channels for communication
    let (md_tx, mut md_rx) = mpsc::channel::<Quote>(1024);
    let (od_tx, mut od_rx) = mpsc::channel::<Order>(1024);
    let (fill_tx, mut fills) = mpsc::channel::<Fill>(1024);
    
    // Spawn the simulator
    tokio::spawn(crate::simulator::run(cfg.clone(), md_tx, od_rx, fill_tx));
    
    // Create strategy and risk components
    let mut strat = crate::strategy::Logic::new(cfg.clone());
    let mut risk = crate::risk::Risk::new(&cfg);
    
    // Process a few quotes and check for orders
    let mut orders_sent = 0;
    let mut fills_received = 0;
    
    // Run for a limited time to collect results
    for _ in 0..1000 {
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
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                // Timeout to prevent infinite loop
                break;
            }
        }
    }
    
    // Verify that we sent some orders
    assert!(orders_sent > 0, "No orders were sent");
    assert!(fills_received > 0, "No fills were received");
}