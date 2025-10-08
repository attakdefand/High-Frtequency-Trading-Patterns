//! Integration tests for the web-based services in the sniper-rs project.
//! 
//! This test suite verifies that the HTTP APIs for svc-gateway, svc-portfolio, 
//! and svc-orders are working correctly.

use reqwest;
use serde_json::{json, Value};
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

/// Start the gateway service and return the process handle
fn start_gateway_service() -> Child {
    let mut child = Command::new("cargo")
        .args(&["run", "-p", "svc-gateway", "--", "--port", "3000"])
        .spawn()
        .expect("Failed to start gateway service");
    
    // Give the service time to start
    thread::sleep(Duration::from_secs(2));
    
    child
}

/// Start the portfolio service and return the process handle
fn start_portfolio_service() -> Child {
    let mut child = Command::new("cargo")
        .args(&["run", "-p", "svc-portfolio", "--", "--port", "8080"])
        .spawn()
        .expect("Failed to start portfolio service");
    
    // Give the service time to start
    thread::sleep(Duration::from_secs(2));
    
    child
}

/// Start the orders service and return the process handle
fn start_orders_service() -> Child {
    let mut child = Command::new("cargo")
        .args(&["run", "-p", "svc-orders", "--", "--port", "8081"])
        .spawn()
        .expect("Failed to start orders service");
    
    // Give the service time to start
    thread::sleep(Duration::from_secs(2));
    
    child
}

/// Test the gateway service health endpoint
#[tokio::test]
async fn test_gateway_health() {
    let _gateway_process = start_gateway_service();
    
    // Give the service time to fully start
    tokio::time::sleep(Duration::from_secs(3)).await;
    
    let client = reqwest::Client::new();
    let res = client
        .get("http://localhost:3000/health")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(res.status(), 200);
    
    let json: Value = res.json().await.expect("Failed to parse JSON");
    assert_eq!(json["status"], "ok");
    assert_eq!(json["service"], "svc-gateway");
    
    // Note: We're not killing the process here as it would affect other tests
    // In a real test environment, we would properly manage process lifecycle
}

/// Test the portfolio service health endpoint
#[tokio::test]
async fn test_portfolio_health() {
    let _portfolio_process = start_portfolio_service();
    
    // Give the service time to fully start
    tokio::time::sleep(Duration::from_secs(3)).await;
    
    let client = reqwest::Client::new();
    let res = client
        .get("http://localhost:8080/health")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(res.status(), 200);
    
    let json: Value = res.json().await.expect("Failed to parse JSON");
    assert_eq!(json["success"], true);
    assert_eq!(json["data"], "Portfolio service is healthy");
    
    // Note: We're not killing the process here as it would affect other tests
}

/// Test the orders service health endpoint
#[tokio::test]
async fn test_orders_health() {
    let _orders_process = start_orders_service();
    
    // Give the service time to fully start
    tokio::time::sleep(Duration::from_secs(3)).await;
    
    let client = reqwest::Client::new();
    let res = client
        .get("http://localhost:8081/health")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(res.status(), 200);
    
    let json: Value = res.json().await.expect("Failed to parse JSON");
    assert_eq!(json["success"], true);
    assert_eq!(json["data"], "Orders service is healthy");
    
    // Note: We're not killing the process here as it would affect other tests
}

/// Test creating a signal through the gateway service
#[tokio::test]
async fn test_gateway_create_signal() {
    let _gateway_process = start_gateway_service();
    
    // Give the service time to fully start
    tokio::time::sleep(Duration::from_secs(3)).await;
    
    let client = reqwest::Client::new();
    let signal_data = json!({
        "source": "test_api",
        "kind": "test_signal",
        "chain_name": "ethereum",
        "chain_id": 1,
        "token0": "0xToken0",
        "token1": "0xToken1",
        "extra": {"test": true}
    });
    
    let res = client
        .post("http://localhost:3000/signals")
        .json(&signal_data)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(res.status(), 200);
    
    let json: Value = res.json().await.expect("Failed to parse JSON");
    assert_eq!(json["success"], true);
    assert_eq!(json["message"], "Signal created successfully");
}

/// Test portfolio positions endpoints
#[tokio::test]
async fn test_portfolio_positions() {
    let _portfolio_process = start_portfolio_service();
    
    // Give the service time to fully start
    tokio::time::sleep(Duration::from_secs(3)).await;
    
    let client = reqwest::Client::new();
    
    // Test getting positions (should be empty initially)
    let res = client
        .get("http://localhost:8080/positions")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(res.status(), 200);
    
    let json: Value = res.json().await.expect("Failed to parse JSON");
    assert_eq!(json["success"], true);
    
    // Test creating a position
    let position_data = json!({
        "symbol": "ETH/USDT",
        "chain_id": 1,
        "chain_name": "ethereum",
        "amount": 1.5,
        "entry_price": 3000.0,
        "current_price": 3100.0,
        "side": "long",
        "leverage": 2.0
    });
    
    let res = client
        .post("http://localhost:8080/positions")
        .json(&position_data)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(res.status(), 200);
    
    let json: Value = res.json().await.expect("Failed to parse JSON");
    assert_eq!(json["success"], true);
    assert_eq!(json["message"], "Position created successfully");
}

/// Test orders endpoints
#[tokio::test]
async fn test_orders_endpoints() {
    let _orders_process = start_orders_service();
    
    // Give the service time to fully start
    tokio::time::sleep(Duration::from_secs(3)).await;
    
    let client = reqwest::Client::new();
    
    // Test getting orders (should be empty initially)
    let res = client
        .get("http://localhost:8081/orders")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(res.status(), 200);
    
    let json: Value = res.json().await.expect("Failed to parse JSON");
    assert_eq!(json["success"], true);
    
    // Test creating an order
    let order_data = json!({
        "symbol": "ETH/USDT",
        "chain_id": 1,
        "chain_name": "ethereum",
        "order_type": "limit",
        "side": "buy",
        "amount": 1.0,
        "price": 3000.0,
        "stop_price": null,
        "limit_price": null,
        "trail_percent": null,
        "visible_amount": null,
        "total_amount": null,
        "duration_minutes": null
    });
    
    let res = client
        .post("http://localhost:8081/orders")
        .json(&order_data)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(res.status(), 200);
    
    let json: Value = res.json().await.expect("Failed to parse JSON");
    assert_eq!(json["success"], true);
    assert_eq!(json["message"], "Order created successfully");
}