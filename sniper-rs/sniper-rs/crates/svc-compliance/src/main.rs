//! Compliance service for the sniper-rs enterprise features.
//! 
//! This service provides REST APIs for compliance reporting, disaster recovery,
//! and backup/restore capabilities.

use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use axum::{
    routing::{get, post},
    Json, Router, Extension,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use sniper_compliance::{
    ComplianceManager, 
    BackupManager, 
    DisasterRecoveryManager, 
    ReportType, 
    ComplianceReport, 
    BackupMetadata, 
    DisasterRecoveryPlan,
    RecoveryStep
};
use chrono::{DateTime, Utc};

/// CLI arguments for the compliance service
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[clap(short, long, default_value = "8085")]
    port: u16,
}

/// Compliance service state
struct AppState {
    compliance_manager: RwLock<ComplianceManager>,
    backup_manager: RwLock<BackupManager>,
    dr_manager: RwLock<DisasterRecoveryManager>,
}

/// Report generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GenerateReportRequest {
    pub report_type: String, // Will be parsed into ReportType
    pub period_start: String, // ISO 8601 format
    pub period_end: String, // ISO 8601 format
    pub generated_by: String,
    pub tenant_id: String,
}

/// Backup creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateBackupRequest {
    pub components: Vec<String>,
    pub tenant_id: String,
}

/// Disaster recovery plan creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateDRPlanRequest {
    pub name: String,
    pub description: String,
    pub steps: Vec<RecoveryStep>,
    pub tenant_id: String,
}

/// Standard response format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

/// Compliance report response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReportResponse {
    pub id: String,
    pub report_type: String,
    pub generated_at: String,
    pub period_start: String,
    pub period_end: String,
    pub content: String,
    pub generated_by: String,
    pub tenant_id: String,
}

impl From<ComplianceReport> for ReportResponse {
    fn from(report: ComplianceReport) -> Self {
        ReportResponse {
            id: report.id,
            report_type: format!("{:?}", report.report_type),
            generated_at: report.generated_at.to_rfc3339(),
            period_start: report.period_start.to_rfc3339(),
            period_end: report.period_end.to_rfc3339(),
            content: report.content,
            generated_by: report.generated_by,
            tenant_id: report.tenant_id,
        }
    }
}

/// Backup metadata response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupResponse {
    pub id: String,
    pub created_at: String,
    pub size_bytes: u64,
    pub checksum: String,
    pub components: Vec<String>,
    pub tenant_id: String,
}

impl From<BackupMetadata> for BackupResponse {
    fn from(backup: BackupMetadata) -> Self {
        BackupResponse {
            id: backup.id,
            created_at: backup.created_at.to_rfc3339(),
            size_bytes: backup.size_bytes,
            checksum: backup.checksum,
            components: backup.components,
            tenant_id: backup.tenant_id,
        }
    }
}

/// Disaster recovery plan response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DRPlanResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub last_updated: String,
    pub steps: Vec<RecoveryStep>,
    pub tenant_id: String,
}

impl From<DisasterRecoveryPlan> for DRPlanResponse {
    fn from(plan: DisasterRecoveryPlan) -> Self {
        DRPlanResponse {
            id: plan.id,
            name: plan.name,
            description: plan.description,
            created_at: plan.created_at.to_rfc3339(),
            last_updated: plan.last_updated.to_rfc3339(),
            steps: plan.steps,
            tenant_id: plan.tenant_id,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    // Create managers
    let compliance_manager = ComplianceManager::new();
    let backup_manager = BackupManager::new();
    let dr_manager = DisasterRecoveryManager::new();
    
    // Create app state
    let app_state = Arc::new(AppState {
        compliance_manager: RwLock::new(compliance_manager),
        backup_manager: RwLock::new(backup_manager),
        dr_manager: RwLock::new(dr_manager),
    });
    
    // Create router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/reports", post(generate_report))
        .route("/reports/:id", get(get_report))
        .route("/reports/tenant/:tenant_id", get(list_tenant_reports))
        .route("/reports/:id/export", post(export_report))
        .route("/backups", post(create_backup))
        .route("/backups/:id", get(get_backup))
        .route("/backups/tenant/:tenant_id", get(list_tenant_backups))
        .route("/backups/:id/restore", post(restore_backup))
        .route("/dr-plans", post(create_dr_plan))
        .route("/dr-plans/:id", get(get_dr_plan))
        .route("/dr-plans/tenant/:tenant_id", get(list_tenant_dr_plans))
        .route("/dr-plans/:id/execute", post(execute_dr_plan))
        .layer(Extension(app_state));
    
    // Run server
    let addr = format!("0.0.0.0:{}", args.port);
    tracing::info!("Compliance service listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
        
    Ok(())
}

/// Health check endpoint
async fn health_check() -> Json<ApiResponse<String>> {
    let response = ApiResponse {
        success: true,
        data: Some("Compliance service is healthy".to_string()),
        message: None,
    };
    Json(response)
}

/// Generate a compliance report
async fn generate_report(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<GenerateReportRequest>,
) -> Json<ApiResponse<ReportResponse>> {
    // Parse report type from string
    let report_type = match payload.report_type.as_str() {
        "DailyActivity" => ReportType::DailyActivity,
        "TradeAudit" => ReportType::TradeAudit,
        "RiskAssessment" => ReportType::RiskAssessment,
        "RegulatoryCompliance" => ReportType::RegulatoryCompliance,
        "FinancialSummary" => ReportType::FinancialSummary,
        _ => ReportType::DailyActivity,
    };
    
    // Parse dates
    let period_start = payload.period_start.parse::<DateTime<Utc>>()
        .unwrap_or_else(|_| Utc::now());
    let period_end = payload.period_end.parse::<DateTime<Utc>>()
        .unwrap_or_else(|_| Utc::now());
    
    let result = state.compliance_manager.write().await.generate_report(
        report_type,
        period_start,
        period_end,
        &payload.generated_by,
        &payload.tenant_id,
    );
    
    match result {
        Ok(report) => {
            let response = ApiResponse {
                success: true,
                data: Some(ReportResponse::from(report)),
                message: Some("Report generated successfully".to_string()),
            };
            Json(response)
        },
        Err(e) => {
            let response = ApiResponse {
                success: false,
                data: None,
                message: Some(format!("Failed to generate report: {}", e)),
            };
            Json(response)
        },
    }
}

/// Get a report by ID
async fn get_report(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<ReportResponse>> {
    let report_opt = state.compliance_manager.read().await.get_report(&id).cloned();
    
    match report_opt {
        Some(report) => {
            let response = ApiResponse {
                success: true,
                data: Some(ReportResponse::from(report)),
                message: None,
            };
            Json(response)
        },
        None => {
            let response = ApiResponse {
                success: false,
                data: None,
                message: Some("Report not found".to_string()),
            };
            Json(response)
        },
    }
}

/// List reports for a tenant
async fn list_tenant_reports(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(tenant_id): axum::extract::Path<String>,
) -> Json<ApiResponse<Vec<ReportResponse>>> {
    let reports = state.compliance_manager.read().await.get_tenant_reports(&tenant_id)
        .iter()
        .map(|&report| ReportResponse::from(report.clone()))
        .collect::<Vec<ReportResponse>>();
    
    let response = ApiResponse {
        success: true,
        data: Some(reports),
        message: None,
    };
    Json(response)
}

/// Export a report
async fn export_report(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Json<ApiResponse<String>> {
    let format = payload.get("format")
        .and_then(|v| v.as_str())
        .unwrap_or("json");
    
    let result = state.compliance_manager.read().await.export_report(&id, format);
    
    match result {
        Ok(data) => {
            let response = ApiResponse {
                success: true,
                data: Some(base64::encode(data)),
                message: Some("Report exported successfully".to_string()),
            };
            Json(response)
        },
        Err(e) => {
            let response = ApiResponse {
                success: false,
                data: None,
                message: Some(format!("Failed to export report: {}", e)),
            };
            Json(response)
        },
    }
}

/// Create a backup
async fn create_backup(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<CreateBackupRequest>,
) -> Json<ApiResponse<BackupResponse>> {
    let result = state.backup_manager.write().await.create_backup(
        payload.components,
        &payload.tenant_id,
    );
    
    match result {
        Ok(backup) => {
            let response = ApiResponse {
                success: true,
                data: Some(BackupResponse::from(backup)),
                message: Some("Backup created successfully".to_string()),
            };
            Json(response)
        },
        Err(e) => {
            let response = ApiResponse {
                success: false,
                data: None,
                message: Some(format!("Failed to create backup: {}", e)),
            };
            Json(response)
        },
    }
}

/// Get a backup by ID
async fn get_backup(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<BackupResponse>> {
    let backup_opt = state.backup_manager.read().await.get_backup(&id).cloned();
    
    match backup_opt {
        Some(backup) => {
            let response = ApiResponse {
                success: true,
                data: Some(BackupResponse::from(backup)),
                message: None,
            };
            Json(response)
        },
        None => {
            let response = ApiResponse {
                success: false,
                data: None,
                message: Some("Backup not found".to_string()),
            };
            Json(response)
        },
    }
}

/// List backups for a tenant
async fn list_tenant_backups(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(tenant_id): axum::extract::Path<String>,
) -> Json<ApiResponse<Vec<BackupResponse>>> {
    let backups = state.backup_manager.read().await.list_tenant_backups(&tenant_id)
        .iter()
        .map(|&backup| BackupResponse::from(backup.clone()))
        .collect::<Vec<BackupResponse>>();
    
    let response = ApiResponse {
        success: true,
        data: Some(backups),
        message: None,
    };
    Json(response)
}

/// Restore from a backup
async fn restore_backup(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<bool>> {
    let result = state.backup_manager.read().await.restore_from_backup(&id);
    
    match result {
        Ok(_) => {
            let response = ApiResponse {
                success: true,
                data: Some(true),
                message: Some("Backup restored successfully".to_string()),
            };
            Json(response)
        },
        Err(e) => {
            let response = ApiResponse {
                success: false,
                data: Some(false),
                message: Some(format!("Failed to restore backup: {}", e)),
            };
            Json(response)
        },
    }
}

/// Create a disaster recovery plan
async fn create_dr_plan(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<CreateDRPlanRequest>,
) -> Json<ApiResponse<DRPlanResponse>> {
    let plan = state.dr_manager.write().await.create_plan(
        &payload.name,
        &payload.description,
        payload.steps,
        &payload.tenant_id,
    );
    
    let response = ApiResponse {
        success: true,
        data: Some(DRPlanResponse::from(plan)),
        message: Some("Disaster recovery plan created successfully".to_string()),
    };
    Json(response)
}

/// Get a disaster recovery plan by ID
async fn get_dr_plan(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<DRPlanResponse>> {
    let plan_opt = state.dr_manager.read().await.get_plan(&id).cloned();
    
    match plan_opt {
        Some(plan) => {
            let response = ApiResponse {
                success: true,
                data: Some(DRPlanResponse::from(plan)),
                message: None,
            };
            Json(response)
        },
        None => {
            let response = ApiResponse {
                success: false,
                data: None,
                message: Some("Disaster recovery plan not found".to_string()),
            };
            Json(response)
        },
    }
}

/// List disaster recovery plans for a tenant
async fn list_tenant_dr_plans(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(tenant_id): axum::extract::Path<String>,
) -> Json<ApiResponse<Vec<DRPlanResponse>>> {
    let plans = state.dr_manager.read().await.list_tenant_plans(&tenant_id)
        .iter()
        .map(|&plan| DRPlanResponse::from(plan.clone()))
        .collect::<Vec<DRPlanResponse>>();
    
    let response = ApiResponse {
        success: true,
        data: Some(plans),
        message: None,
    };
    Json(response)
}

/// Execute a disaster recovery plan
async fn execute_dr_plan(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<bool>> {
    let result = state.dr_manager.read().await.execute_plan(&id);
    
    match result {
        Ok(_) => {
            let response = ApiResponse {
                success: true,
                data: Some(true),
                message: Some("Disaster recovery plan executed successfully".to_string()),
            };
            Json(response)
        },
        Err(e) => {
            let response = ApiResponse {
                success: false,
                data: Some(false),
                message: Some(format!("Failed to execute disaster recovery plan: {}", e)),
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
        let args = Args::parse_from(["svc-compliance", "--port", "8086"]);
        assert_eq!(args.port, 8086);
    }

    #[tokio::test]
    async fn test_compliance_service_creation() -> Result<()> {
        let compliance_manager = ComplianceManager::new();
        let backup_manager = BackupManager::new();
        let dr_manager = DisasterRecoveryManager::new();
        
        let _app_state = Arc::new(AppState {
            compliance_manager: RwLock::new(compliance_manager),
            backup_manager: RwLock::new(backup_manager),
            dr_manager: RwLock::new(dr_manager),
        });
        
        Ok(())
    }
}