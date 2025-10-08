//! Integration tests for the AI-driven trading strategies

use sniper_ai::{AiTradingStrategy, AiModelConfig, MarketDataPoint};
use sniper_plugin::Strategy;
use anyhow::Result;

#[test]
fn test_ai_strategy_core_features() {
    let config = AiModelConfig {
        model_type: "lstm".to_string(),
        features: vec!["price".to_string(), "volume".to_string()],
        lookback_period: 100,
        prediction_horizon: 10,
        confidence_threshold: 0.7,
    };
    
    let strategy = AiTradingStrategy::new(config);
    
    // Test metadata
    assert_eq!(strategy.metadata().id, "ai-trading-strategy");
    assert_eq!(strategy.metadata().capabilities.len(), 2);
    assert!(strategy.metadata().capabilities.contains(&"strategy".to_string()));
    assert!(strategy.metadata().capabilities.contains(&"ai".to_string()));
    
    println!("AI strategy core features test passed!");
}

#[tokio::test]
async fn test_ai_strategy_data_handling() -> Result<()> {
    let config = AiModelConfig {
        model_type: "lstm".to_string(),
        features: vec!["price".to_string(), "volume".to_string()],
        lookback_period: 5,
        prediction_horizon: 10,
        confidence_threshold: 0.7,
    };
    
    let mut strategy = AiTradingStrategy::new(config);
    
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
        
        strategy.add_data_point(data_point);
    }
    
    // We can't directly access private fields, so we'll test through the public interface
    // Test that prediction works (which requires data)
    let prediction = strategy.predict()?;
    assert!(prediction.confidence > 0.0);
    
    println!("AI strategy data handling test passed!");
    Ok(())
}

#[tokio::test]
async fn test_ai_strategy_training() -> Result<()> {
    let config = AiModelConfig {
        model_type: "lstm".to_string(),
        features: vec!["price".to_string(), "volume".to_string()],
        lookback_period: 100,
        prediction_horizon: 10,
        confidence_threshold: 0.7,
    };
    
    let mut strategy = AiTradingStrategy::new(config);
    
    // Test training
    strategy.train()?;
    
    // We can't directly access private fields, so we'll test through the public interface
    // Test that prediction works after training
    let prediction = strategy.predict()?;
    assert!(prediction.confidence > 0.0);
    
    println!("AI strategy training test passed!");
    Ok(())
}

#[tokio::test]
async fn test_ai_strategy_prediction_accuracy() -> Result<()> {
    let config = AiModelConfig {
        model_type: "lstm".to_string(),
        features: vec!["price".to_string(), "volume".to_string()],
        lookback_period: 100,
        prediction_horizon: 10,
        confidence_threshold: 0.7,
    };
    
    let mut strategy = AiTradingStrategy::new(config);
    
    // Add trending data (increasing prices)
    for i in 0..20 {
        strategy.add_data_point(MarketDataPoint {
            timestamp: i,
            price: 100.0 + (i as f64 * 2.0), // Strong upward trend
            volume: 1000.0,
            liquidity: 50000.0,
            volatility: 0.1,
            momentum: 0.1,
            rsi: 60.0,
            macd: 0.05,
            signal: None,
        });
    }
    
    // Generate prediction
    let prediction = strategy.predict()?;
    
    // With strong upward trend, confidence should be high
    assert!(prediction.confidence > 0.5);
    
    // Direction should be positive
    assert!(prediction.predicted_direction > 0.0);
    
    println!("AI strategy prediction accuracy test passed!");
    Ok(())
}