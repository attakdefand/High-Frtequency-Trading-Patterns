//! Monitoring service for the sniper-rs enterprise features.
//! 
//! This service provides REST APIs for advanced monitoring dashboards
//! and automated incident response.

use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use axum::{
    routing::{get, post},
    Json, Router, Extension,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use sniper_monitoring::{
    MonitoringSystem,
    DashboardPanel,
    Incident,
    IncidentSeverity,
    AlertRule,
};

/// CLI arguments for the monitoring service
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[clap(short, long, default_value = "8086")]
    port: u16,
}

/// Monitoring service state
struct AppState {
    monitoring_system: Arc<RwLock<MonitoringSystem>>,
}

/// Dashboard creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateDashboardRequest {
    pub name: String,
    pub description: String,
    pub panels: Vec<DashboardPanel>,
    pub tenant_id: String,
}

/// Incident creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateIncidentRequest {
    pub title: String,
    pub description: String,
    pub severity: String, // Will be parsed into IncidentSeverity
    pub tenant_id: String,
}

/// Alert rule creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateAlertRuleRequest {
    pub name: String,
    pub description: String,
    pub query: String,
    pub threshold: f64,
    pub severity: String, // Will be parsed into IncidentSeverity
    pub tenant_id: String,
}

/// Standard response format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

/// Dashboard response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DashboardResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub panels: Vec<DashboardPanel>,
    pub tenant_id: String,
}

/// Incident response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IncidentResponse {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub assigned_to: Option<String>,
    pub resolution_notes: Option<String>,
    pub tenant_id: String,
}

impl From<Incident> for IncidentResponse {
    fn from(incident: Incident) -> Self {
        IncidentResponse {
            id: incident.id,
            title: incident.title,
            description: incident.description,
            severity: format!("{:?}", incident.severity),
            status: format!("{:?}", incident.status),
            created_at: incident.created_at.to_rfc3339(),
            updated_at: incident.updated_at.to_rfc3339(),
            assigned_to: incident.assigned_to,
            resolution_notes: incident.resolution_notes,
            tenant_id: incident.tenant_id,
        }
    }
}

/// Alert rule response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AlertRuleResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub query: String,
    pub threshold: f64,
    pub severity: String,
    pub enabled: bool,
    pub created_at: String,
    pub tenant_id: String,
}

impl From<AlertRule> for AlertRuleResponse {
    fn from(rule: AlertRule) -> Self {
        AlertRuleResponse {
            id: rule.id,
            name: rule.name,
            description: rule.description,
            query: rule.query,
            threshold: rule.threshold,
            severity: format!("{:?}", rule.severity),
            enabled: rule.enabled,
            created_at: rule.created_at.to_rfc3339(),
            tenant_id: rule.tenant_id,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    // Create monitoring system
    let monitoring_system = MonitoringSystem::new()?;
    
    // Create app state
    let app_state = Arc::new(AppState {
        monitoring_system: Arc::new(RwLock::new(monitoring_system)),
    });
    
    // Create router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(get_metrics))
        .route("/dashboards", post(create_dashboard))
        .route("/dashboards/:id", get(get_dashboard))
        .route("/dashboards/tenant/:tenant_id", get(list_tenant_dashboards))
        .route("/incidents", post(create_incident))
        .route("/incidents/:id", get(get_incident))
        .route("/incidents/tenant/:tenant_id", get(list_tenant_incidents))
        .route("/alerts", post(create_alert_rule))
        .layer(Extension(app_state));
    
    // Run server
    let addr = format!("0.0.0.0:{}", args.port);
    tracing::info!("Monitoring service listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
        
    Ok(())
}

/// Health check endpoint
async fn health_check() -> Json<ApiResponse<String>> {
    let response = ApiResponse {
        success: true,
        data: Some("Monitoring service is healthy".to_string()),
        message: None,
    };
    Json(response)
}

/// Get metrics in Prometheus format
async fn get_metrics(
    Extension(state): Extension<Arc<AppState>>,
) -> Result<String, (axum::http::StatusCode, String)> {
    let monitoring_system = state.monitoring_system.read().await;
    match monitoring_system.get_metrics_text() {
        Ok(metrics) => Ok(metrics),
        Err(e) => Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// Create a dashboard
async fn create_dashboard(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<CreateDashboardRequest>,
) -> Json<ApiResponse<DashboardResponse>> {
    let dashboard = {
        let mut monitoring_system = state.monitoring_system.write().await;
        let dashboard_manager = monitoring_system.dashboard_manager();
        dashboard_manager.create_dashboard(
            &payload.name,
            &payload.description,
            payload.panels,
            &payload.tenant_id,
        )
    };
    
    let response = DashboardResponse {
        id: dashboard.id,
        name: dashboard.name,
        description: dashboard.description,
        created_at: dashboard.created_at.to_rfc3339(),
        panels: dashboard.panels,
        tenant_id: dashboard.tenant_id,
    };
    
    let api_response = ApiResponse {
        success: true,
        data: Some(response),
        message: Some("Dashboard created successfully".to_string()),
    };
    Json(api_response)
}

/// Get a dashboard by ID
async fn get_dashboard(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<DashboardResponse>> {
    let dashboard_opt = {
        let monitoring_system = state.monitoring_system.read().await;
        monitoring_system.dashboard_manager_ref().get_dashboard(&id).cloned()
    };
    
    match dashboard_opt {
        Some(dashboard) => {
            let response = DashboardResponse {
                id: dashboard.id,
                name: dashboard.name,
                description: dashboard.description,
                created_at: dashboard.created_at.to_rfc3339(),
                panels: dashboard.panels,
                tenant_id: dashboard.tenant_id,
            };
            
            let api_response = ApiResponse {
                success: true,
                data: Some(response),
                message: None,
            };
            Json(api_response)
        },
        None => {
            let api_response = ApiResponse {
                success: false,
                data: None,
                message: Some("Dashboard not found".to_string()),
            };
            Json(api_response)
        },
    }
}

/// List dashboards for a tenant
async fn list_tenant_dashboards(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(tenant_id): axum::extract::Path<String>,
) -> Json<ApiResponse<Vec<DashboardResponse>>> {
    let dashboards = {
        let monitoring_system = state.monitoring_system.read().await;
        monitoring_system.dashboard_manager_ref().list_tenant_dashboards(&tenant_id)
            .into_iter()
            .map(|dashboard| DashboardResponse {
                id: dashboard.id.clone(),
                name: dashboard.name.clone(),
                description: dashboard.description.clone(),
                created_at: dashboard.created_at.to_rfc3339(),
                panels: dashboard.panels.clone(),
                tenant_id: dashboard.tenant_id.clone(),
            })
            .collect::<Vec<DashboardResponse>>()
    };
    
    let api_response = ApiResponse {
        success: true,
        data: Some(dashboards),
        message: None,
    };
    Json(api_response)
}

/// Create an incident
async fn create_incident(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<CreateIncidentRequest>,
) -> Json<ApiResponse<IncidentResponse>> {
    // Parse severity from string
    let severity = match payload.severity.as_str() {
        "Low" => IncidentSeverity::Low,
        "Medium" => IncidentSeverity::Medium,
        "High" => IncidentSeverity::High,
        "Critical" => IncidentSeverity::Critical,
        _ => IncidentSeverity::Medium,
    };
    
    let incident = {
        let mut monitoring_system = state.monitoring_system.write().await;
        let incident_manager = monitoring_system.incident_manager();
        incident_manager.create_incident(
            &payload.title,
            &payload.description,
            severity,
            &payload.tenant_id,
        )
    };
    
    let response = IncidentResponse::from(incident);
    
    let api_response = ApiResponse {
        success: true,
        data: Some(response),
        message: Some("Incident created successfully".to_string()),
    };
    Json(api_response)
}

/// Get an incident by ID
async fn get_incident(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<IncidentResponse>> {
    let incident_opt = {
        let monitoring_system = state.monitoring_system.read().await;
        monitoring_system.incident_manager_ref().get_incident(&id).cloned()
    };
    
    match incident_opt {
        Some(incident) => {
            let response = IncidentResponse::from(incident);
            
            let api_response = ApiResponse {
                success: true,
                data: Some(response),
                message: None,
            };
            Json(api_response)
        },
        None => {
            let api_response = ApiResponse {
                success: false,
                data: None,
                message: Some("Incident not found".to_string()),
            };
            Json(api_response)
        },
    }
}

/// List incidents for a tenant
async fn list_tenant_incidents(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(tenant_id): axum::extract::Path<String>,
) -> Json<ApiResponse<Vec<IncidentResponse>>> {
    let incidents = {
        let monitoring_system = state.monitoring_system.read().await;
        monitoring_system.incident_manager_ref().list_tenant_incidents(&tenant_id)
            .into_iter()
            .map(|incident| IncidentResponse::from((*incident).clone()))
            .collect::<Vec<IncidentResponse>>()
    };
    
    let api_response = ApiResponse {
        success: true,
        data: Some(incidents),
        message: None,
    };
    Json(api_response)
}

/// Create an alert rule
async fn create_alert_rule(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<CreateAlertRuleRequest>,
) -> Json<ApiResponse<AlertRuleResponse>> {
    // Parse severity from string
    let severity = match payload.severity.as_str() {
        "Low" => IncidentSeverity::Low,
        "Medium" => IncidentSeverity::Medium,
        "High" => IncidentSeverity::High,
        "Critical" => IncidentSeverity::Critical,
        _ => IncidentSeverity::Medium,
    };
    
    let rule = {
        let mut monitoring_system = state.monitoring_system.write().await;
        let incident_manager = monitoring_system.incident_manager();
        incident_manager.create_alert_rule(
            &payload.name,
            &payload.description,
            &payload.query,
            payload.threshold,
            severity,
            &payload.tenant_id,
        )
    };
    
    let response = AlertRuleResponse::from(rule);
    
    let api_response = ApiResponse {
        success: true,
        data: Some(response),
        message: Some("Alert rule created successfully".to_string()),
    };
    Json(api_response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing() {
        let args = Args::parse_from(["svc-monitoring", "--port", "8087"]);
        assert_eq!(args.port, 8087);
    }

    #[tokio::test]
    async fn test_monitoring_service_creation() -> Result<()> {
        let monitoring_system = MonitoringSystem::new()?;
        let _app_state = Arc::new(AppState {
            monitoring_system: Arc::new(RwLock::new(monitoring_system)),
        });
        
        Ok(())
    }
}