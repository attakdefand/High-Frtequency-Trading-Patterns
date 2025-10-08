use sniper_core::{bus::InMemoryBus, prelude::*};
use tokio::time::{sleep, Duration};
use axum::{
    routing::{get, post, put, delete},
    Json, Router, Extension,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use clap::Parser;
use std::collections::HashMap;

/// CLI arguments for the gateway service
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[clap(short, long, default_value = "3000")]
    port: u16,
}

/// Health check response
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
    timestamp: u64,
}

/// Signal request
#[derive(Deserialize)]
struct SignalRequest {
    source: String,
    kind: String,
    chain_name: String,
    chain_id: u64,
    token0: Option<String>,
    token1: Option<String>,
    extra: serde_json::Value,
}

/// Signal response
#[derive(Serialize)]
struct SignalResponse {
    success: bool,
    message: String,
}

/// External API integration configuration
#[derive(Deserialize, Serialize, Clone)]
struct ExternalApiConfig {
    id: String,
    name: String,
    url: String,
    api_key: Option<String>,
    enabled: bool,
}

/// External API response
#[derive(Serialize)]
struct ExternalApiResponse {
    success: bool,
    message: String,
}

/// List of external APIs response
#[derive(Serialize)]
struct ExternalApisResponse {
    apis: Vec<ExternalApiConfig>,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .json()
        .init();
    dotenvy::dotenv().ok();

    let args = Args::parse();
    
    let bus = InMemoryBus::new(1024);
    
    // Store bus and external APIs in app state
    let app_state = Arc::new(AppState { 
        bus: bus.clone(),
        external_apis: tokio::sync::RwLock::new(HashMap::new()),
    });

    // Demo: publisher task
    let tx_bus = bus.clone();
    tokio::spawn(async move {
        loop {
            let sig = Signal {
                source: "dex".into(),
                kind: "pair_created".into(),
                chain: ChainRef {
                    name: "ethereum".into(),
                    id: 1,
                },
                token0: None,
                token1: None,
                extra: serde_json::json!({"demo":true}),
                seen_at_ms: 0,
            };
            let _ = tx_bus.publish("signals.dex.pair_created", &sig).await;
            sleep(Duration::from_secs(5)).await;
        }
    });

    // Demo: subscriber task
    let mut rx = bus.subscribe("signals.>");
    tokio::spawn(async move {
        loop {
            if let Ok(bytes) = rx.recv().await {
                if let Ok(sig) = serde_json::from_slice::<Signal>(&bytes) {
                    tracing::info!(?sig.kind, "received signal");
                }
            }
        }
    });

    // Create router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/signals", post(create_signal))
        .route("/external-apis", get(list_external_apis))
        .route("/external-apis", post(add_external_api))
        .route("/external-apis/:id", put(update_external_api))
        .route("/external-apis/:id", delete(remove_external_api))
        .layer(Extension(app_state));

    // Run server
    let addr = format!("0.0.0.0:{}", args.port);
    tracing::info!("Gateway service listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

/// Application state
struct AppState {
    bus: InMemoryBus,
    external_apis: tokio::sync::RwLock<HashMap<String, ExternalApiConfig>>,
}

/// Health check endpoint
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        service: "svc-gateway".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    })
}

/// Create a new signal
async fn create_signal(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<SignalRequest>,
) -> Json<SignalResponse> {
    let signal = Signal {
        source: payload.source,
        kind: payload.kind,
        chain: ChainRef {
            name: payload.chain_name,
            id: payload.chain_id,
        },
        token0: payload.token0,
        token1: payload.token1,
        extra: payload.extra,
        seen_at_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64, // Changed to i64 to match Signal struct
    };

    match state.bus.publish("signals.api.created", &signal).await {
        Ok(_) => Json(SignalResponse {
            success: true,
            message: "Signal created successfully".to_string(),
        }),
        Err(_) => Json(SignalResponse {
            success: false,
            message: "Failed to create signal".to_string(),
        }),
    }
}

/// List all external API integrations
async fn list_external_apis(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<ExternalApisResponse> {
    let apis = state.external_apis.read().await;
    let api_list: Vec<ExternalApiConfig> = apis.values().cloned().collect();
    
    Json(ExternalApisResponse {
        apis: api_list,
    })
}

/// Add a new external API integration
async fn add_external_api(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<ExternalApiConfig>,
) -> Json<ExternalApiResponse> {
    let mut apis = state.external_apis.write().await;
    apis.insert(payload.id.clone(), payload);
    
    Json(ExternalApiResponse {
        success: true,
        message: "External API added successfully".to_string(),
    })
}

/// Update an existing external API integration
async fn update_external_api(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<ExternalApiConfig>,
) -> Json<ExternalApiResponse> {
    let mut apis = state.external_apis.write().await;
    
    if apis.contains_key(&id) {
        apis.insert(id, payload);
        Json(ExternalApiResponse {
            success: true,
            message: "External API updated successfully".to_string(),
        })
    } else {
        Json(ExternalApiResponse {
            success: false,
            message: "External API not found".to_string(),
        })
    }
}

/// Remove an external API integration
async fn remove_external_api(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ExternalApiResponse> {
    let mut apis = state.external_apis.write().await;
    
    if apis.remove(&id).is_some() {
        Json(ExternalApiResponse {
            success: true,
            message: "External API removed successfully".to_string(),
        })
    } else {
        Json(ExternalApiResponse {
            success: false,
            message: "External API not found".to_string(),
        })
    }
}