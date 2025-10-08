//! Liquidity aggregation service for the sniper-rs ecosystem.
//! 
//! This service provides REST APIs for cross-protocol liquidity aggregation
//! and optimal trade routing.

use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use axum::{
    routing::{get, post, delete},
    Json, Router, Extension,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use sniper_liquidity::{LiquidityAggregator, LiquidityConfig, LiquiditySource, TokenPair, AggregatedLiquidity, TradeRoute};

/// CLI arguments for the liquidity service
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[clap(short, long, default_value = "8097")]
    port: u16,
}

/// Liquidity service state
struct AppState {
    liquidity_aggregator: RwLock<LiquidityAggregator>,
}

/// Health check response
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
    timestamp: u64,
}

/// Liquidity source request
#[derive(Deserialize)]
struct AddLiquiditySourceRequest {
    source_id: String,
    source: LiquiditySource,
}

/// Liquidity source response
#[derive(Serialize)]
struct AddLiquiditySourceResponse {
    success: bool,
    message: String,
}

/// Remove liquidity source request
#[derive(Deserialize)]
struct RemoveLiquiditySourceRequest {
    source_id: String,
}

/// Remove liquidity source response
#[derive(Serialize)]
struct RemoveLiquiditySourceResponse {
    success: bool,
    message: String,
}

/// Aggregate liquidity request
#[derive(Deserialize)]
struct AggregateLiquidityRequest {
    pair: TokenPair,
}

/// Aggregate liquidity response
#[derive(Serialize)]
struct AggregateLiquidityResponse {
    success: bool,
    data: Option<AggregatedLiquidity>,
    message: Option<String>,
}

/// Find route request
#[derive(Deserialize)]
struct FindRouteRequest {
    token_in: String,
    token_out: String,
    amount_in: String, // Using string to avoid parsing issues
}

/// Find route response
#[derive(Serialize)]
struct FindRouteResponse {
    success: bool,
    data: Option<TradeRoute>,
    message: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    // Create liquidity aggregator with default config
    let config = LiquidityConfig {
        chains: vec!["ethereum".to_string(), "bsc".to_string(), "polygon".to_string()],
        protocols: vec!["uniswap".to_string(), "pancakeswap".to_string(), "sushiswap".to_string()],
        min_liquidity: 1000000,
        max_price_impact: 0.05,
    };
    
    let liquidity_aggregator = LiquidityAggregator::new(config);
    
    // Create app state
    let app_state = Arc::new(AppState {
        liquidity_aggregator: RwLock::new(liquidity_aggregator),
    });
    
    // Create router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/liquidity/sources", post(add_liquidity_source))
        .route("/liquidity/sources/:id", delete(remove_liquidity_source))
        .route("/liquidity/aggregate", post(aggregate_liquidity))
        .route("/liquidity/route", post(find_best_route))
        .layer(Extension(app_state));
    
    // Run server
    let addr = format!("0.0.0.0:{}", args.port);
    tracing::info!("Liquidity service listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
        
    Ok(())
}

/// Health check endpoint
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        service: "svc-liquidity".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    })
}

/// Add liquidity source
async fn add_liquidity_source(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<AddLiquiditySourceRequest>,
) -> Json<AddLiquiditySourceResponse> {
    {
        let mut aggregator = state.liquidity_aggregator.write().await;
        aggregator.add_liquidity_source(payload.source_id, payload.source);
    }
    
    Json(AddLiquiditySourceResponse {
        success: true,
        message: "Liquidity source added successfully".to_string(),
    })
}

/// Remove liquidity source
async fn remove_liquidity_source(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<RemoveLiquiditySourceResponse> {
    {
        let mut aggregator = state.liquidity_aggregator.write().await;
        aggregator.remove_liquidity_source(&id);
    }
    
    Json(RemoveLiquiditySourceResponse {
        success: true,
        message: "Liquidity source removed successfully".to_string(),
    })
}

/// Aggregate liquidity for a token pair
async fn aggregate_liquidity(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<AggregateLiquidityRequest>,
) -> Json<AggregateLiquidityResponse> {
    match state.liquidity_aggregator.read().await.aggregate_liquidity(&payload.pair) {
        Ok(aggregated) => {
            Json(AggregateLiquidityResponse {
                success: true,
                data: Some(aggregated),
                message: None,
            })
        },
        Err(e) => {
            Json(AggregateLiquidityResponse {
                success: false,
                data: None,
                message: Some(format!("Error aggregating liquidity: {}", e)),
            })
        }
    }
}

/// Find the best route for a trade
async fn find_best_route(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<FindRouteRequest>,
) -> Json<FindRouteResponse> {
    // Parse amount_in
    let amount_in = payload.amount_in.parse::<u128>().unwrap_or(0);
    
    match state.liquidity_aggregator.read().await.find_best_route(
        &payload.token_in,
        &payload.token_out,
        amount_in,
    ) {
        Ok(Some(route)) => {
            Json(FindRouteResponse {
                success: true,
                data: Some(route),
                message: None,
            })
        },
        Ok(None) => {
            Json(FindRouteResponse {
                success: false,
                data: None,
                message: Some("No suitable route found".to_string()),
            })
        },
        Err(e) => {
            Json(FindRouteResponse {
                success: false,
                data: None,
                message: Some(format!("Error finding route: {}", e)),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing() {
        let args = Args::parse_from(["svc-liquidity", "--port", "8098"]);
        assert_eq!(args.port, 8098);
    }

    #[tokio::test]
    async fn test_liquidity_service_creation() -> Result<()> {
        let config = LiquidityConfig {
            chains: vec!["ethereum".to_string()],
            protocols: vec!["uniswap".to_string()],
            min_liquidity: 1000000,
            max_price_impact: 0.05,
        };
        
        let liquidity_aggregator = LiquidityAggregator::new(config);
        let _app_state = Arc::new(AppState {
            liquidity_aggregator: RwLock::new(liquidity_aggregator),
        });
        
        Ok(())
    }
}