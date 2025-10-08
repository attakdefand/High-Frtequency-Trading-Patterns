//! Integration tests for the liquidity service

use sniper_liquidity::{LiquidityConfig, LiquiditySource, TokenPair};
use sniper_core::types::ChainRef;

#[tokio::test]
async fn test_liquidity_service_core_features() {
    let config = LiquidityConfig {
        chains: vec!["ethereum".to_string(), "bsc".to_string()],
        protocols: vec!["uniswap".to_string(), "pancakeswap".to_string()],
        min_liquidity: 1000000,
        max_price_impact: 0.05,
    };
    
    let liquidity_aggregator = sniper_liquidity::LiquidityAggregator::new(config);
    
    // Test that the liquidity aggregator was created correctly
    let pair = TokenPair {
        token0: "WETH".to_string(),
        token1: "USDC".to_string(),
    };
    
    // Initially, there should be no sources for this pair
    let sources = liquidity_aggregator.get_liquidity_sources(&pair);
    assert_eq!(sources.len(), 0);
    
    println!("Liquidity service core features test passed!");
}

#[tokio::test]
async fn test_liquidity_service_data_handling() {
    let config = LiquidityConfig {
        chains: vec!["ethereum".to_string()],
        protocols: vec!["uniswap".to_string()],
        min_liquidity: 1000000,
        max_price_impact: 0.05,
    };
    
    let mut liquidity_aggregator = sniper_liquidity::LiquidityAggregator::new(config);
    
    let pair = TokenPair {
        token0: "WETH".to_string(),
        token1: "USDC".to_string(),
    };
    
    let source = LiquiditySource {
        protocol: "uniswap".to_string(),
        chain: ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        },
        pair: pair.clone(),
        reserve0: 1000000000000000000000, // 1000 WETH
        reserve1: 2000000000000, // 2,000,000 USDC
        fee: 0.003,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    
    // Add liquidity source
    liquidity_aggregator.add_liquidity_source("test_source".to_string(), source);
    
    // Test aggregation
    let aggregated = liquidity_aggregator.aggregate_liquidity(&pair).unwrap();
    assert_eq!(aggregated.pair.token0, "WETH");
    assert_eq!(aggregated.pair.token1, "USDC");
    assert_eq!(aggregated.sources.len(), 1);
    assert!(aggregated.total_liquidity > 0);
    
    println!("Liquidity service data handling test passed!");
}

#[tokio::test]
async fn test_liquidity_service_cross_protocol() {
    let config = LiquidityConfig {
        chains: vec!["ethereum".to_string(), "bsc".to_string()],
        protocols: vec!["uniswap".to_string(), "pancakeswap".to_string(), "sushiswap".to_string()],
        min_liquidity: 1000000,
        max_price_impact: 0.05,
    };
    
    let mut liquidity_aggregator = sniper_liquidity::LiquidityAggregator::new(config);
    
    let pair = TokenPair {
        token0: "WETH".to_string(),
        token1: "USDC".to_string(),
    };
    
    // Add multiple protocol sources
    let sources = vec![
        ("uniswap_ethereum", LiquiditySource {
            protocol: "uniswap".to_string(),
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            pair: pair.clone(),
            reserve0: 1000000000000000000000, // 1000 WETH
            reserve1: 2000000000000, // 2,000,000 USDC
            fee: 0.003,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }),
        ("pancakeswap_bsc", LiquiditySource {
            protocol: "pancakeswap".to_string(),
            chain: ChainRef {
                name: "bsc".to_string(),
                id: 56,
            },
            pair: pair.clone(),
            reserve0: 500000000000000000000, // 500 WETH
            reserve1: 1000000000000, // 1,000,000 USDC
            fee: 0.0025,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }),
        ("sushiswap_ethereum", LiquiditySource {
            protocol: "sushiswap".to_string(),
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            pair: pair.clone(),
            reserve0: 250000000000000000000, // 250 WETH
            reserve1: 500000000000, // 500,000 USDC
            fee: 0.003,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }),
    ];
    
    // Add all sources
    for (id, source) in sources {
        liquidity_aggregator.add_liquidity_source(id.to_string(), source);
    }
    
    // Test aggregation across multiple protocols
    let aggregated = liquidity_aggregator.aggregate_liquidity(&pair).unwrap();
    assert_eq!(aggregated.sources.len(), 3);
    assert!(aggregated.total_liquidity > 0);
    
    println!("Liquidity service cross-protocol test passed!");
}

#[tokio::test]
async fn test_liquidity_service_endpoints() {
    // Test that the service binary exists and compiles
    let output = std::process::Command::new("cargo")
        .args(&["check", "-p", "svc-liquidity"])
        .output()
        .expect("Failed to execute cargo check");
    
    assert!(output.status.success(), "svc-liquidity failed to compile");
    
    println!("Liquidity service endpoints test passed!");
}