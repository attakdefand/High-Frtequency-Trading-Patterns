# High-Frequency Trading Patterns - User Instructions

This document provides comprehensive instructions for running and using the 25 high-frequency trading patterns implemented in this Rust workspace.

## Table of Contents

1. [Project Overview](#project-overview)
2. [Prerequisites](#prerequisites)
3. [Project Structure](#project-structure)
4. [Building the Project](#building-the-project)
5. [Running Individual Patterns](#running-individual-patterns)
6. [Running All Patterns](#running-all-patterns)
7. [Configuration](#configuration)
8. [Understanding the Patterns](#understanding-the-patterns)
9. [Testing](#testing)
10. [Troubleshooting](#troubleshooting)
11. [Performance Considerations](#performance-considerations)
12. [Extending the System](#extending-the-system)

## Project Overview

This repository contains 25 high-frequency trading patterns implemented in Rust. Each pattern is a separate crate in a workspace structure, allowing for modular development and testing. The patterns include market making, statistical arbitrage, latency arbitrage, and many other sophisticated trading strategies.

## Prerequisites

Before running the patterns, ensure you have the following installed:

1. **Rust Toolchain** (version 1.38 or higher)
   - Install using [rustup](https://www.rust-lang.org/tools/install):
     ```bash
     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
     ```

2. **Cargo** (comes with Rust toolchain)

3. **Git** (for version control)

## Project Structure

The project follows a workspace structure with the following key components:

```
high-frequency-trading-patterns/
├── Cargo.toml                    # Workspace configuration
├── crates/
│   ├── hft-common/              # Shared components
│   │   ├── src/
│   │   │   ├── models.rs        # Data structures (Side, Quote, Order, Fill)
│   │   │   ├── config.rs        # Configuration management
│   │   │   ├── enhanced_risk.rs # Enhanced risk management
│   │   │   ├── enhanced_mm.rs   # Enhanced market making strategies
│   │   │   ├── enhanced_arb.rs  # Enhanced arbitrage strategies
│   │   │   ├── prelude.rs       # Common imports
│   │   │   └── lib.rs           # Library exports
│   ├── market-making/           # Market making pattern
│   ├── statistical-arbitrage/   # Statistical arbitrage pattern
│   ├── latency-arbitrage/       # Latency arbitrage pattern
│   └── ... (22 more pattern crates)
├── PRODUCT_FEATURES.md          # Product features documentation
├── PRODUCT_ROADMAP.md           # Development roadmap
├── PRODUCT_DAILY.md             # Daily development log
├── troubleshooting.md           # Troubleshooting guide
└── INSTRUCTIONS.md              # This file
```

Each pattern crate follows a consistent structure:
- `src/main.rs` - Application entry point and wiring logic
- `src/simulator.rs` - Market/environment simulation
- `src/strategy.rs` - Core algorithmic logic
- `src/risk.rs` - Risk management controls (basic implementation)

## Building the Project

To build all patterns in the workspace:

```bash
# Build in debug mode
cargo build

# Build in release mode (recommended for performance)
cargo build --release
```

To build a specific pattern:

```bash
# Build a specific pattern in debug mode
cargo build -p market-making

# Build a specific pattern in release mode
cargo build -p market-making --release
```

## Running Individual Patterns

To run a specific pattern:

```bash
# Run in debug mode
cargo run -p market-making

# Run in release mode (recommended for performance)
cargo run -p market-making --release
```

Available patterns (use with `-p` flag):
1. `market-making`
2. `statistical-arbitrage`
3. `index-etf-basis-arb`
4. `triangular-cross-exchange-arb`
5. `latency-arbitrage`
6. `event-news-algo`
7. `dark-midpoint-arb`
8. `liquidity-detection-scout`
9. `rebate-fee-arb`
10. `queue-dynamics`
11. `auction-imbalance-alpha`
12. `options-vol-arb`
13. `inventory-aware-exec`
14. `orderbook-ml-microstructure`
15. `sor-venue-alpha`
16. `flow-anticipation`
17. `liquidity-mirroring`
18. `momentum-ignition`
19. `spoofing-layering`
20. `quote-stuffing`
21. `wash-painting`
22. `last-look-fx`
23. `stop-trigger-hunting`
24. `opening-gap-fade`
25. `cross-asset-latency-lead`

Example:
```bash
# Run the statistical arbitrage pattern
cargo run -p statistical-arbitrage --release
```

## Running All Patterns

To run all patterns simultaneously, you can use:

```bash
# This will run all binaries in the workspace
cargo run --release
```

However, this will run all patterns in sequence. To run multiple patterns concurrently, you'll need to run them in separate terminals or use a process manager.

## Configuration

Each pattern uses a common configuration structure defined in `crates/hft-common/src/config.rs`. The default configuration is:

```rust
Cfg {
    symbol: "XYZ".to_string(),      // Trading symbol
    tick_ms: 1,                     // Tick interval in milliseconds
    tick_sz: 0.01,                  // Tick size
    max_pos: 10_000.0,              // Maximum position size
    max_orders_s: 50_000,           // Maximum orders per second
    max_drawdown: 1_000.0,          // Maximum drawdown
    max_order_value: 100_000.0,     // Maximum order value
    circuit_breaker_pct: 5.0,       // Circuit breaker percentage
    circuit_breaker_duration: 60,   // Circuit breaker duration in seconds
}
```

To modify the configuration for a pattern, edit the `src/main.rs` file in that pattern's directory and adjust the Cfg values.

## Understanding the Patterns

### Pattern Categories

The 25 patterns are organized into several categories:

1. **Market Making Patterns**
   - `market-making`: Core market making strategy with advanced features

2. **Arbitrage Patterns**
   - `statistical-arbitrage`: Mean reversion strategies
   - `index-etf-basis-arb`: ETF creation/redemption arbitrage
   - `triangular-cross-exchange-arb`: 3-leg cross exchange arbitrage
   - `latency-arbitrage`: Speed-based arbitrage
   - `rebate-fee-arb`: Exchange rebate optimization

3. **Event-Driven Patterns**
   - `event-news-algo`: News-driven strategies
   - `auction-imbalance-alpha`: Opening/closing auction strategies

4. **Order Book Patterns**
   - `dark-midpoint-arb`: Dark pool midpoint strategies
   - `liquidity-detection-scout`: Liquidity hunting algorithms
   - `queue-dynamics`: Order queue positioning
   - `orderbook-ml-microstructure`: Machine learning on orderbook data

5. **Execution Patterns**
   - `inventory-aware-exec`: Position-aware execution
   - `sor-venue-alpha`: Smart order routing optimization

6. **Flow Prediction Patterns**
   - `flow-anticipation`: Order flow prediction
   - `liquidity-mirroring`: Liquidity provision strategies

7. **Momentum Patterns**
   - `momentum-ignition`: Momentum-based strategies
   - `opening-gap-fade`: Gap trading strategies

8. **Cross-Asset Patterns**
   - `cross-asset-latency-lead`: Cross-market timing strategies

9. **Detection/Negative Patterns**
   - `spoofing-layering`: Detection of spoofing patterns
   - `quote-stuffing`: Detection of quote stuffing
   - `wash-painting`: Detection of wash trading
   - `stop-trigger-hunting`: Detection of stop hunting

10. **Specialized Patterns**
    - `options-vol-arb`: Volatility-based strategies
    - `last-look-fx`: FX venue patterns
    - `liquidity-mirroring`: Liquidity provision strategies

### Enhanced Features

The system includes enhanced features in the `hft-common` crate:

1. **Enhanced Risk Management** (`enhanced_risk.rs`):
   - Position limits
   - Rate limiting
   - Circuit breakers
   - Drawdown limits
   - Order value limits

2. **Enhanced Market Making** (`enhanced_mm.rs`):
   - Advanced market making algorithms
   - Inventory management
   - Spread optimization
   - Queue position tracking

3. **Enhanced Arbitrage** (`enhanced_arb.rs`):
   - Index/ETF basis arbitrage
   - Triangular cross-exchange arbitrage
   - Statistical arbitrage
   - Latency-based strategies

## Testing

To run all unit tests:

```bash
# Run tests for the entire workspace
cargo test

# Run tests for a specific crate
cargo test -p hft-common

# Run tests with more verbose output
cargo test -- --nocapture
```

To run specific test suites:

```bash
# Run only enhanced market making tests
cargo test -p hft-common enhanced_mm

# Run only enhanced arbitrage tests
cargo test -p hft-common enhanced_arb

# Run only enhanced risk management tests
cargo test -p hft-common enhanced_risk
```

## Troubleshooting

Common issues and their solutions are documented in the [troubleshooting.md](troubleshooting.md) file. Some common issues include:

1. **Cargo.toml parsing errors**: Clean the Cargo.toml file to remove duplicate entries
2. **Module name issues**: Use `hft_common` instead of `hft-common` in imports
3. **Missing dependencies**: Ensure `tracing-subscriber` is included in dependencies
4. **Missing hft-common library target**: Add `[lib]` section to hft-common/Cargo.toml

For detailed troubleshooting information, refer to the [troubleshooting.md](troubleshooting.md) file.

## Performance Considerations

1. **Compilation Time**: With 25 pattern crates, initial compilation can take several minutes. Subsequent builds will be faster due to incremental compilation.

2. **Runtime Performance**: All patterns are built in release mode for optimal performance. The default configuration simulates a high-frequency environment with 1ms tick intervals.

3. **Memory Usage**: Each pattern runs in its own process, so running multiple patterns simultaneously will increase memory usage.

4. **CPU Usage**: The patterns are designed to be CPU-intensive, especially in release mode with high-frequency ticking.

## Extending the System

### Adding a New Pattern

1. Add the new pattern name to the `members` array in `Cargo.toml`
2. Create a new directory in `crates/` with your pattern name
3. Create the standard structure:
   - `Cargo.toml` (pattern-specific)
   - `src/main.rs` (wiring logic)
   - `src/simulator.rs` (market simulation)
   - `src/strategy.rs` (trading strategy)
   - `src/risk.rs` (risk management)
4. Use the existing patterns as templates for implementation

### Modifying Existing Patterns

1. Edit the relevant files in the pattern's `src/` directory:
   - `main.rs` (wiring logic)
   - `simulator.rs` (market simulation)
   - `strategy.rs` (trading strategy)
   - `risk.rs` (risk management)
2. Test the changes:
   ```bash
   cargo run --release -p <pattern-name>
   ```

### Using Enhanced Components

The `hft-common` crate provides enhanced components that can be used in any pattern:

1. **Enhanced Risk Management**:
   ```rust
   use hft_common::prelude::*;
   
   let cfg = Cfg::default();
   let mut risk = EnhancedRisk::new(&cfg);
   
   // Check if order is allowed
   if risk.allow(&order) {
       // Send order
   }
   ```

2. **Enhanced Market Making**:
   ```rust
   use hft_common::prelude::*;
   
   let cfg = Cfg::default();
   let mut mm = EnhancedMarketMaking::new(cfg);
   
   // Process quote and generate orders
   let orders = mm.on_quote(&quote);
   ```

3. **Enhanced Arbitrage**:
   ```rust
   use hft_common::prelude::*;
   
   let cfg = Cfg::default();
   let mut arb = EnhancedArbitrage::new(cfg, ArbitrageType::Statistical);
   
   // Process quote for statistical arbitrage
   if let Some(order) = arb.on_statistical_arbitrage_quote(&quote, fair_price) {
       // Send order
   }
   ```

## Conclusion

This system provides a comprehensive set of 25 high-frequency trading patterns implemented in Rust. Each pattern is designed to be modular, testable, and extensible. The enhanced components in `hft-common` provide sophisticated risk management, market making, and arbitrage capabilities.

For any issues not covered in this document, please refer to the [troubleshooting.md](troubleshooting.md) file or create an issue in the repository.