//! AI service for the sniper-rs ecosystem.
//! 
//! This service provides REST APIs for AI-driven trading strategies and market prediction.

use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use axum::{
    routing::{get, post},
    Json, Router, Extension,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use sniper_ai::{AiTradingStrategy, AiModelConfig, MarketDataPoint, MarketPrediction};

/// CLI arguments for the AI service
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[clap(short, long, default_value = "8096")]
    port: u16,
}

/// AI service state
struct AppState {
    ai_strategy: RwLock<AiTradingStrategy>,
}

/// Health check response
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
    timestamp: u64,
}

/// Market data request
#[derive(Deserialize)]
struct MarketDataRequest {
    data_points: Vec<MarketDataPoint>,
}

/// Market data response
#[derive(Serialize)]
struct MarketDataResponse {
    success: bool,
    message: String,
}

/// Prediction response
#[derive(Serialize)]
struct PredictionResponse {
    success: bool,
    data: Option<MarketPrediction>,
    message: Option<String>,
}

/// Training response
#[derive(Serialize)]
struct TrainingResponse {
    success: bool,
    message: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    // Create AI strategy with default config
    let config = AiModelConfig {
        model_type: "lstm".to_string(),
        features: vec!["price".to_string(), "volume".to_string()],
        lookback_period: 100,
        prediction_horizon: 10,
        confidence_threshold: 0.7,
    };
    
    let ai_strategy = AiTradingStrategy::new(config);
    
    // Create app state
    let app_state = Arc::new(AppState {
        ai_strategy: RwLock::new(ai_strategy),
    });
    
    // Create router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/data", post(add_market_data))
        .route("/predict", get(get_prediction))
        .route("/train", post(train_model))
        .layer(Extension(app_state));
    
    // Run server
    let addr = format!("0.0.0.0:{}", args.port);
    tracing::info!("AI service listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
        
    Ok(())
}

/// Health check endpoint
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        service: "svc-ai".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    })
}

/// Add market data points
async fn add_market_data(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<MarketDataRequest>,
) -> Json<MarketDataResponse> {
    {
        let mut ai_strategy = state.ai_strategy.write().await;
        for data_point in payload.data_points {
            ai_strategy.add_data_point(data_point);
        }
    }
    
    Json(MarketDataResponse {
        success: true,
        message: "Market data added successfully".to_string(),
    })
}

/// Get market prediction
async fn get_prediction(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<PredictionResponse> {
    match state.ai_strategy.read().await.predict() {
        Ok(prediction) => {
            Json(PredictionResponse {
                success: true,
                data: Some(prediction),
                message: None,
            })
        },
        Err(e) => {
            Json(PredictionResponse {
                success: false,
                data: None,
                message: Some(format!("Error generating prediction: {}", e)),
            })
        }
    }
}

/// Train the AI model
async fn train_model(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<TrainingResponse> {
    match state.ai_strategy.write().await.train() {
        Ok(_) => {
            Json(TrainingResponse {
                success: true,
                message: "Model trained successfully".to_string(),
            })
        },
        Err(e) => {
            Json(TrainingResponse {
                success: false,
                message: format!("Error training model: {}", e),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing() {
        let args = Args::parse_from(["svc-ai", "--port", "8097"]);
        assert_eq!(args.port, 8097);
    }

    #[tokio::test]
    async fn test_ai_service_creation() -> Result<()> {
        let config = AiModelConfig {
            model_type: "lstm".to_string(),
            features: vec!["price".to_string(), "volume".to_string()],
            lookback_period: 100,
            prediction_horizon: 10,
            confidence_threshold: 0.7,
        };
        
        let ai_strategy = AiTradingStrategy::new(config);
        let _app_state = Arc::new(AppState {
            ai_strategy: RwLock::new(ai_strategy),
        });
        
        Ok(())
    }
}