//! Advanced order types service for the sniper bot.
//! 
//! This service provides a REST API for managing advanced order types including
//! limit orders, stop-loss orders, take-profit orders, trailing stops, and more.

use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use sniper_orders::{OrderManager, AdvancedOrder, OrderType, TimeInForce, OrderStatus};
use sniper_core::types::{ChainRef, TradePlan};
use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{
    routing::{get, post, put, delete},
    Json, Router, Extension,
};
use uuid::Uuid;

/// CLI arguments for the orders service
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[clap(short, long, default_value = "8081")]
    port: u16,
}

/// Order service state
struct AppState {
    order_manager: RwLock<OrderManager>,
}

/// Order creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateOrderRequest {
    pub symbol: String,
    pub chain_id: u64,
    pub chain_name: String,
    pub order_type: String, // Will be parsed into OrderType
    pub side: String,
    pub amount: f64,
    pub price: Option<f64>, // For limit, stop-loss, take-profit orders
    pub stop_price: Option<f64>, // For stop-limit orders
    pub limit_price: Option<f64>, // For stop-limit orders
    pub trail_percent: Option<f64>, // For trailing stop orders
    pub visible_amount: Option<f64>, // For iceberg orders
    pub total_amount: Option<f64>, // For iceberg, TWAP, VWAP orders
    pub duration_minutes: Option<u64>, // For TWAP orders
}

/// Standard response format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

/// Order response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OrderResponse {
    pub id: String,
    pub symbol: String,
    pub chain_id: u64,
    pub chain_name: String,
    pub order_type: String,
    pub side: String,
    pub amount: f64,
    pub price: Option<f64>,
    pub status: String,
    pub created_at: u64,
    pub updated_at: u64,
}

impl From<&AdvancedOrder> for OrderResponse {
    fn from(order: &AdvancedOrder) -> Self {
        OrderResponse {
            id: order.id.clone(),
            symbol: order.symbol.clone(),
            chain_id: order.chain.id,
            chain_name: order.chain.name.clone(),
            order_type: format!("{:?}", order.order_type),
            side: order.side.clone(),
            amount: order.amount,
            price: match &order.order_type {
                OrderType::Limit { price } => Some(*price),
                OrderType::StopLoss { price } => Some(*price),
                _ => None,
            },
            status: format!("{:?}", order.status),
            created_at: order.created_at,
            updated_at: order.updated_at,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    // Create order manager
    let order_manager = OrderManager::new();
    
    // Create app state
    let app_state = Arc::new(AppState {
        order_manager: RwLock::new(order_manager),
    });
    
    // Create router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/orders", get(get_orders).post(create_order))
        .route("/orders/:id", get(get_order).put(update_order).delete(cancel_order))
        .route("/orders/:id/status", get(get_order_status))
        .route("/orders/:id/plan", get(get_trade_plan))
        .layer(Extension(app_state));
    
    // Run server
    let addr = format!("0.0.0.0:{}", args.port);
    tracing::info!("Orders service listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
        
    Ok(())
}

/// Health check endpoint
async fn health_check() -> Json<ApiResponse<String>> {
    let response = ApiResponse {
        success: true,
        data: Some("Orders service is healthy".to_string()),
        message: None,
    };
    Json(response)
}

/// Get all orders
async fn get_orders(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<ApiResponse<Vec<OrderResponse>>> {
    let orders = {
        let manager = state.order_manager.read().await;
        manager.list_orders()
            .iter()
            .map(|&order| OrderResponse::from(order))
            .collect::<Vec<OrderResponse>>()
    };
    
    let response = ApiResponse {
        success: true,
        data: Some(orders),
        message: None,
    };
    Json(response)
}

/// Get a specific order
async fn get_order(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<OrderResponse>> {
    let order_result = {
        let manager = state.order_manager.read().await;
        manager.get_order(&id).cloned()
    };
    
    match order_result {
        Some(order) => {
            let response = ApiResponse {
                success: true,
                data: Some(OrderResponse::from(&order)),
                message: None,
            };
            Json(response)
        },
        None => {
            let response = ApiResponse {
                success: false,
                data: None,
                message: Some("Order not found".to_string()),
            };
            Json(response)
        },
    }
}

/// Create a new order
async fn create_order(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<CreateOrderRequest>,
) -> Json<ApiResponse<OrderResponse>> {
    let chain_ref = ChainRef {
        name: payload.chain_name,
        id: payload.chain_id,
    };
    
    // Parse order type from string
    let order_type = match payload.order_type.as_str() {
        "market" => OrderType::Market,
        "limit" => OrderType::Limit { price: payload.price.unwrap_or(0.0) },
        "stop_loss" => OrderType::StopLoss { price: payload.price.unwrap_or(0.0) },
        "take_profit" => OrderType::TakeProfit { price: payload.price.unwrap_or(0.0) },
        "stop_limit" => OrderType::StopLimit { 
            stop_price: payload.stop_price.unwrap_or(0.0), 
            limit_price: payload.limit_price.unwrap_or(0.0) 
        },
        "trailing_stop" => OrderType::TrailingStop { trail_percent: payload.trail_percent.unwrap_or(1.0) },
        "iceberg" => OrderType::Iceberg { 
            visible_amount: payload.visible_amount.unwrap_or(0.0), 
            total_amount: payload.total_amount.unwrap_or(0.0) 
        },
        "twap" => OrderType::TWAP { 
            total_amount: payload.total_amount.unwrap_or(0.0), 
            duration_minutes: payload.duration_minutes.unwrap_or(60) 
        },
        "vwap" => OrderType::VWAP { total_amount: payload.total_amount.unwrap_or(0.0) },
        _ => OrderType::Market, // Default to market order
    };
    
    let time_in_force = TimeInForce::GoodTillCancelled; // Default to Good Till Cancelled
    
    let order = AdvancedOrder {
        id: Uuid::new_v4().to_string(),
        symbol: payload.symbol,
        chain: chain_ref,
        order_type,
        side: payload.side,
        amount: payload.amount,
        time_in_force,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        updated_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        status: OrderStatus::Pending,
    };
    
    let result = state.order_manager.write().await.create_order(order.clone());
    match result {
        Ok(_) => {
            let response = ApiResponse {
                success: true,
                data: Some(OrderResponse::from(&order)),
                message: Some("Order created successfully".to_string()),
            };
            Json(response)
        },
        Err(e) => {
            let response = ApiResponse {
                success: false,
                data: None,
                message: Some(format!("Failed to create order: {}", e)),
            };
            Json(response)
        },
    }
}

/// Update an existing order
async fn update_order(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<CreateOrderRequest>,
) -> Json<ApiResponse<OrderResponse>> {
    let order_result = {
        let manager = state.order_manager.read().await;
        manager.get_order(&id).cloned()
    };
    
    match order_result {
        Some(mut existing_order) => {
            let chain_ref = ChainRef {
                name: payload.chain_name,
                id: payload.chain_id,
            };
            
            // Parse order type from string
            let order_type = match payload.order_type.as_str() {
                "market" => OrderType::Market,
                "limit" => OrderType::Limit { price: payload.price.unwrap_or(0.0) },
                "stop_loss" => OrderType::StopLoss { price: payload.price.unwrap_or(0.0) },
                "take_profit" => OrderType::TakeProfit { price: payload.price.unwrap_or(0.0) },
                "stop_limit" => OrderType::StopLimit { 
                    stop_price: payload.stop_price.unwrap_or(0.0), 
                    limit_price: payload.limit_price.unwrap_or(0.0) 
                },
                "trailing_stop" => OrderType::TrailingStop { trail_percent: payload.trail_percent.unwrap_or(1.0) },
                "iceberg" => OrderType::Iceberg { 
                    visible_amount: payload.visible_amount.unwrap_or(0.0), 
                    total_amount: payload.total_amount.unwrap_or(0.0) 
                },
                "twap" => OrderType::TWAP { 
                    total_amount: payload.total_amount.unwrap_or(0.0), 
                    duration_minutes: payload.duration_minutes.unwrap_or(60) 
                },
                "vwap" => OrderType::VWAP { total_amount: payload.total_amount.unwrap_or(0.0) },
                _ => OrderType::Market, // Default to market order
            };
            
            existing_order.symbol = payload.symbol;
            existing_order.chain = chain_ref;
            existing_order.order_type = order_type;
            existing_order.side = payload.side;
            existing_order.amount = payload.amount;
            existing_order.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            let result = state.order_manager.write().await.create_order(existing_order.clone());
            match result {
                Ok(_) => {
                    let response = ApiResponse {
                        success: true,
                        data: Some(OrderResponse::from(&existing_order)),
                        message: Some("Order updated successfully".to_string()),
                    };
                    Json(response)
                },
                Err(e) => {
                    let response = ApiResponse {
                        success: false,
                        data: None,
                        message: Some(format!("Failed to update order: {}", e)),
                    };
                    Json(response)
                },
            }
        },
        None => {
            let response = ApiResponse {
                success: false,
                data: None,
                message: Some("Order not found".to_string()),
            };
            Json(response)
        },
    }
}

/// Cancel an order
async fn cancel_order(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<bool>> {
    let result = state.order_manager.write().await.cancel_order(&id);
    match result {
        Ok(_) => {
            let response = ApiResponse {
                success: true,
                data: Some(true),
                message: Some("Order cancelled successfully".to_string()),
            };
            Json(response)
        },
        Err(e) => {
            let response = ApiResponse {
                success: false,
                data: Some(false),
                message: Some(format!("Failed to cancel order: {}", e)),
            };
            Json(response)
        },
    }
}

/// Get order status
async fn get_order_status(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<String>> {
    let status_result = {
        let manager = state.order_manager.read().await;
        manager.get_order(&id).map(|order| order.status.clone())
    };
    
    match status_result {
        Some(status) => {
            let response = ApiResponse {
                success: true,
                data: Some(format!("{:?}", status)),
                message: None,
            };
            Json(response)
        },
        None => {
            let response = ApiResponse {
                success: false,
                data: None,
                message: Some("Order not found".to_string()),
            };
            Json(response)
        },
    }
}

/// Get trade plan for an order
async fn get_trade_plan(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<TradePlan>> {
    // For demonstration, we'll use a default price
    let current_price = 3000.0;
    
    let plan_result = {
        let manager = state.order_manager.read().await;
        manager.to_trade_plan(&id, current_price)
    };
    
    match plan_result {
        Ok(trade_plan) => {
            let response = ApiResponse {
                success: true,
                data: Some(trade_plan),
                message: Some("Trade plan generated successfully".to_string()),
            };
            Json(response)
        },
        Err(e) => {
            let response = ApiResponse {
                success: false,
                data: None,
                message: Some(format!("Failed to generate trade plan: {}", e)),
            };
            Json(response)
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing() {
        let args = Args::parse_from(["svc-orders", "--port", "8082"]);
        assert_eq!(args.port, 8082);
    }

    #[tokio::test]
    async fn test_orders_service_creation() -> Result<()> {
        let order_manager = OrderManager::new();
        let _app_state = Arc::new(AppState {
            order_manager: RwLock::new(order_manager),
        });
        
        Ok(())
    }
}