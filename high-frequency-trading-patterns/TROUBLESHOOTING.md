# High-Frequency Trading Patterns - Troubleshooting Guide

## Common Issues and Solutions

### 1. Cargo.toml Parsing Errors

**Problem**: 
```
error: missing table open, expected `[`
 --> Cargo.toml:57:1
  |
57 | ]
  | ^
```

**Cause**: 
The PowerShell script was appending members to the Cargo.toml file multiple times, causing duplicate entries and formatting issues.

**Solution**:
1. Manually clean the Cargo.toml file to remove duplicate entries
2. Ensure each member appears only once in the members list
3. Verify the file ends with the correct structure:
```toml
[workspace]
members = [
  # List of crate paths
]
resolver = "2"
```

### 2. Module Name Issues with Hyphens

**Problem**:
```
error: expected one of `::`, `;`, or `as`, found `-`
 --> crates\market-making\src\main.rs:7:8
  |
7 | use hft-common::prelude::*;
  |        ^ expected one of `::`, `;`, or `as`
```

**Cause**: 
Rust module names cannot contain hyphens. The crate name `hft-common` is valid, but when referencing it as a module, it must be `hft_common`.

**Solution**:
1. Update all source files to use `hft_common` instead of `hft-common`
2. In the PowerShell script, ensure the template uses the correct module name:
```rust
use hft_common::prelude::*;
```

### 3. Missing Dependencies

**Problem**:
```
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `tracing_subscriber`
 --> crates\market-making\src\main.rs:11:5
  |
11 |     tracing_subscriber::fmt::init();
  |     ^^^^^^^^^^^^^^^^^^ use of unresolved module or unlinked crate `tracing_subscriber`
```

**Cause**: 
The `tracing-subscriber` dependency was missing from the pattern crates' Cargo.toml files.

**Solution**:
1. Add `tracing-subscriber = "0.3"` to the dependencies section of the Cargo.toml template in the generation script
2. Regenerate all pattern crates

### 4. Missing hft-common Library Target

**Problem**:
```
Caused by:
  no targets specified in the manifest
  either src/lib.rs, src/main.rs, a [lib] section, or [[bin]] section must be present
```

**Cause**: 
The hft-common crate was missing the lib target specification and source files.

**Solution**:
1. Add a [lib] section to crates/hft-common/Cargo.toml:
```toml
[lib]
path = "src/lib.rs"
```
2. Ensure the src directory exists with the required files:
   - lib.rs
   - models.rs
   - config.rs
   - prelude.rs

### 5. Missing Source Files

**Problem**:
```
error: couldn't read `crates\hft-common\src\lib.rs`: The system cannot find the path specified.
```

**Cause**: 
The hft-common crate's source files were deleted or not created properly.

**Solution**:
1. Recreate the src directory in crates/hft-common
2. Restore the required source files:
   - lib.rs (re-exports models, config, and prelude modules)
   - models.rs (contains Side, Quote, Order, and Fill structs)
   - config.rs (contains Cfg struct and Default implementation)
   - prelude.rs (re-exports commonly used items)

## Testing Individual Patterns

### Running a Specific Pattern

To test a specific pattern, use:
```bash
cargo run --release -p <pattern-name>
```

For example:
```bash
cargo run --release -p market-making
```

### Building All Patterns

To build all patterns in release mode:
```bash
cargo build --release
```

### Building a Specific Pattern

To build a specific pattern:
```bash
cargo build --release -p <pattern-name>
```

## Testing Framework

### Unit Testing

The project includes unit tests for the common components in the hft-common crate. To run these tests:

```bash
cargo test -p hft-common
```

The tests cover:
- Data structure validation (Side, Quote, Order, Fill)
- Configuration management
- Basic functionality of shared components

### Integration Testing

Integration tests can be run for the entire workspace:

```bash
cargo test
```

This will run unit tests for all crates in the workspace.

### Writing Tests for Individual Patterns

To write tests for individual patterns:

1. Create a `tests` directory in the pattern crate
2. Add test files with the `.rs` extension
3. Use the `#[cfg(test)]` attribute for test modules
4. Import necessary modules with `use` statements

Example test structure:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_logic() {
        // Test implementation
    }
}
```

### Common Testing Issues

#### Module Resolution in Tests

**Problem**: 
Tests cannot find modules from the same crate.

**Solution**:
Use proper module paths and ensure the modules are public if needed for testing.

#### Async Testing

**Problem**: 
Tests involving async code may timeout or hang.

**Solution**:
Use `tokio::test` attribute for async tests and implement proper timeouts:

```rust
#[tokio::test]
async fn test_async_function() {
    // Test implementation with timeout
    tokio::time::timeout(Duration::from_secs(5), async {
        // Your async code here
    }).await.unwrap();
}
```

#### Test Dependencies

**Problem**: 
Tests fail due to missing dependencies.

**Solution**:
Ensure all necessary dependencies are included in the Cargo.toml file:

```toml
[dev-dependencies]
tokio = { version = "1.38", features = ["full"] }
tokio-test = "0.4"
```

### Performance Testing

For performance testing, you can use the `criterion` crate:

1. Add criterion to dev-dependencies:
```toml
[dev-dependencies]
criterion = "0.5"
```

2. Create benchmark tests in the `benches` directory:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("strategy execution", |b| {
        b.iter(|| {
            // Your code to benchmark
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
```

3. Run benchmarks:
```bash
cargo bench
```

## Performance Considerations

### Compilation Time

With 25 pattern crates, initial compilation can take several minutes. Subsequent builds will be faster due to incremental compilation.

### Runtime Performance

All patterns are built in release mode for optimal performance. The default configuration simulates a high-frequency environment with 1ms tick intervals.

## Common Warnings

### Unused Mut Warning

```
warning: variable does not need to be mutable
 --> crates\market-making\src\main.rs:15:19
```

**Solution**: Remove the `mut` keyword from variables that aren't mutated.

### Dead Code Warning

```
warning: field `cfg` is never read
 --> crates\market-making\src\strategy.rs:3:20
```

**Solution**: Either use the field or remove it if it's not needed.

## Development Workflow

### Adding a New Pattern

1. Add the new pattern name to the $Patterns array in generate-crates.ps1
2. Run the generation script:
   ```bash
   powershell -ExecutionPolicy Bypass -File generate-crates.ps1
   ```
3. Add the new crate to the workspace members in Cargo.toml
4. Build and test the new pattern

### Modifying Pattern Logic

1. Edit the relevant files in the pattern's src directory:
   - main.rs (wiring logic)
   - simulator.rs (market simulation)
   - strategy.rs (trading strategy)
   - risk.rs (risk management)
2. Test the changes:
   ```bash
   cargo run --release -p <pattern-name>
   ```

### Updating Dependencies

1. Update the dependency versions in the generation script template
2. Regenerate all pattern crates
3. Update the hft-common crate dependencies if needed
4. Rebuild the workspace

## Debugging Tips

### Enable More Verbose Logging

The patterns use the tracing crate for logging. To enable more verbose logging, modify the initialization in main.rs:
```rust
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

### Check for Race Conditions

The patterns use asynchronous execution with Tokio. If you encounter unexpected behavior:
1. Check that all shared state is properly synchronized
2. Ensure proper use of channels for communication between components
3. Verify that the select! macro is used correctly for concurrent operations

## Windows-Specific Issues

### PowerShell Execution Policy

If you encounter issues running the PowerShell script:
```bash
powershell -ExecutionPolicy Bypass -File generate-crates.ps1
```

### Path Issues

Windows uses backslashes in paths. When writing scripts, use forward slashes or properly escape backslashes.

## Version Compatibility

### Rust Version

This project is tested with Rust 1.38+. Ensure you have a compatible version:
```bash
rustc --version
```

### Dependency Versions

The project uses specific versions of dependencies:
- tokio = "1.38"
- tracing = "0.1"
- tracing-subscriber = "0.3"
- serde = "1.0"
- anyhow = "1.0"

Updating these versions may require code changes.

## CI/CD Pipeline

### GitHub Actions Workflows

This project uses GitHub Actions for CI/CD with three main workflows:

1. **CI Workflow** (.github/workflows/ci.yml):
   - Runs on push/PR to main/master branches
   - Builds the workspace
   - Runs all tests
   - Checks code formatting
   - Runs Clippy linter

2. **CD Workflow** (.github/workflows/cd.yml):
   - Runs on tagged releases
   - Builds release binaries
   - Runs tests
   - Creates GitHub releases

3. **Quality Workflow** (.github/workflows/quality.yml):
   - Runs on push/PR to main/master branches
   - Checks code formatting
   - Runs Clippy linter
   - Runs security audits

### Common CI/CD Issues

#### Workflow Failing on Formatting

**Problem**:
```
error: rustfmt failed
```

**Solution**:
Run `cargo fmt --all` locally to format your code, then commit the changes.

#### Clippy Warnings

**Problem**:
```
error: Clippy found issues
```

**Solution**:
Run `cargo clippy --all-targets --all-features -- -D warnings` locally to see and fix the issues.

#### Security Audit Failures

**Problem**:
```
error: Security vulnerabilities found
```

**Solution**:
Run `cargo audit` locally to see the vulnerabilities and update dependencies as needed.

#### Cache Issues

**Problem**:
Intermittent build failures due to corrupted cache.

**Solution**:
GitHub Actions will automatically retry with a clean cache. If problems persist, you can manually clear the cache in the GitHub Actions settings.

### Local Development with CI Checks

To run the same checks locally that CI runs:

1. Install cargo-make:
   ```bash
   cargo install cargo-make
   ```

2. Run all CI checks:
   ```bash
   cargo make ci
   ```

3. Build release version:
   ```bash
   cargo make release
   ```

### Adding New Checks

To add new checks to the CI pipeline:

1. Modify the appropriate workflow file in `.github/workflows/`
2. Test locally with cargo-make
3. Commit and push to verify in CI

## Enhanced Risk Management

### Configuration Issues

**Problem**: 
Risk limits are too restrictive or too loose for the trading strategy.

**Solution**:
Adjust the risk parameters in the [Cfg](file:///c%3A/Users/RMT/Documents/vscodium/Master-Test-Cases-Rust/high-frequency-trading-patterns/crates/hft-common/src/config.rs#L7-L12) struct:
- `max_pos` - Maximum position size
- `max_orders_s` - Maximum orders per second
- `max_drawdown` - Maximum allowed drawdown
- `max_order_value` - Maximum value of a single order
- `circuit_breaker_pct` - Percentage change that triggers circuit breaker
- `circuit_breaker_duration` - Duration in seconds for circuit breaker

### Circuit Breaker False Activations

**Problem**: 
Circuit breaker is activating too frequently due to normal market volatility.

**Solution**:
1. Increase the `circuit_breaker_pct` threshold
2. Increase the `circuit_breaker_duration` to allow for longer cooldown periods
3. Review the price feed for anomalies that might be causing false triggers

### Rate Limiting Issues

**Problem**: 
Orders are being rejected due to rate limiting, but the strategy requires higher order frequency.

**Solution**:
1. Increase the `max_orders_s` parameter
2. Optimize the strategy to send fewer but more valuable orders
3. Implement order batching to reduce the number of individual orders

### Position Limit Issues

**Problem**: 
Valid trading opportunities are being rejected due to position limits.

**Solution**:
1. Increase the `max_pos` parameter
2. Implement position reduction logic to free up capacity
3. Review the position calculation logic for accuracy

### Drawdown Limit Issues

**Problem**: 
Circuit breaker is activating due to drawdown limits being reached too quickly.

**Solution**:
1. Increase the `max_drawdown` parameter
2. Implement profit-taking logic to reduce drawdown
3. Review the PnL calculation for accuracy

### Testing Risk Management

To test the enhanced risk management features:

1. Run unit tests:
   ```bash
   cargo test -p hft-common
   ```

2. Run integration tests:
   ```bash
   cargo test -p hft-common --lib integration_tests
   ```

3. Test with a specific pattern:
   ```bash
   cargo run -p market-making
   ```

### Common Test Failures

#### Circuit Breaker Not Activating

**Problem**: 
Tests expect circuit breaker to activate but it doesn't.

**Solution**:
1. Verify the price change exceeds the `circuit_breaker_pct` threshold
2. Check that sufficient time has passed for the price change to be detected
3. Ensure the risk management system is properly integrated with the quote feed

#### Rate Limit Not Enforced

**Problem**: 
Tests expect rate limit to be enforced but orders are still being sent.

**Solution**:
1. Verify the `max_orders_s` parameter is set correctly
2. Check that the rate limit counter is being updated properly
3. Ensure the time window calculation is correct

## Enhanced Market Making

### Advanced Market Making Algorithms

The enhanced market making implementation includes sophisticated algorithms for:

1. **Dynamic Spread Adjustment**: Spreads are adjusted based on market volatility to optimize profit while managing risk.
2. **Inventory Management**: Active inventory control to maintain target positions and reduce exposure.
3. **Queue Position Tracking**: Monitoring of order queue positions to optimize execution timing.

### Configuration Issues

**Problem**: 
Market making spreads are too tight or too wide for the market conditions.

**Solution**:
Adjust the market making parameters in the EnhancedMarketMaking struct:
- `base_spread` - Base spread for quoting
- `min_spread` - Minimum allowed spread
- `spread_multiplier` - Multiplier for volatility-based spread adjustment
- `target_inventory` - Target inventory position
- `max_inventory` - Maximum inventory limits

### Inventory Management Issues

**Problem**: 
Inventory builds up too quickly, leading to excessive exposure.

**Solution**:
1. Adjust the `target_inventory` to a more neutral position
2. Tighten the `max_inventory` limits
3. Increase the inventory skew factor to adjust quotes more aggressively

### Spread Optimization Issues

**Problem**: 
Spreads are not adjusting properly to changing market conditions.

**Solution**:
1. Review the volatility calculation in `calculate_volatility()`
2. Adjust the `spread_multiplier` to make spreads more or less sensitive to volatility
3. Verify that the quote processing is correctly updating the queue positions

### Testing Market Making Strategies

To test the enhanced market making features:

1. Run unit tests:
   ```bash
   cargo test -p hft-common enhanced_mm
   ```

2. Run the market making pattern:
   ```bash
   cargo run -p market-making
   ```

3. Monitor the logs for order placement and fill information

### Common Test Failures

#### Orders Not Being Generated

**Problem**: 
Market maker is not generating orders despite receiving quotes.

**Solution**:
1. Check that inventory is within limits
2. Verify that the quote processing is working correctly
3. Review the order size calculation logic

#### Inventory Imbalance

**Problem**: 
Market maker builds up excessive inventory in one direction.

**Solution**:
1. Review the inventory skew calculation
2. Adjust the inventory limits
3. Check that fills are being processed correctly to update inventory

## Enhanced Arbitrage Strategies

### Index/ETF Basis Arbitrage

The enhanced arbitrage implementation includes index/ETF basis arbitrage strategies that:

1. Monitor ETF prices against their underlying index values
2. Identify creation and redemption opportunities
3. Execute trades when profit thresholds are met

### Triangular Cross-Exchange Arbitrage

The implementation also includes triangular cross-exchange arbitrage that:

1. Identifies pricing discrepancies across three different assets
2. Calculates round-trip profitability
3. Executes three-legged trades when opportunities exist

### Cross-Asset Latency Strategies

Latency-based arbitrage strategies that:

1. Exploit price discrepancies between venues
2. Use speed advantages to capture alpha
3. Implement position management to control risk

### Performance Optimization

The enhanced arbitrage strategies include:

1. **Latency Tracking**: Monitoring quote-to-quote latencies to optimize execution
2. **Position Management**: Active position control to maintain risk limits
3. **Profit Thresholds**: Configurable minimum profit requirements

### Configuration Issues

**Problem**: 
Arbitrage opportunities are not being identified or executed.

**Solution**:
Adjust the arbitrage parameters in the EnhancedArbitrage struct:
- `min_profit_threshold` - Minimum profit required to execute trades
- `max_position` - Maximum position size limits
- `latency_stats` - Latency tracking parameters

### Testing Arbitrage Strategies

To test the enhanced arbitrage features:

1. Run unit tests:
   ```bash
   cargo test -p hft-common enhanced_arb
   ```

2. Run specific arbitrage patterns:
   ```bash
   cargo run -p statistical-arbitrage
   cargo run -p latency-arbitrage
   ```

3. Monitor the logs for arbitrage opportunity identification and execution

### Common Test Failures

#### Opportunities Not Identified

**Problem**: 
Arbitrage strategies are not identifying profitable opportunities.

**Solution**:
1. Check that the minimum profit threshold is set appropriately
2. Verify that quotes are being processed correctly
3. Review the fair price calculation logic

#### Position Limits Reached

**Problem**: 
Arbitrage strategies are hitting position limits too quickly.

**Solution**:
1. Increase the `max_position` parameter
2. Review the position update logic in `on_fill()`
3. Implement position reduction mechanisms