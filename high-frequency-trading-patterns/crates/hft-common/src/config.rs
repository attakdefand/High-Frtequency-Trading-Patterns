use serde::Deserialize;

/// Minimal config (kept pure-Rust so all crates compile immediately)
#[derive(Debug, Clone, Deserialize)]
pub struct Cfg {
    pub symbol:  String,
    pub tick_ms: u64,
    pub tick_sz: f64,
    pub max_pos: f64,
    pub max_orders_s: usize,
    // New risk management parameters
    pub max_drawdown: f64,        // Maximum allowed drawdown in currency units
    pub max_order_value: f64,     // Maximum value of a single order (qty * price)
    pub circuit_breaker_pct: f64, // Percentage change that triggers circuit breaker
    pub circuit_breaker_duration: u64, // Duration in seconds for circuit breaker
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            symbol: "XYZ".into(),
            tick_ms: 1,
            tick_sz: 0.01,
            max_pos: 10_000.0,
            max_orders_s: 50_000,
            // New risk management defaults
            max_drawdown: 1_000.0,        // $1000 max drawdown
            max_order_value: 100_000.0,   // $100k max order value
            circuit_breaker_pct: 5.0,     // 5% price change triggers circuit breaker
            circuit_breaker_duration: 60, // 60 seconds circuit breaker
        }
    }
}