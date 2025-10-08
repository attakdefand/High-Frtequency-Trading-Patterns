//! Plugin architecture for the sniper-rs ecosystem.
//! 
//! This module provides a flexible plugin system that allows third-party integrations
//! with the sniper bot framework. Plugins can extend functionality in various areas
//! including signal processing, strategy execution, risk management, and more.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub capabilities: Vec<String>,
    pub config_schema: Option<Value>,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub settings: HashMap<String, Value>,
}

/// Signal processing plugin trait
#[async_trait]
pub trait SignalProcessor: Send + Sync {
    /// Process an incoming signal
    async fn process_signal(&self, signal: &Value) -> Result<Option<Value>>;
    
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;
}

/// Strategy plugin trait
#[async_trait]
pub trait Strategy: Send + Sync {
    /// Generate a trade plan from a signal
    async fn generate_plan(&self, signal: &Value) -> Result<Option<Value>>;
    
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;
}

/// Risk assessment plugin trait
#[async_trait]
pub trait RiskAssessor: Send + Sync {
    /// Assess risk for a trade plan
    async fn assess_risk(&self, plan: &Value) -> Result<Value>;
    
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;
}

/// Execution plugin trait
#[async_trait]
pub trait Executor: Send + Sync {
    /// Execute a trade plan
    async fn execute(&self, plan: &Value) -> Result<Value>;
    
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;
}

/// Plugin manager for loading and managing plugins
pub struct PluginManager {
    signal_processors: Vec<Box<dyn SignalProcessor>>,
    strategies: Vec<Box<dyn Strategy>>,
    risk_assessors: Vec<Box<dyn RiskAssessor>>,
    executors: Vec<Box<dyn Executor>>,
    config: HashMap<String, PluginConfig>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            signal_processors: Vec::new(),
            strategies: Vec::new(),
            risk_assessors: Vec::new(),
            executors: Vec::new(),
            config: HashMap::new(),
        }
    }
    
    /// Register a signal processor plugin
    pub fn register_signal_processor(&mut self, processor: Box<dyn SignalProcessor>) {
        self.signal_processors.push(processor);
    }
    
    /// Register a strategy plugin
    pub fn register_strategy(&mut self, strategy: Box<dyn Strategy>) {
        self.strategies.push(strategy);
    }
    
    /// Register a risk assessor plugin
    pub fn register_risk_assessor(&mut self, assessor: Box<dyn RiskAssessor>) {
        self.risk_assessors.push(assessor);
    }
    
    /// Register an executor plugin
    pub fn register_executor(&mut self, executor: Box<dyn Executor>) {
        self.executors.push(executor);
    }
    
    /// Configure a plugin
    pub fn configure_plugin(&mut self, plugin_id: &str, config: PluginConfig) {
        self.config.insert(plugin_id.to_string(), config);
    }
    
    /// Get plugin configuration
    pub fn get_plugin_config(&self, plugin_id: &str) -> Option<&PluginConfig> {
        self.config.get(plugin_id)
    }
    
    /// Process signals through all registered signal processors
    pub async fn process_signals(&self, signal: &Value) -> Result<Vec<Value>> {
        let mut results = Vec::new();
        
        for processor in &self.signal_processors {
            if let Some(result) = processor.process_signal(signal).await? {
                results.push(result);
            }
        }
        
        Ok(results)
    }
    
    /// Generate plans through all registered strategies
    pub async fn generate_plans(&self, signal: &Value) -> Result<Vec<Value>> {
        let mut plans = Vec::new();
        
        for strategy in &self.strategies {
            if let Some(plan) = strategy.generate_plan(signal).await? {
                plans.push(plan);
            }
        }
        
        Ok(plans)
    }
    
    /// Assess risk for a plan through all registered risk assessors
    pub async fn assess_risks(&self, plan: &Value) -> Result<Vec<Value>> {
        let mut assessments = Vec::new();
        
        for assessor in &self.risk_assessors {
            let assessment = assessor.assess_risk(plan).await?;
            assessments.push(assessment);
        }
        
        Ok(assessments)
    }
    
    /// Execute a plan through all registered executors
    pub async fn execute_plans(&self, plan: &Value) -> Result<Vec<Value>> {
        let mut results = Vec::new();
        
        for executor in &self.executors {
            let result = executor.execute(plan).await?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Get all registered plugin metadata
    pub fn list_plugins(&self) -> Vec<&PluginMetadata> {
        let mut metadata = Vec::new();
        
        for processor in &self.signal_processors {
            metadata.push(processor.metadata());
        }
        
        for strategy in &self.strategies {
            metadata.push(strategy.metadata());
        }
        
        for assessor in &self.risk_assessors {
            metadata.push(assessor.metadata());
        }
        
        for executor in &self.executors {
            metadata.push(executor.metadata());
        }
        
        metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // Mock signal processor plugin
    struct MockSignalProcessor {
        metadata: PluginMetadata,
    }
    
    #[async_trait]
    impl SignalProcessor for MockSignalProcessor {
        async fn process_signal(&self, signal: &Value) -> Result<Option<Value>> {
            // Simple mock implementation
            Ok(Some(json!({
                "processed_signal": signal,
                "plugin": self.metadata.name
            })))
        }
        
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }
    }
    
    // Mock strategy plugin
    struct MockStrategy {
        metadata: PluginMetadata,
    }
    
    #[async_trait]
    impl Strategy for MockStrategy {
        async fn generate_plan(&self, signal: &Value) -> Result<Option<Value>> {
            // Simple mock implementation
            Ok(Some(json!({
                "trade_plan": signal,
                "strategy": self.metadata.name
            })))
        }
        
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }
    }
    
    #[test]
    fn test_plugin_metadata() {
        let metadata = PluginMetadata {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            capabilities: vec!["signal_processing".to_string()],
            config_schema: Some(json!({
                "enabled": true,
                "settings": {}
            })),
        };
        
        assert_eq!(metadata.id, "test-plugin");
        assert_eq!(metadata.name, "Test Plugin");
        assert_eq!(metadata.capabilities.len(), 1);
    }
    
    #[tokio::test]
    async fn test_plugin_manager() {
        let mut plugin_manager = PluginManager::new();
        
        // Create mock plugins
        let signal_processor = MockSignalProcessor {
            metadata: PluginMetadata {
                id: "mock-signal-processor".to_string(),
                name: "Mock Signal Processor".to_string(),
                version: "1.0.0".to_string(),
                description: "A mock signal processor".to_string(),
                author: "Test".to_string(),
                capabilities: vec!["signal_processing".to_string()],
                config_schema: None,
            },
        };
        
        let strategy = MockStrategy {
            metadata: PluginMetadata {
                id: "mock-strategy".to_string(),
                name: "Mock Strategy".to_string(),
                version: "1.0.0".to_string(),
                description: "A mock strategy".to_string(),
                author: "Test".to_string(),
                capabilities: vec!["strategy".to_string()],
                config_schema: None,
            },
        };
        
        // Register plugins
        plugin_manager.register_signal_processor(Box::new(signal_processor));
        plugin_manager.register_strategy(Box::new(strategy));
        
        // Test plugin listing
        let plugins = plugin_manager.list_plugins();
        assert_eq!(plugins.len(), 2);
        
        // Test signal processing
        let signal = json!({
            "type": "pair_created",
            "token0": "0x123",
            "token1": "0x456"
        });
        
        let processed_signals = plugin_manager.process_signals(&signal).await.unwrap();
        assert_eq!(processed_signals.len(), 1);
        
        // Test plan generation
        let plans = plugin_manager.generate_plans(&signal).await.unwrap();
        assert_eq!(plans.len(), 1);
        
        println!("Plugin manager tests passed!");
    }
    
    #[tokio::test]
    async fn test_plugin_configuration() {
        let mut plugin_manager = PluginManager::new();
        
        let config = PluginConfig {
            enabled: true,
            settings: HashMap::from([
                ("max_concurrent_trades".to_string(), json!(10)),
                ("risk_tolerance".to_string(), json!("medium")),
            ]),
        };
        
        plugin_manager.configure_plugin("test-plugin", config);
        
        let retrieved_config = plugin_manager.get_plugin_config("test-plugin");
        assert!(retrieved_config.is_some());
        
        let config = retrieved_config.unwrap();
        assert!(config.enabled);
        assert_eq!(config.settings.len(), 2);
        
        println!("Plugin configuration tests passed!");
    }
}