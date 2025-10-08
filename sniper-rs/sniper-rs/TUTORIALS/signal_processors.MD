# Creating Custom Signal Processors

This tutorial will guide you through creating custom signal processors for the Sniper-RS framework.

## What are Signal Processors?

Signal processors are plugins that filter, enhance, and transform incoming trading signals before they reach the strategy engine. They can:
- Filter out low-quality signals
- Enhance signals with additional data
- Transform signals into different formats
- Enrich signals with external data sources

## Prerequisites
- Basic Rust knowledge
- Understanding of the Sniper-RS architecture
- Completed the "Building Your First Strategy" tutorial

## Step 1: Create a New Signal Processor Crate

Create a new crate for your signal processor:

```bash
cargo new --lib my-signal-processor
cd my-signal-processor
```

## Step 2: Add Dependencies

Add the necessary dependencies to your `Cargo.toml`:

```toml
[package]
name = "my-signal-processor"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
async-trait = "0.1"
chrono = "0.4"
sniper-plugin = { path = "../sniper-rs/crates/sniper-plugin" }
```

## Step 3: Implement the Signal Processor

Edit `src/lib.rs` to implement your signal processor:

```rust
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};
use sniper_plugin::{SignalProcessor, PluginMetadata};

/// A signal processor that filters and enhances trading signals
pub struct EnhancedSignalProcessor {
    metadata: PluginMetadata,
}

impl EnhancedSignalProcessor {
    /// Create a new instance of the signal processor
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "enhanced-signal-processor".to_string(),
                name: "Enhanced Signal Processor".to_string(),
                version: "1.0.0".to_string(),
                description: "A signal processor that filters low-quality signals and enhances them with timestamp and confidence data".to_string(),
                author: "Your Name".to_string(),
                capabilities: vec!["signal_processing".to_string()],
                config_schema: Some(json!({
                    "type": "object",
                    "properties": {
                        "min_liquidity": {"type": "number", "default": 1000},
                        "max_price_impact": {"type": "number", "default": 5.0}
                    }
                })),
            },
        }
    }
}

#[async_trait]
impl SignalProcessor for EnhancedSignalProcessor {
    async fn process_signal(&self, signal: &Value) -> Result<Option<Value>> {
        // Extract signal data
        let signal_type = signal.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let liquidity = signal.get("liquidity").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let price_impact = signal.get("price_impact").and_then(|v| v.as_f64()).unwrap_or(100.0);
        
        // Apply filtering logic
        match signal_type {
            "pair_created" => {
                // For pair creation signals, check basic criteria
                if liquidity < 1000.0 || price_impact > 5.0 {
                    // Filter out signals that don't meet minimum criteria
                    return Ok(None);
                }
                
                // Enhance the signal with additional metadata
                let enhanced_signal = json!({
                    "original_signal": signal,
                    "processed_at": Utc::now().to_rfc3339(),
                    "confidence_score": 0.85,
                    "priority": "high",
                    "enhanced": true
                });
                
                Ok(Some(enhanced_signal))
            },
            "trading_enabled" => {
                // For trading enabled signals, apply different logic
                let volume_24h = signal.get("volume_24h").and_then(|v| v.as_f64()).unwrap_or(0.0);
                
                if volume_24h < 10000.0 {
                    // Filter out low volume signals
                    return Ok(None);
                }
                
                let enhanced_signal = json!({
                    "original_signal": signal,
                    "processed_at": Utc::now().to_rfc3339(),
                    "confidence_score": 0.75,
                    "priority": "medium",
                    "enhanced": true
                });
                
                Ok(Some(enhanced_signal))
            },
            _ => {
                // For other signal types, pass through with minimal processing
                let enhanced_signal = json!({
                    "original_signal": signal,
                    "processed_at": Utc::now().to_rfc3339(),
                    "confidence_score": 0.5,
                    "priority": "low",
                    "enhanced": true
                });
                
                Ok(Some(enhanced_signal))
            }
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
    async fn test_enhanced_signal_processor() {
        let processor = EnhancedSignalProcessor::new();
        
        // Test a high-quality pair creation signal
        let good_signal = json!({
            "type": "pair_created",
            "token0": "WETH",
            "token1": "USDC",
            "liquidity": 50000.0,
            "price_impact": 2.5
        });
        
        let processed = processor.process_signal(&good_signal).await.unwrap();
        assert!(processed.is_some());
        
        let enhanced = processed.unwrap();
        assert_eq!(enhanced.get("enhanced").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(enhanced.get("priority").and_then(|v| v.as_str()), Some("high"));
        
        // Test a low-quality pair creation signal
        let bad_signal = json!({
            "type": "pair_created",
            "token0": "WETH",
            "token1": "USDC",
            "liquidity": 100.0,
            "price_impact": 10.0
        });
        
        let processed = processor.process_signal(&bad_signal).await.unwrap();
        assert!(processed.is_none());
    }
}
```

## Step 4: Build and Test

Build and test your signal processor:

```bash
cargo build
cargo test
```

## Step 5: Register with the Plugin Service

Register your signal processor with the plugin service:

```bash
# Start the plugin service if not already running
cargo run -p svc-plugin

# Register your signal processor (this is a simplified example)
curl -X POST http://localhost:8094/plugins \
  -H "Content-Type: application/json" \
  -d '{
    "plugin_type": "signal_processor",
    "plugin_data": {
      "name": "Enhanced Signal Processor",
      "path": "/path/to/your/plugin.so"
    }
  }'
```

## Step 6: Configure and Use

Configure your signal processor through the plugin service:

```bash
curl -X PUT http://localhost:8094/plugins/enhanced-signal-processor/config \
  -H "Content-Type: application/json" \
  -d '{
    "config": {
      "enabled": true,
      "settings": {
        "min_liquidity": 2000,
        "max_price_impact": 3.0
      }
    }
  }'
```

## Advanced Topics

### External Data Integration
You can enhance signals with external data sources:

```rust
use reqwest;

#[async_trait]
impl SignalProcessor for EnhancedSignalProcessor {
    async fn process_signal(&self, signal: &Value) -> Result<Option<Value>> {
        // Fetch external data
        let client = reqwest::Client::new();
        let price_data = client
            .get("https://api.coingecko.com/api/v3/simple/price?ids=ethereum&vs_currencies=usd")
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        
        // Enhance signal with price data
        let enhanced_signal = json!({
            "original_signal": signal,
            "eth_price_usd": price_data.get("ethereum").and_then(|v| v.get("usd")).cloned(),
            "processed_at": Utc::now().to_rfc3339(),
        });
        
        Ok(Some(enhanced_signal))
    }
}
```

### Machine Learning Integration
Integrate ML models for signal scoring:

```rust
// This is a conceptual example - actual implementation would depend on your ML framework
#[async_trait]
impl SignalProcessor for EnhancedSignalProcessor {
    async fn process_signal(&self, signal: &Value) -> Result<Option<Value>> {
        // Preprocess signal for ML model
        let features = preprocess_signal(signal);
        
        // Score signal with ML model
        let confidence = self.ml_model.predict(&features).await?;
        
        // Filter based on ML score
        if confidence < 0.7 {
            return Ok(None);
        }
        
        let enhanced_signal = json!({
            "original_signal": signal,
            "ml_confidence": confidence,
            "processed_at": Utc::now().to_rfc3339(),
        });
        
        Ok(Some(enhanced_signal))
    }
}
```

## Best Practices

1. **Performance**: Signal processors should be fast as they process signals in real-time
2. **Reliability**: Handle errors gracefully to avoid dropping valid signals
3. **Configuration**: Make filtering criteria configurable
4. **Testing**: Thoroughly test with various signal types and edge cases
5. **Documentation**: Document the filtering logic and enhancement process

## Troubleshooting

Common issues and solutions:

1. **Signal filtering too aggressively**: Adjust configuration parameters
2. **Performance bottlenecks**: Optimize external API calls and data processing
3. **Memory leaks**: Ensure proper resource cleanup in async operations
4. **Serialization errors**: Validate JSON structures before processing

## Conclusion

You've now created a custom signal processor that can filter and enhance trading signals. This is a powerful way to improve the quality of signals that reach your trading strategies, leading to better trading performance.