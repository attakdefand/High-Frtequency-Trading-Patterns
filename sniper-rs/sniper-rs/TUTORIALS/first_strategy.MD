# Building Your First Strategy

This tutorial will guide you through creating your first trading strategy for the Sniper-RS framework.

## Prerequisites
- Basic Rust knowledge
- Understanding of DeFi concepts
- Sniper-RS installed and running

## Step 1: Create a New Strategy Crate

First, create a new crate for your strategy:

```bash
cargo new --lib my-first-strategy
cd my-first-strategy
```

## Step 2: Add Dependencies

Add the necessary dependencies to your `Cargo.toml`:

```toml
[package]
name = "my-first-strategy"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
async-trait = "0.1"
sniper-plugin = { path = "../sniper-rs/crates/sniper-plugin" }
```

## Step 3: Implement the Strategy

Edit `src/lib.rs` to implement your strategy:

```rust
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use sniper_plugin::{Strategy, PluginMetadata};

/// A simple arbitrage strategy
pub struct SimpleArbitrageStrategy {
    metadata: PluginMetadata,
}

impl SimpleArbitrageStrategy {
    /// Create a new instance of the strategy
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "simple-arbitrage".to_string(),
                name: "Simple Arbitrage Strategy".to_string(),
                version: "1.0.0".to_string(),
                description: "A basic arbitrage strategy that looks for price differences between pools".to_string(),
                author: "Your Name".to_string(),
                capabilities: vec!["strategy".to_string()],
                config_schema: None,
            },
        }
    }
}

#[async_trait]
impl Strategy for SimpleArbitrageStrategy {
    async fn generate_plan(&self, signal: &Value) -> Result<Option<Value>> {
        // Extract signal data
        let token0 = signal.get("token0").and_then(|v| v.as_str()).unwrap_or("");
        let token1 = signal.get("token1").and_then(|v| v.as_str()).unwrap_or("");
        
        // Simple arbitrage logic
        if !token0.is_empty() && !token1.is_empty() {
            // Generate a trade plan
            let plan = serde_json::json!({
                "action": "arbitrage",
                "token_in": token0,
                "token_out": token1,
                "amount_in": "1000000000000000000", // 1 token in wei
                "max_slippage": "0.005", // 0.5%
                "execution_deadline": 30 // 30 seconds
            });
            
            Ok(Some(plan))
        } else {
            // No valid trade opportunity
            Ok(None)
        }
    }
    
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_simple_arbitrage_strategy() {
        let strategy = SimpleArbitrageStrategy::new();
        
        let signal = json!({
            "token0": "WETH",
            "token1": "USDC"
        });
        
        let plan = strategy.generate_plan(&signal).await.unwrap();
        assert!(plan.is_some());
        
        let plan = plan.unwrap();
        assert_eq!(plan.get("action").and_then(|v| v.as_str()), Some("arbitrage"));
        assert_eq!(plan.get("token_in").and_then(|v| v.as_str()), Some("WETH"));
        assert_eq!(plan.get("token_out").and_then(|v| v.as_str()), Some("USDC"));
    }
}
```

## Step 4: Build and Test

Build and test your strategy:

```bash
cargo build
cargo test
```

## Step 5: Register with the Plugin Service

To use your strategy with the Sniper-RS framework, you need to register it with the plugin service:

```bash
# Start the plugin service
cargo run -p svc-plugin

# Register your strategy (this is a simplified example)
curl -X POST http://localhost:8094/plugins \
  -H "Content-Type: application/json" \
  -d '{
    "plugin_type": "strategy",
    "plugin_data": {
      "name": "Simple Arbitrage Strategy",
      "path": "/path/to/your/plugin.so"
    }
  }'
```

## Step 6: Configure and Use

Configure your strategy through the plugin service:

```bash
curl -X PUT http://localhost:8094/plugins/simple-arbitrage/config \
  -H "Content-Type: application/json" \
  -d '{
    "config": {
      "enabled": true,
      "settings": {
        "max_slippage": "0.01",
        "min_profit_threshold": "0.005"
      }
    }
  }'
```

## Next Steps

- Explore more complex strategy patterns
- Add risk management to your strategies
- Implement custom signal processors
- Share your strategy on the community marketplace

## Troubleshooting

If you encounter issues:

1. Check that all dependencies are correctly installed
2. Verify that the Sniper-RS services are running
3. Review the logs for error messages
4. Consult the community resources for help

## Conclusion

You've now created your first trading strategy for Sniper-RS! This is just the beginning - the framework is designed to be extensible, so you can build increasingly sophisticated strategies and plugins.