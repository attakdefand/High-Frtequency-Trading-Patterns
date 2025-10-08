//! User management service for the sniper-rs enterprise features.
//! 
//! This service provides REST APIs for multi-user support with isolated contexts,
//! advanced RBAC, and audit logging.

use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use axum::{
    routing::{get, post},
    Json, Router, Extension,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use sniper_users::{UserManager, UserRole, User, UserContext, AuditLog};

/// CLI arguments for the user service
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[clap(short, long, default_value = "8084")]
    port: u16,
}

/// User service state
struct AppState {
    user_manager: RwLock<UserManager>,
}

/// User creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub roles: Vec<String>, // Will be parsed into UserRole
    pub tenant_id: String,
}

/// User authentication request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuthenticateUserRequest {
    pub username: String,
}

/// Role assignment request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AssignRoleRequest {
    pub role: String, // Will be parsed into UserRole
}

/// Standard response format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

/// User response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
    pub tenant_id: String,
    pub created_at: String,
    pub last_login: Option<String>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            roles: user.roles.iter().map(|r| format!("{:?}", r)).collect(),
            tenant_id: user.tenant_id,
            created_at: user.created_at.to_rfc3339(),
            last_login: user.last_login.map(|dt| dt.to_rfc3339()),
        }
    }
}

/// User context response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserContextResponse {
    pub user_id: String,
    pub tenant_id: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

impl From<UserContext> for UserContextResponse {
    fn from(context: UserContext) -> Self {
        UserContextResponse {
            user_id: context.user_id,
            tenant_id: context.tenant_id,
            roles: context.roles.iter().map(|r| format!("{:?}", r)).collect(),
            permissions: context.permissions,
        }
    }
}

/// Audit log response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuditLogResponse {
    pub id: String,
    pub user_id: String,
    pub action: String,
    pub resource: String,
    pub timestamp: String,
    pub details: Option<String>,
}

impl From<AuditLog> for AuditLogResponse {
    fn from(log: AuditLog) -> Self {
        AuditLogResponse {
            id: log.id,
            user_id: log.user_id,
            action: log.action,
            resource: log.resource,
            timestamp: log.timestamp.to_rfc3339(),
            details: log.details,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    // Create user manager
    let user_manager = UserManager::new();
    
    // Create app state
    let app_state = Arc::new(AppState {
        user_manager: RwLock::new(user_manager),
    });
    
    // Create router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/users", post(create_user))
        .route("/users/:id", get(get_user))
        .route("/auth", post(authenticate_user))
        .route("/users/:id/roles", post(assign_role))
        .route("/users/:id/context", get(get_user_context))
        .route("/users/:id/audit", get(get_user_audit_logs))
        .route("/audit", get(get_all_audit_logs))
        .layer(Extension(app_state));
    
    // Run server
    let addr = format!("0.0.0.0:{}", args.port);
    tracing::info!("User service listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
        
    Ok(())
}

/// Health check endpoint
async fn health_check() -> Json<ApiResponse<String>> {
    let response = ApiResponse {
        success: true,
        data: Some("User service is healthy".to_string()),
        message: None,
    };
    Json(response)
}

/// Create a new user
async fn create_user(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>,
) -> Json<ApiResponse<UserResponse>> {
    // Parse roles from strings to UserRole enum
    let roles: Vec<UserRole> = payload.roles
        .iter()
        .map(|role| match role.as_str() {
            "Admin" => UserRole::Admin,
            "Trader" => UserRole::Trader,
            "Analyst" => UserRole::Analyst,
            "Auditor" => UserRole::Auditor,
            _ => UserRole::Guest,
        })
        .collect();
    
    let result = state.user_manager.write().await.create_user(
        &payload.username,
        &payload.email,
        roles,
        &payload.tenant_id,
    );
    
    match result {
        Ok(user) => {
            let response = ApiResponse {
                success: true,
                data: Some(UserResponse::from(user)),
                message: Some("User created successfully".to_string()),
            };
            Json(response)
        },
        Err(e) => {
            let response = ApiResponse {
                success: false,
                data: None,
                message: Some(format!("Failed to create user: {}", e)),
            };
            Json(response)
        },
    }
}

/// Get a user by ID
async fn get_user(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<UserResponse>> {
    let user_opt = state.user_manager.read().await.get_user(&id).cloned();
    
    match user_opt {
        Some(user) => {
            let response = ApiResponse {
                success: true,
                data: Some(UserResponse::from(user)),
                message: None,
            };
            Json(response)
        },
        None => {
            let response = ApiResponse {
                success: false,
                data: None,
                message: Some("User not found".to_string()),
            };
            Json(response)
        },
    }
}

/// Authenticate a user
async fn authenticate_user(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<AuthenticateUserRequest>,
) -> Json<ApiResponse<UserContextResponse>> {
    let context_opt = state.user_manager.write().await.authenticate_user(&payload.username);
    
    match context_opt {
        Some(context) => {
            let response = ApiResponse {
                success: true,
                data: Some(UserContextResponse::from(context)),
                message: Some("User authenticated successfully".to_string()),
            };
            Json(response)
        },
        None => {
            let response = ApiResponse {
                success: false,
                data: None,
                message: Some("Authentication failed".to_string()),
            };
            Json(response)
        },
    }
}

/// Assign a role to a user
async fn assign_role(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<AssignRoleRequest>,
) -> Json<ApiResponse<bool>> {
    // Parse role from string to UserRole enum
    let role = match payload.role.as_str() {
        "Admin" => UserRole::Admin,
        "Trader" => UserRole::Trader,
        "Analyst" => UserRole::Analyst,
        "Auditor" => UserRole::Auditor,
        _ => UserRole::Guest,
    };
    
    let result = state.user_manager.write().await.add_user_role(&id, role);
    
    match result {
        Ok(_) => {
            let response = ApiResponse {
                success: true,
                data: Some(true),
                message: Some("Role assigned successfully".to_string()),
            };
            Json(response)
        },
        Err(e) => {
            let response = ApiResponse {
                success: false,
                data: Some(false),
                message: Some(format!("Failed to assign role: {}", e)),
            };
            Json(response)
        },
    }
}

/// Get user context
async fn get_user_context(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<UserContextResponse>> {
    let context_opt = state.user_manager.read().await.get_user_context(&id);
    
    match context_opt {
        Some(context) => {
            let response = ApiResponse {
                success: true,
                data: Some(UserContextResponse::from(context)),
                message: None,
            };
            Json(response)
        },
        None => {
            let response = ApiResponse {
                success: false,
                data: None,
                message: Some("User context not found".to_string()),
            };
            Json(response)
        },
    }
}

/// Get user audit logs
async fn get_user_audit_logs(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<ApiResponse<Vec<AuditLogResponse>>> {
    let logs = state.user_manager.read().await.get_user_audit_logs(&id)
        .iter()
        .map(|&log| AuditLogResponse::from(log.clone()))
        .collect::<Vec<AuditLogResponse>>();
    
    let response = ApiResponse {
        success: true,
        data: Some(logs),
        message: None,
    };
    Json(response)
}

/// Get all audit logs
async fn get_all_audit_logs(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<ApiResponse<Vec<AuditLogResponse>>> {
    let logs = state.user_manager.read().await.get_all_audit_logs()
        .iter()
        .map(|log| AuditLogResponse::from(log.clone()))
        .collect::<Vec<AuditLogResponse>>();
    
    let response = ApiResponse {
        success: true,
        data: Some(logs),
        message: None,
    };
    Json(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing() {
        let args = Args::parse_from(["svc-users", "--port", "8085"]);
        assert_eq!(args.port, 8085);
    }

    #[tokio::test]
    async fn test_user_service_creation() -> Result<()> {
        let user_manager = UserManager::new();
        let _app_state = Arc::new(AppState {
            user_manager: RwLock::new(user_manager),
        });
        
        Ok(())
    }
}