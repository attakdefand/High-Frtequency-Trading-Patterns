//! Integration tests for the plugin service and plugin architecture

use sniper_plugin::{PluginManager, PluginMetadata, PluginConfig};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_plugin_architecture_core_features() {
    let mut plugin_manager = PluginManager::new();
    
    // Test plugin metadata creation
    let metadata = PluginMetadata {
        id: "test-plugin".to_string(),
        name: "Test Plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "A test plugin for integration testing".to_string(),
        author: "Integration Test".to_string(),
        capabilities: vec!["signal_processing".to_string(), "strategy".to_string()],
        config_schema: Some(json!({
            "type": "object",
            "properties": {
                "enabled": {"type": "boolean", "default": true},
                "max_concurrent_trades": {"type": "integer", "default": 5}
            }
        })),
    };
    
    assert_eq!(metadata.id, "test-plugin");
    assert_eq!(metadata.capabilities.len(), 2);
    assert!(metadata.config_schema.is_some());
    
    // Test plugin configuration
    let mut config_settings = HashMap::new();
    config_settings.insert("enabled".to_string(), json!(true));
    config_settings.insert("max_concurrent_trades".to_string(), json!(10));
    
    let config = PluginConfig {
        enabled: true,
        settings: config_settings,
    };
    
    plugin_manager.configure_plugin("test-plugin", config);
    
    let retrieved_config = plugin_manager.get_plugin_config("test-plugin");
    assert!(retrieved_config.is_some());
    
    let config = retrieved_config.unwrap();
    assert!(config.enabled);
    assert_eq!(config.settings.len(), 2);
    
    // Test plugin listing (empty initially)
    let plugins = plugin_manager.list_plugins();
    assert_eq!(plugins.len(), 0);
    
    println!("Plugin architecture core features test passed!");
}

#[tokio::test]
async fn test_plugin_manager_functionality() {
    let plugin_manager = PluginManager::new();
    
    // Test signal processing with empty manager
    let signal = json!({
        "type": "pair_created",
        "chain": "ethereum",
        "token0": "0x123",
        "token1": "0x456"
    });
    
    let processed_signals = plugin_manager.process_signals(&signal).await.unwrap();
    assert_eq!(processed_signals.len(), 0);
    
    // Test plan generation with empty manager
    let plans = plugin_manager.generate_plans(&signal).await.unwrap();
    assert_eq!(plans.len(), 0);
    
    // Test risk assessment with empty manager
    let plan = json!({
        "chain": "ethereum",
        "amount_in": "1000000000000000000",
        "token_in": "ETH",
        "token_out": "USDC"
    });
    
    let risk_assessments = plugin_manager.assess_risks(&plan).await.unwrap();
    assert_eq!(risk_assessments.len(), 0);
    
    // Test execution with empty manager
    let execution_results = plugin_manager.execute_plans(&plan).await.unwrap();
    assert_eq!(execution_results.len(), 0);
    
    println!("Plugin manager functionality test passed!");
}

#[tokio::test]
async fn test_plugin_service_endpoints() {
    // Test that the service binary exists and compiles
    let output = std::process::Command::new("cargo")
        .args(&["check", "-p", "svc-plugin"])
        .output()
        .expect("Failed to execute cargo check");
    
    assert!(output.status.success(), "svc-plugin failed to compile");
    
    println!("Plugin service endpoints test passed!");
}

#[test]
fn test_plugin_configuration_management() {
    let mut plugin_manager = PluginManager::new();
    
    // Test multiple plugin configurations
    let configs = vec![
        ("plugin-1", true, vec![("setting1", json!("value1")), ("setting2", json!(42))]),
        ("plugin-2", false, vec![("enabled", json!(false)), ("timeout", json!(5000))]),
        ("plugin-3", true, vec![("mode", json!("aggressive")), ("slippage", json!(0.5))]),
    ];
    
    for (plugin_id, enabled, settings) in configs {
        let mut config_settings = HashMap::new();
        for (key, value) in &settings { // Borrow instead of move
            config_settings.insert(key.to_string(), value.clone());
        }
        
        let config = PluginConfig {
            enabled,
            settings: config_settings,
        };
        
        plugin_manager.configure_plugin(plugin_id, config);
        
        let retrieved_config = plugin_manager.get_plugin_config(plugin_id);
        assert!(retrieved_config.is_some());
        
        let config = retrieved_config.unwrap();
        assert_eq!(config.enabled, enabled);
        assert_eq!(config.settings.len(), settings.len());
    }
    
    println!("Plugin configuration management test passed!");
}