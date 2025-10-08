//! Example signal processor plugin for the sniper-rs ecosystem.
//! 
//! This example demonstrates how to create a third-party plugin that can be
//! loaded into the sniper-rs plugin system.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use sniper_plugin::{SignalProcessor, PluginMetadata};

/// A simple signal processor that filters and enhances trading signals
pub struct SimpleSignalProcessor {
    metadata: PluginMetadata,
}

impl SimpleSignalProcessor {
    /// Create a new simple signal processor
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "simple-signal-processor".to_string(),
                name: "Simple Signal Processor".to_string(),
                version: "1.0.0".to_string(),
                description: "A basic signal processor that filters and enhances trading signals".to_string(),
                author: "Example Developer".to_string(),
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
impl SignalProcessor for SimpleSignalProcessor {
    async fn process_signal(&self, signal: &Value) -> Result<Option<Value>> {
        // Extract signal data
        let signal_type = signal.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let chain = signal.get("chain").and_then(|v| v.as_str()).unwrap_or("");
        
        // Apply filtering logic
        match signal_type {
            "pair_created" => {
                // For pair creation signals, check basic criteria
                let liquidity = signal.get("liquidity").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let price_impact = signal.get("price_impact").and_then(|v| v.as_f64()).unwrap_or(100.0);
                
                // Filter out signals that don't meet minimum criteria
                if liquidity < 1000.0 || price_impact > 5.0 {
                    return Ok(None); // Filter out this signal
                }
                
                // Enhance the signal with additional metadata
                let enhanced_signal = json!({
                    "original_signal": signal,
                    "processed_at": chrono::Utc::now().to_rfc3339(),
                    "confidence_score": 0.85,
                    "recommendation": "high_priority",
                    "enhanced": true
                });
                
                Ok(Some(enhanced_signal))
            },
            "trading_enabled" => {
                // For trading enabled signals, apply different logic
                let volume_24h = signal.get("volume_24h").and_then(|v| v.as_f64()).unwrap_or(0.0);
                
                if volume_24h < 10000.0 {
                    return Ok(None); // Filter out low volume signals
                }
                
                let enhanced_signal = json!({
                    "original_signal": signal,
                    "processed_at": chrono::Utc::now().to_rfc3339(),
                    "confidence_score": 0.75,
                    "recommendation": "medium_priority",
                    "enhanced": true
                });
                
                Ok(Some(enhanced_signal))
            },
            _ => {
                // For other signal types, pass through with minimal processing
                let enhanced_signal = json!({
                    "original_signal": signal,
                    "processed_at": chrono::Utc::now().to_rfc3339(),
                    "confidence_score": 0.5,
                    "recommendation": "low_priority",
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

#[tokio::main]
async fn main() -> Result<()> {
    // Create the plugin
    let processor = SimpleSignalProcessor::new();
    
    // Example signal
    let signal = json!({
        "type": "pair_created",
        "chain": "ethereum",
        "token0": "WETH",
        "token1": "USDC",
        "liquidity": 50000.0,
        "price_impact": 2.5
    });
    
    // Process the signal
    match processor.process_signal(&signal).await? {
        Some(processed_signal) => {
            println!("Processed signal: {}", serde_json::to_string_pretty(&processed_signal)?);
        },
        None => {
            println!("Signal was filtered out");
        }
    }
    
    Ok(())
}