//! User management system for the sniper-rs enterprise features.
//! 
//! This module provides functionality for multi-user support with isolated contexts,
//! advanced RBAC (Role-Based Access Control), and audit logging.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// User roles for RBAC
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UserRole {
    Admin,
    Trader,
    Analyst,
    Auditor,
    Guest,
}

/// User context for isolated environments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: String,
    pub tenant_id: String,
    pub roles: Vec<UserRole>,
    pub permissions: Vec<String>,
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub roles: Vec<UserRole>,
    pub tenant_id: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: String,
    pub user_id: String,
    pub action: String,
    pub resource: String,
    pub timestamp: DateTime<Utc>,
    pub details: Option<String>,
}

/// Role-Based Access Control manager
pub struct RBACManager {
    roles_permissions: HashMap<UserRole, Vec<String>>,
}

impl RBACManager {
    /// Create a new RBAC manager
    pub fn new() -> Self {
        let mut roles_permissions = HashMap::new();
        
        // Define permissions for each role
        roles_permissions.insert(UserRole::Admin, vec![
            "manage_users".to_string(),
            "manage_roles".to_string(),
            "view_all_data".to_string(),
            "execute_trades".to_string(),
            "view_reports".to_string(),
            "configure_system".to_string(),
        ]);
        
        roles_permissions.insert(UserRole::Trader, vec![
            "execute_trades".to_string(),
            "view_portfolio".to_string(),
            "view_orders".to_string(),
        ]);
        
        roles_permissions.insert(UserRole::Analyst, vec![
            "view_portfolio".to_string(),
            "view_orders".to_string(),
            "view_reports".to_string(),
        ]);
        
        roles_permissions.insert(UserRole::Auditor, vec![
            "view_audit_logs".to_string(),
            "view_reports".to_string(),
        ]);
        
        roles_permissions.insert(UserRole::Guest, vec![
            "view_public_data".to_string(),
        ]);
        
        Self { roles_permissions }
    }
    
    /// Check if a user has a specific permission
    pub fn has_permission(&self, user: &User, permission: &str) -> bool {
        user.roles.iter().any(|role| {
            self.roles_permissions
                .get(role)
                .map(|permissions| permissions.contains(&permission.to_string()))
                .unwrap_or(false)
        })
    }
    
    /// Get all permissions for a user
    pub fn get_user_permissions(&self, user: &User) -> Vec<String> {
        let mut permissions = Vec::new();
        for role in &user.roles {
            if let Some(role_permissions) = self.roles_permissions.get(role) {
                permissions.extend(role_permissions.clone());
            }
        }
        // Remove duplicates
        permissions.sort();
        permissions.dedup();
        permissions
    }
}

/// User manager for multi-user support
pub struct UserManager {
    users: HashMap<String, User>,
    rbac: RBACManager,
    audit_logs: Vec<AuditLog>,
}

impl UserManager {
    /// Create a new user manager
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            rbac: RBACManager::new(),
            audit_logs: Vec::new(),
        }
    }
    
    /// Create a new user
    pub fn create_user(&mut self, username: &str, email: &str, roles: Vec<UserRole>, tenant_id: &str) -> Result<User> {
        let user = User {
            id: Uuid::new_v4().to_string(),
            username: username.to_string(),
            email: email.to_string(),
            roles,
            tenant_id: tenant_id.to_string(),
            created_at: Utc::now(),
            last_login: None,
        };
        
        self.users.insert(user.id.clone(), user.clone());
        
        // Log the creation
        self.log_audit(&user.id, "CREATE_USER", "users", Some(format!("Created user {}", username)));
        
        Ok(user)
    }
    
    /// Get a user by ID
    pub fn get_user(&self, user_id: &str) -> Option<&User> {
        self.users.get(user_id)
    }
    
    /// Get a user by username
    pub fn get_user_by_username(&self, username: &str) -> Option<&User> {
        self.users.values().find(|user| user.username == username)
    }
    
    /// Authenticate a user (simplified for demo)
    pub fn authenticate_user(&mut self, username: &str) -> Option<UserContext> {
        if let Some(user) = self.get_user_by_username(username) {
            let mut user = user.clone();
            user.last_login = Some(Utc::now());
            
            // Update the user with the new login time
            self.users.insert(user.id.clone(), user.clone());
            
            // Log the authentication
            self.log_audit(&user.id, "LOGIN", "auth", Some(format!("User {} logged in", username)));
            
            Some(UserContext {
                user_id: user.id.clone(),
                tenant_id: user.tenant_id.clone(),
                roles: user.roles.clone(),
                permissions: self.rbac.get_user_permissions(&user),
            })
        } else {
            None
        }
    }
    
    /// Check if a user has a specific permission
    pub fn user_has_permission(&self, user_id: &str, permission: &str) -> bool {
        if let Some(user) = self.get_user(user_id) {
            self.rbac.has_permission(user, permission)
        } else {
            false
        }
    }
    
    /// Add a role to a user
    pub fn add_user_role(&mut self, user_id: &str, role: UserRole) -> Result<()> {
        if let Some(user) = self.users.get_mut(user_id) {
            if !user.roles.contains(&role) {
                user.roles.push(role.clone());
                self.log_audit(user_id, "ADD_ROLE", "users", Some(format!("Added role {:?} to user", role)));
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("User not found"))
        }
    }
    
    /// Log an audit entry
    pub fn log_audit(&mut self, user_id: &str, action: &str, resource: &str, details: Option<String>) {
        let log_entry = AuditLog {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            action: action.to_string(),
            resource: resource.to_string(),
            timestamp: Utc::now(),
            details,
        };
        
        self.audit_logs.push(log_entry);
    }
    
    /// Get audit logs for a user
    pub fn get_user_audit_logs(&self, user_id: &str) -> Vec<&AuditLog> {
        self.audit_logs
            .iter()
            .filter(|log| log.user_id == user_id)
            .collect()
    }
    
    /// Get all audit logs
    pub fn get_all_audit_logs(&self) -> &Vec<AuditLog> {
        &self.audit_logs
    }
    
    /// Get user context for isolated environments
    pub fn get_user_context(&self, user_id: &str) -> Option<UserContext> {
        if let Some(user) = self.get_user(user_id) {
            Some(UserContext {
                user_id: user.id.clone(),
                tenant_id: user.tenant_id.clone(),
                roles: user.roles.clone(),
                permissions: self.rbac.get_user_permissions(user),
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let mut user_manager = UserManager::new();
        let user = user_manager.create_user(
            "testuser", 
            "test@example.com", 
            vec![UserRole::Trader], 
            "tenant-1"
        ).unwrap();
        
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.roles, vec![UserRole::Trader]);
        assert_eq!(user.tenant_id, "tenant-1");
    }

    #[test]
    fn test_user_authentication() {
        let mut user_manager = UserManager::new();
        let user = user_manager.create_user(
            "testuser", 
            "test@example.com", 
            vec![UserRole::Trader], 
            "tenant-1"
        ).unwrap();
        
        let context = user_manager.authenticate_user("testuser");
        assert!(context.is_some());
        let context = context.unwrap();
        assert_eq!(context.user_id, user.id);
        assert_eq!(context.tenant_id, "tenant-1");
        assert!(context.permissions.contains(&"execute_trades".to_string()));
    }

    #[test]
    fn test_rbac_permissions() {
        let mut user_manager = UserManager::new();
        let user = user_manager.create_user(
            "adminuser", 
            "admin@example.com", 
            vec![UserRole::Admin], 
            "tenant-1"
        ).unwrap();
        
        assert!(user_manager.user_has_permission(&user.id, "manage_users"));
        assert!(user_manager.user_has_permission(&user.id, "execute_trades"));
        assert!(!user_manager.user_has_permission(&user.id, "nonexistent_permission"));
    }

    #[test]
    fn test_audit_logging() {
        let mut user_manager = UserManager::new();
        let user = user_manager.create_user(
            "testuser", 
            "test@example.com", 
            vec![UserRole::Trader], 
            "tenant-1"
        ).unwrap();
        
        user_manager.log_audit(&user.id, "TEST_ACTION", "test_resource", Some("test details".to_string()));
        
        let logs = user_manager.get_user_audit_logs(&user.id);
        assert_eq!(logs.len(), 2); // One from creation, one from manual log
        
        let all_logs = user_manager.get_all_audit_logs();
        assert_eq!(all_logs.len(), 2);
    }

    #[test]
    fn test_user_context_isolation() {
        let mut user_manager = UserManager::new();
        let user1 = user_manager.create_user(
            "user1", 
            "user1@example.com", 
            vec![UserRole::Trader], 
            "tenant-1"
        ).unwrap();
        
        let user2 = user_manager.create_user(
            "user2", 
            "user2@example.com", 
            vec![UserRole::Trader], 
            "tenant-2"
        ).unwrap();
        
        let context1 = user_manager.get_user_context(&user1.id);
        let context2 = user_manager.get_user_context(&user2.id);
        
        assert!(context1.is_some());
        assert!(context2.is_some());
        
        let context1 = context1.unwrap();
        let context2 = context2.unwrap();
        
        assert_eq!(context1.tenant_id, "tenant-1");
        assert_eq!(context2.tenant_id, "tenant-2");
        assert_ne!(context1.user_id, context2.user_id);
    }
}