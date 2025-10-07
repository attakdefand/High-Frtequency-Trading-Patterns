# Enhanced Risk Management Features

This document describes the enhanced risk management features implemented for the high-frequency trading patterns.

## Features Implemented

### 1. Position Limits
- **Purpose**: Prevents excessive exposure in any single position
- **Implementation**: Tracks current position and rejects orders that would exceed the maximum allowed position
- **Configuration**: `max_pos` parameter in [Cfg](file:///c%3A/Users/RMT/Documents/vscodium/Master-Test-Cases-Rust/high-frequency-trading-patterns/crates/hft-common/src/config.rs#L7-L12) struct

### 2. Rate Limiting
- **Purpose**: Controls the number of orders sent per second to prevent overwhelming the market or violating exchange rules
- **Implementation**: Tracks orders sent per second and rejects orders when the limit is exceeded
- **Configuration**: `max_orders_s` parameter in [Cfg](file:///c%3A/Users/RMT/Documents/vscodium/Master-Test-Cases-Rust/high-frequency-trading-patterns/crates/hft-common/src/config.rs#L7-L12) struct

### 3. Order Value Limits
- **Purpose**: Prevents excessively large orders that could move the market or violate risk limits
- **Implementation**: Calculates order value (quantity × price) and rejects orders that exceed the maximum allowed value
- **Configuration**: `max_order_value` parameter in [Cfg](file:///c%3A/Users/RMT/Documents/vscodium/Master-Test-Cases-Rust/high-frequency-trading-patterns/crates/hft-common/src/config.rs#L7-L12) struct

### 4. Drawdown Limits
- **Purpose**: Prevents excessive losses by tracking profit and loss
- **Implementation**: Maintains a running PnL calculation and activates circuit breaker when drawdown exceeds the limit
- **Configuration**: `max_drawdown` parameter in [Cfg](file:///c%3A/Users/RMT/Documents/vscodium/Master-Test-Cases-Rust/high-frequency-trading-patterns/crates/hft-common/src/config.rs#L7-L12) struct

### 5. Circuit Breakers
- **Purpose**: Temporarily halts trading when market conditions become too volatile or losses exceed limits
- **Implementation**: Activated by either:
  - Excessive price changes (configurable percentage)
  - Excessive drawdown (configurable amount)
- **Configuration**: 
  - `circuit_breaker_pct` - Percentage price change that triggers circuit breaker
  - `circuit_breaker_duration` - Duration in seconds for circuit breaker activation

## Usage

The enhanced risk management features are implemented in the [EnhancedRisk](file:///c%3A/Users/RMT/Documents/vscodium/Master-Test-Cases-Rust/high-frequency-trading-patterns/crates/hft-common/src/enhanced_risk.rs#L25-L37) struct, which can be used in place of the basic risk management in trading patterns.

### Example Usage

```rust
use hft_common::prelude::*;

// Initialize with configuration
let cfg = Cfg::default();
let mut risk = EnhancedRisk::new(&cfg);

// In the main event loop
loop {
    tokio::select! {
        Some(q) = md_rx.recv() => {
            // Update risk management with latest quote for circuit breaker checks
            risk.on_quote(&q);
            
            if let Some(o) = strategy.on_quote(&q) {
                if risk.allow(&o) { 
                    od_tx.send(o).await?; 
                } else {
                    info!("Order rejected by risk management");
                }
            }
        },
        Some(f) = fills.recv() => {
            strategy.on_fill(&f);
            risk.on_fill(&f); // Update risk management with fill information
            info!("FILL {:?} {:.0} @ {:.2}", f.side, f.qty, f.px);
        },
    }
}
```

## Testing

The enhanced risk management features include comprehensive unit tests and integration tests:

1. **Unit Tests**: Test individual risk controls (position limits, rate limiting, etc.)
2. **Integration Tests**: Test the complete risk management system in a simulated trading environment

To run the tests:
```bash
cargo test -p hft-common
```

## Configuration

All risk parameters are configurable through the [Cfg](file:///c%3A/Users/RMT/Documents/vscodium/Master-Test-Cases-Rust/high-frequency-trading-patterns/crates/hft-common/src/config.rs#L7-L12) struct:

```rust
let cfg = Cfg {
    symbol: "XYZ".into(),
    tick_ms: 1,
    tick_sz: 0.01,
    max_pos: 10_000.0,           // Maximum position
    max_orders_s: 50_000,        // Maximum orders per second
    max_drawdown: 1_000.0,       // Maximum allowed drawdown
    max_order_value: 100_000.0,  // Maximum value of a single order
    circuit_breaker_pct: 5.0,    // Percentage change that triggers circuit breaker
    circuit_breaker_duration: 60,// Duration in seconds for circuit breaker
};
```