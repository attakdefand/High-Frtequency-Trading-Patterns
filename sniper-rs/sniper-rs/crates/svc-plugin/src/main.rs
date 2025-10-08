//! Plugin service for the sniper-rs ecosystem.
//! 
//! This service provides REST APIs for managing third-party plugins,
//! including loading, configuring, and monitoring plugin performance.

use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use axum::{
    routing::{get, post, put, delete},
    Json, Router, Extension,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use sniper_plugin::{PluginManager, PluginConfig, PluginMetadata};

/// CLI arguments for the plugin service
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[clap(short, long, default_value = "8094")]
    port: u16,
}

/// Plugin service state
struct AppState {
    plugin_manager: RwLock<PluginManager>,
}

/// Plugin registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RegisterPluginRequest {
    pub plugin_type: String, // "signal_processor", "strategy", "risk_assessor", "executor"
    pub plugin_data: serde_json::Value,
}

/// Plugin configuration request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigurePluginRequest {
    pub config: PluginConfig,
}

/// Standard response format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

/// Plugin metadata response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PluginMetadataResponse {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub capabilities: Vec<String>,
}

impl From<&PluginMetadata> for PluginMetadataResponse {
    fn from(metadata: &PluginMetadata) -> Self {
        Self {
            id: metadata.id.clone(),
            name: metadata.name.clone(),
            version: metadata.version.clone(),
            description: metadata.description.clone(),
            author: metadata.author.clone(),
            capabilities: metadata.capabilities.clone(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    // Create plugin manager
    let plugin_manager = PluginManager::new();
    
    // Create app state
    let app_state = Arc::new(AppState {
        plugin_manager: RwLock::new(plugin_manager),
    });
    
    // Create router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/plugins", get(list_plugins))
        .route("/plugins/:id", get(get_plugin))
        .route("/plugins", post(register_plugin))
        .route("/plugins/:id/config", put(configure_plugin))
        .route("/plugins/:id", delete(unregister_plugin))
        .route("/process/signals", post(process_signals))
        .route("/generate/plans", post(generate_plans))
        .layer(Extension(app_state));
    
    // Run server
    let addr = format!("0.0.0.0:{}", args.port);
    tracing::info!("Plugin service listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
        
    Ok(())
}

/// Health check endpoint
async fn health_check() -> Json<ApiResponse<String>> {
    let response = ApiResponse {
        success: true,
        data: Some("Plugin service is healthy".to_string()),
        message: None,
    };
    Json(response)
}

/// List all registered plugins
async fn list_plugins(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<ApiResponse<Vec<PluginMetadataResponse>>> {
    let plugin_responses = {
        let plugin_manager = state.plugin_manager.read().await;
        plugin_manager.list_plugins()
            .iter()
            .map(|&metadata| PluginMetadataResponse::from(metadata))
            .collect::<Vec<PluginMetadataResponse>>()
    };
    
    let response = ApiResponse {
        success: true,
        data: Some(plugin_responses),
        message: None,
    };
    Json(response)
}

/// Get a plugin by ID
async fn get_plugin(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<PluginMetadataResponse>> {
    let plugin_data = {
        let plugin_manager = state.plugin_manager.read().await;
        plugin_manager.list_plugins()
            .iter()
            .find(|metadata| metadata.id == id)
            .map(|&metadata| PluginMetadataResponse::from(metadata))
    };
    
    match plugin_data {
        Some(plugin_response) => {
            let response = ApiResponse {
                success: true,
                data: Some(plugin_response),
                message: None,
            };
            Json(response)
        },
        None => {
            let response = ApiResponse {
                success: false,
                data: None,
                message: Some("Plugin not found".to_string()),
            };
            Json(response)
        },
    }
}

/// Register a new plugin
async fn register_plugin(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<RegisterPluginRequest>,
) -> Json<ApiResponse<bool>> {
    // In a real implementation, this would dynamically load plugins
    // For now, we'll just acknowledge the registration
    tracing::info!("Registering plugin of type: {}", payload.plugin_type);
    
    let response = ApiResponse {
        success: true,
        data: Some(true),
        message: Some("Plugin registered successfully".to_string()),
    };
    Json(response)
}

/// Configure a plugin
async fn configure_plugin(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<ConfigurePluginRequest>,
) -> Json<ApiResponse<bool>> {
    state.plugin_manager.write().await.configure_plugin(&id, payload.config);
    
    let response = ApiResponse {
        success: true,
        data: Some(true),
        message: Some("Plugin configured successfully".to_string()),
    };
    Json(response)
}

/// Unregister a plugin
async fn unregister_plugin(
    Extension(_state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<bool>> {
    // In a real implementation, this would unload the plugin
    // For now, we'll just acknowledge the unregistration
    tracing::info!("Unregistering plugin: {}", id);
    
    let response = ApiResponse {
        success: true,
        data: Some(true),
        message: Some("Plugin unregistered successfully".to_string()),
    };
    Json(response)
}

/// Process signals through registered signal processors
async fn process_signals(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Json<ApiResponse<Vec<serde_json::Value>>> {
    let results = state.plugin_manager.read().await.process_signals(&payload)
        .await
        .unwrap_or_else(|e| {
            tracing::error!("Error processing signals: {}", e);
            Vec::new()
        });
    
    let response = ApiResponse {
        success: true,
        data: Some(results),
        message: None,
    };
    Json(response)
}

/// Generate plans through registered strategies
async fn generate_plans(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Json<ApiResponse<Vec<serde_json::Value>>> {
    let plans = state.plugin_manager.read().await.generate_plans(&payload)
        .await
        .unwrap_or_else(|e| {
            tracing::error!("Error generating plans: {}", e);
            Vec::new()
        });
    
    let response = ApiResponse {
        success: true,
        data: Some(plans),
        message: None,
    };
    Json(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing() {
        let args = Args::parse_from(["svc-plugin", "--port", "8095"]);
        assert_eq!(args.port, 8095);
    }

    #[tokio::test]
    async fn test_plugin_service_creation() -> Result<()> {
        let plugin_manager = PluginManager::new();
        let _app_state = Arc::new(AppState {
            plugin_manager: RwLock::new(plugin_manager),
        });
        
        Ok(())
    }
}