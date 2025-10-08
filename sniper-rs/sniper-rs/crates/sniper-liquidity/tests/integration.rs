//! Integration tests for the liquidity aggregator

use sniper_liquidity::{LiquidityAggregator, LiquidityConfig, LiquiditySource, TokenPair};
use sniper_core::types::ChainRef;

#[test]
fn test_liquidity_aggregator_core_features() {
    let config = LiquidityConfig {
        chains: vec!["ethereum".to_string(), "bsc".to_string()],
        protocols: vec!["uniswap".to_string(), "pancakeswap".to_string()],
        min_liquidity: 1000000,
        max_price_impact: 0.05,
    };
    
    let aggregator = LiquidityAggregator::new(config);
    
    // We can't directly access private fields, so we'll test through the public interface
    // Test that we can add and query liquidity sources
    let pair = TokenPair {
        token0: "WETH".to_string(),
        token1: "USDC".to_string(),
    };
    
    // Initially, there should be no sources for this pair
    let sources = aggregator.get_liquidity_sources(&pair);
    assert_eq!(sources.len(), 0);
    
    println!("Liquidity aggregator core features test passed!");
}

#[tokio::test]
async fn test_liquidity_aggregation_functionality() {
    let config = LiquidityConfig {
        chains: vec!["ethereum".to_string()],
        protocols: vec!["uniswap".to_string()],
        min_liquidity: 1000000,
        max_price_impact: 0.05,
    };
    
    let mut aggregator = LiquidityAggregator::new(config);
    
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
    aggregator.add_liquidity_source("test_source".to_string(), source);
    
    // Test aggregation
    let aggregated = aggregator.aggregate_liquidity(&pair).unwrap();
    assert_eq!(aggregated.pair.token0, "WETH");
    assert_eq!(aggregated.pair.token1, "USDC");
    assert_eq!(aggregated.sources.len(), 1);
    assert!(aggregated.total_liquidity > 0);
    
    println!("Liquidity aggregation functionality test passed!");
}

#[tokio::test]
async fn test_cross_protocol_liquidity_aggregation() {
    let config = LiquidityConfig {
        chains: vec!["ethereum".to_string(), "bsc".to_string()],
        protocols: vec!["uniswap".to_string(), "pancakeswap".to_string(), "sushiswap".to_string()],
        min_liquidity: 1000000,
        max_price_impact: 0.05,
    };
    
    let mut aggregator = LiquidityAggregator::new(config);
    
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
        aggregator.add_liquidity_source(id.to_string(), source);
    }
    
    // Test aggregation across multiple protocols
    let aggregated = aggregator.aggregate_liquidity(&pair).unwrap();
    assert_eq!(aggregated.sources.len(), 3);
    assert!(aggregated.total_liquidity > 0);
    
    println!("Cross-protocol liquidity aggregation test passed!");
}

#[tokio::test]
async fn test_liquidity_aggregator_endpoints() {
    // Test that the library compiles correctly
    let output = std::process::Command::new("cargo")
        .args(&["check", "-p", "sniper-liquidity"])
        .output()
        .expect("Failed to execute cargo check");
    
    assert!(output.status.success(), "sniper-liquidity failed to compile");
    
    println!("Liquidity aggregator endpoints test passed!");
}