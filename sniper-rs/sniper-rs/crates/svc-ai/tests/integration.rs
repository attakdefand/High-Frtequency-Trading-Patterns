//! Integration tests for the AI service

use sniper_ai::{AiModelConfig, MarketDataPoint};
use sniper_plugin::Strategy;

#[tokio::test]
async fn test_ai_service_core_features() {
    let config = AiModelConfig {
        model_type: "lstm".to_string(),
        features: vec!["price".to_string(), "volume".to_string()],
        lookback_period: 100,
        prediction_horizon: 10,
        confidence_threshold: 0.7,
    };
    
    let ai_strategy = sniper_ai::AiTradingStrategy::new(config);
    
    // Test that the AI strategy was created correctly
    assert_eq!(ai_strategy.metadata().id, "ai-trading-strategy");
    assert_eq!(ai_strategy.metadata().capabilities.len(), 2);
    
    println!("AI service core features test passed!");
}

#[tokio::test]
async fn test_ai_service_data_handling() {
    let config = AiModelConfig {
        model_type: "lstm".to_string(),
        features: vec!["price".to_string(), "volume".to_string()],
        lookback_period: 5,
        prediction_horizon: 10,
        confidence_threshold: 0.7,
    };
    
    let mut ai_strategy = sniper_ai::AiTradingStrategy::new(config);
    
    // Add data points
    for i in 0..10 {
        let data_point = MarketDataPoint {
            timestamp: i,
            price: 100.0 + (i as f64),
            volume: 1000.0 + (i as f64 * 10.0),
            liquidity: 50000.0,
            volatility: 0.1,
            momentum: 0.05,
            rsi: 50.0,
            macd: 0.0,
            signal: None,
        };
        
        ai_strategy.add_data_point(data_point);
    }
    
    // Test that prediction works (which requires data)
    let prediction = ai_strategy.predict().unwrap();
    assert!(prediction.confidence > 0.0);
    
    println!("AI service data handling test passed!");
}

#[tokio::test]
async fn test_ai_service_training() {
    let config = AiModelConfig {
        model_type: "lstm".to_string(),
        features: vec!["price".to_string(), "volume".to_string()],
        lookback_period: 100,
        prediction_horizon: 10,
        confidence_threshold: 0.7,
    };
    
    let mut ai_strategy = sniper_ai::AiTradingStrategy::new(config);
    
    // Test training
    ai_strategy.train().unwrap();
    
    // Test that prediction works after training
    let prediction = ai_strategy.predict().unwrap();
    assert!(prediction.confidence > 0.0);
    
    println!("AI service training test passed!");
}

#[tokio::test]
async fn test_ai_service_endpoints() {
    // Test that the service binary exists and compiles
    let output = std::process::Command::new("cargo")
        .args(&["check", "-p", "svc-ai"])
        .output()
        .expect("Failed to execute cargo check");
    
    assert!(output.status.success(), "svc-ai failed to compile");
    
    println!("AI service endpoints test passed!");
}