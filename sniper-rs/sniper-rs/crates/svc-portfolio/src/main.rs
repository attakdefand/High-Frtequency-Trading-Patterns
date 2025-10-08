//! Portfolio management service for the sniper bot.
//! 
//! This service provides a REST API for managing trading portfolios,
//! including position tracking, risk allocation, and performance analytics.

use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use sniper_portfolio::{PortfolioManager, AllocationSettings, Position, PerformanceMetrics};
use sniper_core::types::{ChainRef, TradePlan};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{
    routing::{get, post, put, delete},
    Json, Router, Extension,
};
use uuid::Uuid;

/// CLI arguments for the portfolio service
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[clap(short, long, default_value = "8080")]
    port: u16,

    /// Initial capital for the portfolio
    #[clap(long, default_value = "10000.0")]
    initial_capital: f64,
}

/// Portfolio service state
struct AppState {
    portfolio_manager: RwLock<PortfolioManager>,
}

/// Position creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreatePositionRequest {
    pub symbol: String,
    pub chain_id: u64,
    pub chain_name: String,
    pub amount: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub side: String,
    pub leverage: f64,
}

/// Position update request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UpdatePositionRequest {
    pub current_price: f64,
}

/// Trade plan request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GenerateTradePlanRequest {
    pub symbol: String,
    pub chain_id: u64,
    pub chain_name: String,
    pub amount: f64,
    pub side: String,
}

/// Standard response format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

/// Portfolio metrics response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortfolioMetricsResponse {
    pub total_value: f64,
    pub total_pnl: f64,
    pub total_pnl_percentage: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub positions_count: usize,
}

/// Position response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PositionResponse {
    pub id: String,
    pub symbol: String,
    pub chain_id: u64,
    pub chain_name: String,
    pub amount: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub side: String,
    pub leverage: f64,
    pub pnl: f64,
    pub pnl_percentage: f64,
    pub created_at: u64,
    pub updated_at: u64,
}

impl From<Position> for PositionResponse {
    fn from(position: Position) -> Self {
        PositionResponse {
            id: position.id,
            symbol: position.symbol,
            chain_id: position.chain.id,
            chain_name: position.chain.name,
            amount: position.amount,
            entry_price: position.entry_price,
            current_price: position.current_price,
            side: position.side,
            leverage: position.leverage,
            pnl: position.pnl,
            pnl_percentage: position.pnl_percentage,
            created_at: position.created_at,
            updated_at: position.updated_at,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    // Create default allocation settings
    let allocation_settings = AllocationSettings {
        max_position_size_pct: 5.0,
        max_portfolio_risk_pct: 2.0,
        diversification_targets: HashMap::new(),
        stop_loss_pct: 5.0,
        take_profit_pct: 10.0,
    };
    
    // Create portfolio manager
    let portfolio_manager = PortfolioManager::new(args.initial_capital, allocation_settings);
    
    // Create app state
    let app_state = Arc::new(AppState {
        portfolio_manager: RwLock::new(portfolio_manager),
    });
    
    // Create router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/positions", get(get_positions).post(create_position))
        .route("/positions/:id", get(get_position).put(update_position).delete(close_position))
        .route("/metrics", get(get_portfolio_metrics))
        .route("/plan", post(generate_trade_plan))
        .layer(Extension(app_state));
    
    // Run server
    let addr = format!("0.0.0.0:{}", args.port);
    tracing::info!("Portfolio service listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
        
    Ok(())
}

/// Health check endpoint
async fn health_check() -> Json<ApiResponse<String>> {
    let response = ApiResponse {
        success: true,
        data: Some("Portfolio service is healthy".to_string()),
        message: None,
    };
    Json(response)
}

/// Get all positions
async fn get_positions(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<ApiResponse<Vec<PositionResponse>>> {
    let positions = {
        let manager = state.portfolio_manager.read().await;
        manager.list_positions()
            .iter()
            .map(|&p| PositionResponse::from((*p).clone()))
            .collect::<Vec<PositionResponse>>()
    };
    
    let api_response = ApiResponse {
        success: true,
        data: Some(positions),
        message: None,
    };
    Json(api_response)
}

/// Get a specific position
async fn get_position(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<PositionResponse>> {
    let position_result = {
        let manager = state.portfolio_manager.read().await;
        manager.get_position(&id).cloned()
    };
    
    match position_result {
        Some(position) => {
            let response = ApiResponse {
                success: true,
                data: Some(PositionResponse::from(position)),
                message: None,
            };
            Json(response)
        },
        None => {
            let response = ApiResponse {
                success: false,
                data: None,
                message: Some("Position not found".to_string()),
            };
            Json(response)
        },
    }
}

/// Create a new position
async fn create_position(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<CreatePositionRequest>,
) -> Json<ApiResponse<PositionResponse>> {
    let chain_ref = ChainRef {
        name: payload.chain_name,
        id: payload.chain_id,
    };
    
    // Calculate initial PnL
    let pnl = (payload.current_price - payload.entry_price) * payload.amount;
    let pnl_percentage = if payload.entry_price > 0.0 {
        ((payload.current_price - payload.entry_price) / payload.entry_price) * 100.0
    } else {
        0.0
    };
    
    let position = Position {
        id: Uuid::new_v4().to_string(),
        symbol: payload.symbol,
        chain: chain_ref,
        amount: payload.amount,
        entry_price: payload.entry_price,
        current_price: payload.current_price,
        side: payload.side,
        leverage: payload.leverage,
        pnl,
        pnl_percentage,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        updated_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    
    let result = state.portfolio_manager.write().await.add_position(position.clone());
    match result {
        Ok(_) => {
            let response = ApiResponse {
                success: true,
                data: Some(PositionResponse::from(position)),
                message: Some("Position created successfully".to_string()),
            };
            Json(response)
        },
        Err(e) => {
            let response = ApiResponse {
                success: false,
                data: None,
                message: Some(format!("Failed to create position: {}", e)),
            };
            Json(response)
        },
    }
}

/// Update an existing position
async fn update_position(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<UpdatePositionRequest>,
) -> Json<ApiResponse<PositionResponse>> {
    let position_result = {
        let manager = state.portfolio_manager.read().await;
        manager.get_position(&id).cloned()
    };
    
    match position_result {
        Some(mut existing_position) => {
            existing_position.current_price = payload.current_price;
            
            // Recalculate PnL
            existing_position.pnl = (payload.current_price - existing_position.entry_price) * existing_position.amount;
            existing_position.pnl_percentage = if existing_position.entry_price > 0.0 {
                ((payload.current_price - existing_position.entry_price) / existing_position.entry_price) * 100.0
            } else {
                0.0
            };
            
            existing_position.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            let result = state.portfolio_manager.write().await.update_position(&id, existing_position.clone());
            match result {
                Ok(_) => {
                    let response = ApiResponse {
                        success: true,
                        data: Some(PositionResponse::from(existing_position)),
                        message: Some("Position updated successfully".to_string()),
                    };
                    Json(response)
                },
                Err(e) => {
                    let response = ApiResponse {
                        success: false,
                        data: None,
                        message: Some(format!("Failed to update position: {}", e)),
                    };
                    Json(response)
                },
            }
        },
        None => {
            let response = ApiResponse {
                success: false,
                data: None,
                message: Some("Position not found".to_string()),
            };
            Json(response)
        },
    }
}

/// Close a position
async fn close_position(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<bool>> {
    let result = state.portfolio_manager.write().await.remove_position(&id);
    match result {
        Ok(_) => {
            let response = ApiResponse {
                success: true,
                data: Some(true),
                message: Some("Position closed successfully".to_string()),
            };
            Json(response)
        },
        Err(e) => {
            let response = ApiResponse {
                success: false,
                data: Some(false),
                message: Some(format!("Failed to close position: {}", e)),
            };
            Json(response)
        },
    }
}

/// Get portfolio metrics
async fn get_portfolio_metrics(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<ApiResponse<PortfolioMetricsResponse>> {
    let metrics = {
        let manager = state.portfolio_manager.read().await;
        manager.calculate_performance()
    };
    
    let response = PortfolioMetricsResponse {
        total_value: metrics.total_value,
        total_pnl: metrics.total_pnl,
        total_pnl_percentage: metrics.total_pnl_percentage,
        win_rate: metrics.win_rate,
        profit_factor: metrics.profit_factor,
        sharpe_ratio: metrics.sharpe_ratio,
        max_drawdown: metrics.max_drawdown,
        positions_count: metrics.positions_count,
    };
    
    let api_response = ApiResponse {
        success: true,
        data: Some(response),
        message: None,
    };
    Json(api_response)
}

/// Generate a trade plan
async fn generate_trade_plan(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<GenerateTradePlanRequest>,
) -> Json<ApiResponse<TradePlan>> {
    let chain_ref = ChainRef {
        name: payload.chain_name,
        id: payload.chain_id,
    };
    
    let plan_result = {
        let manager = state.portfolio_manager.read().await;
        manager.generate_trade_plan(
            &payload.symbol,
            chain_ref,
            payload.amount,
            &payload.side,
        )
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
        let args = Args::parse_from(["svc-portfolio", "--port", "8081", "--initial-capital", "50000.0"]);
        assert_eq!(args.port, 8081);
        assert_eq!(args.initial_capital, 50000.0);
    }

    #[tokio::test]
    async fn test_portfolio_service_creation() -> Result<()> {
        let allocation_settings = AllocationSettings {
            max_position_size_pct: 5.0,
            max_portfolio_risk_pct: 2.0,
            diversification_targets: HashMap::new(),
            stop_loss_pct: 5.0,
            take_profit_pct: 10.0,
        };
        
        let portfolio_manager = PortfolioManager::new(10000.0, allocation_settings);
        let _app_state = Arc::new(AppState {
            portfolio_manager: RwLock::new(portfolio_manager),
        });
        
        Ok(())
    }
}