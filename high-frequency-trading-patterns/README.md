# High-Frequency Trading Patterns

A comprehensive collection of 25 high-frequency trading patterns implemented in Rust. This project provides a modular, extensible framework for algorithmic trading research and development.

## Project Overview

This repository contains 25 high-frequency trading patterns implemented in Rust. Each pattern is a separate crate in a workspace structure, allowing for modular development and testing. The patterns include market making, statistical arbitrage, latency arbitrage, and many other sophisticated trading strategies.

## Features

- **25 Trading Patterns**: Market making, arbitrage, event-driven, order book, execution, flow prediction, momentum, cross-asset, detection, and specialized strategies
- **Cross-Platform Compatibility**: Runs on Windows, macOS, and Linux
- **Enhanced Risk Management**: Position limits, rate limiting, circuit breakers, drawdown limits
- **Modular Architecture**: Each pattern is a separate crate for easy development and testing
- **Performance Optimized**: Built with Rust for speed and memory safety
- **Comprehensive Documentation**: Detailed instructions and troubleshooting guides

## Prerequisites

1. **Rust Toolchain** (version 1.38 or higher)
   - Install using [rustup](https://www.rust-lang.org/tools/install):
     ```bash
     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
     ```

2. **Git** (for version control)

## Quick Start

### Windows

Open PowerShell as Administrator and navigate to the project root directory:

```powershell
# Start all 25 patterns
.\master-run.ps1 start

# Stop all patterns
.\master-run.ps1 stop

# Check status of all patterns
.\master-run.ps1 status
```

### macOS/Linux or Windows with WSL/Git Bash

Open Terminal and navigate to the project root directory:

```bash
# Make the script executable
chmod +x master-run.sh

# Start all 25 patterns
./master-run.sh start

# Stop all patterns
./master-run.sh stop

# Check status of all patterns
./master-run.sh status
```

## Pattern Categories

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

## Enhanced Features

The system includes enhanced features in the `hft-common` crate:

1. **Enhanced Risk Management**:
   - Position limits
   - Rate limiting
   - Circuit breakers
   - Drawdown limits
   - Order value limits

2. **Enhanced Market Making**:
   - Advanced market making algorithms
   - Inventory management
   - Spread optimization
   - Queue position tracking

3. **Enhanced Arbitrage**:
   - Index/ETF basis arbitrage
   - Triangular cross-exchange arbitrage
   - Statistical arbitrage
   - Latency-based strategies

## Project Structure

```
high-frequency-trading-patterns/
├── Cargo.toml                    # Workspace configuration
├── .gitignore                    # Git ignore file
├── README.md                     # This file
├── INSTRUCTIONS.md               # Detailed usage instructions
├── MASTER-RUN-README.md          # Master run script documentation
├── master-run.sh                 # Bash script for macOS/Linux
├── master-run.ps1                # PowerShell script for Windows
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
└── troubleshooting.md           # Troubleshooting guide
```

## Running Individual Patterns

To run a specific pattern:

```bash
# Run in debug mode
cargo run -p market-making

# Run in release mode (recommended for performance)
cargo run -p market-making --release
```

## Building the Project

To build all patterns in the workspace:

```bash
# Build in debug mode
cargo build

# Build in release mode (recommended for performance)
cargo build --release
```

## Testing

To run all unit tests:

```bash
# Run tests for the entire workspace
cargo test

# Run tests for a specific crate
cargo test -p hft-common
```

## Documentation

For detailed instructions on using this system, please refer to:

1. [INSTRUCTIONS.md](INSTRUCTIONS.md) - Comprehensive usage guide
2. [MASTER-RUN-README.md](MASTER-RUN-README.md) - Master run script documentation
3. [troubleshooting.md](troubleshooting.md) - Troubleshooting guide
4. [PRODUCT_FEATURES.md](PRODUCT_FEATURES.md) - Product features documentation
5. [PRODUCT_ROADMAP.md](PRODUCT_ROADMAP.md) - Development roadmap
6. [PRODUCT_DAILY.md](PRODUCT_DAILY.md) - Daily development log

## Cross-Platform Compatibility

This project is designed to run on all major operating systems:

- **Windows**: Use `master-run.ps1` PowerShell script
- **macOS**: Use `master-run.sh` Bash script
- **Linux**: Use `master-run.sh` Bash script
- **Windows with WSL/Git Bash**: Use `master-run.sh` Bash script

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with Rust for performance and safety
- Inspired by real-world high-frequency trading strategies
- Designed for educational and research purposes