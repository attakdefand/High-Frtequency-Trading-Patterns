//! Marketplace service for the sniper-rs ecosystem.
//! 
//! This service provides REST APIs for community strategy sharing and discovery.

use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use axum::{
    routing::{get, post},
    Json, Router, Extension,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use sniper_market::{InMemoryMarketplace, Marketplace, StrategyListing, StrategyReview, MarketStats};

/// CLI arguments for the marketplace service
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[clap(short, long, default_value = "8095")]
    port: u16,
}

/// Marketplace service state
struct AppState {
    marketplace: RwLock<InMemoryMarketplace>,
}

/// Standard response format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    // Create marketplace
    let marketplace = InMemoryMarketplace::new();
    
    // Create app state
    let app_state = Arc::new(AppState {
        marketplace: RwLock::new(marketplace),
    });
    
    // Create router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/strategies", get(list_strategies))
        .route("/strategies/:id", get(get_strategy))
        .route("/strategies", post(upload_strategy))
        .route("/strategies/:id/download", get(download_strategy))
        .route("/strategies/:id/reviews", get(get_reviews))
        .route("/reviews", post(add_review))
        .route("/stats", get(get_stats))
        .layer(Extension(app_state));
    
    // Run server
    let addr = format!("0.0.0.0:{}", args.port);
    tracing::info!("Marketplace service listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
        
    Ok(())
}

/// Health check endpoint
async fn health_check() -> Json<ApiResponse<String>> {
    let response = ApiResponse {
        success: true,
        data: Some("Marketplace service is healthy".to_string()),
        message: None,
    };
    Json(response)
}

/// List strategies
async fn list_strategies(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<ApiResponse<Vec<StrategyListing>>> {
    let strategies = state.marketplace.read().await.list_strategies(None).await
        .unwrap_or_else(|e| {
            tracing::error!("Error listing strategies: {}", e);
            Vec::new()
        });
    
    let response = ApiResponse {
        success: true,
        data: Some(strategies),
        message: None,
    };
    Json(response)
}

/// Get a strategy by ID
async fn get_strategy(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<StrategyListing>> {
    match state.marketplace.read().await.get_strategy(&id).await {
        Ok(Some(strategy)) => {
            let response = ApiResponse {
                success: true,
                data: Some(strategy),
                message: None,
            };
            Json(response)
        },
        Ok(None) => {
            let response = ApiResponse {
                success: false,
                data: None,
                message: Some("Strategy not found".to_string()),
            };
            Json(response)
        },
        Err(e) => {
            let response = ApiResponse {
                success: false,
                data: None,
                message: Some(format!("Error retrieving strategy: {}", e)),
            };
            Json(response)
        }
    }
}

/// Upload a new strategy
async fn upload_strategy(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<StrategyListing>,
) -> Json<ApiResponse<bool>> {
    match state.marketplace.read().await.upload_strategy(payload).await {
        Ok(_) => {
            let response = ApiResponse {
                success: true,
                data: Some(true),
                message: Some("Strategy uploaded successfully".to_string()),
            };
            Json(response)
        },
        Err(e) => {
            let response = ApiResponse {
                success: false,
                data: Some(false),
                message: Some(format!("Error uploading strategy: {}", e)),
            };
            Json(response)
        }
    }
}

/// Download a strategy
async fn download_strategy(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<Vec<u8>>> {
    match state.marketplace.read().await.download_strategy(&id).await {
        Ok(content) => {
            let response = ApiResponse {
                success: true,
                data: Some(content),
                message: None,
            };
            Json(response)
        },
        Err(e) => {
            let response = ApiResponse {
                success: false,
                data: None,
                message: Some(format!("Error downloading strategy: {}", e)),
            };
            Json(response)
        }
    }
}

/// Get reviews for a strategy
async fn get_reviews(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<Vec<StrategyReview>>> {
    match state.marketplace.read().await.get_reviews(&id).await {
        Ok(reviews) => {
            let response = ApiResponse {
                success: true,
                data: Some(reviews),
                message: None,
            };
            Json(response)
        },
        Err(e) => {
            let response = ApiResponse {
                success: false,
                data: None,
                message: Some(format!("Error retrieving reviews: {}", e)),
            };
            Json(response)
        }
    }
}

/// Add a review
async fn add_review(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<StrategyReview>,
) -> Json<ApiResponse<bool>> {
    match state.marketplace.read().await.add_review(payload).await {
        Ok(_) => {
            let response = ApiResponse {
                success: true,
                data: Some(true),
                message: Some("Review added successfully".to_string()),
            };
            Json(response)
        },
        Err(e) => {
            let response = ApiResponse {
                success: false,
                data: Some(false),
                message: Some(format!("Error adding review: {}", e)),
            };
            Json(response)
        }
    }
}

/// Get marketplace statistics
async fn get_stats(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<ApiResponse<MarketStats>> {
    match state.marketplace.read().await.get_stats().await {
        Ok(stats) => {
            let response = ApiResponse {
                success: true,
                data: Some(stats),
                message: None,
            };
            Json(response)
        },
        Err(e) => {
            let response = ApiResponse {
                success: false,
                data: None,
                message: Some(format!("Error retrieving stats: {}", e)),
            };
            Json(response)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing() {
        let args = Args::parse_from(["svc-market", "--port", "8096"]);
        assert_eq!(args.port, 8096);
    }

    #[tokio::test]
    async fn test_marketplace_service_creation() -> Result<()> {
        let marketplace = InMemoryMarketplace::new();
        let _app_state = Arc::new(AppState {
            marketplace: RwLock::new(marketplace),
        });
        
        Ok(())
    }
}