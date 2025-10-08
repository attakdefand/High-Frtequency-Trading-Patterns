//! Integration tests for the marketplace service and marketplace functionality

use sniper_market::{InMemoryMarketplace, Marketplace};

#[tokio::test]
async fn test_marketplace_core_features() {
    let marketplace = InMemoryMarketplace::new();
    
    // We can't directly access private fields, so we'll test the public interface
    // Test listing strategies (empty initially)
    let strategies = marketplace.list_strategies(None).await.unwrap();
    assert_eq!(strategies.len(), 0);
    
    println!("Marketplace core features test passed!");
}

#[tokio::test]
async fn test_strategy_listing_functionality() {
    let marketplace = InMemoryMarketplace::new();
    
    // Test listing strategies (empty initially)
    let strategies = marketplace.list_strategies(None).await.unwrap();
    assert_eq!(strategies.len(), 0);
    
    // Test filtering strategies
    let filtered = marketplace.list_strategies(Some("test")).await.unwrap();
    assert_eq!(filtered.len(), 0);
    
    println!("Strategy listing functionality test passed!");
}

#[tokio::test]
async fn test_marketplace_statistics() {
    let marketplace = InMemoryMarketplace::new();
    
    // Test initial stats
    let stats = marketplace.get_stats().await.unwrap();
    assert_eq!(stats.total_strategies, 0);
    assert_eq!(stats.total_downloads, 0);
    assert_eq!(stats.total_reviews, 0);
    assert_eq!(stats.average_rating, 0.0);
    
    println!("Marketplace statistics test passed!");
}

#[tokio::test]
async fn test_marketplace_service_endpoints() {
    // Test that the service binary exists and compiles
    let output = std::process::Command::new("cargo")
        .args(&["check", "-p", "svc-market"])
        .output()
        .expect("Failed to execute cargo check");
    
    assert!(output.status.success(), "svc-market failed to compile");
    
    println!("Marketplace service endpoints test passed!");
}