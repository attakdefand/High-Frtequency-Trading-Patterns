//! Integration tests for the user service and enterprise features

use sniper_users::{UserManager, UserRole};

#[test]
fn test_user_management_enterprise_features() {
    let mut user_manager = UserManager::new();
    
    // Test creating a user with multiple roles
    let roles = vec![UserRole::Admin, UserRole::Trader, UserRole::Analyst];
    let user = user_manager.create_user(
        "enterprise_user",
        "enterprise@example.com",
        roles,
        "enterprise-tenant-1",
    ).expect("Failed to create enterprise user");
    
    assert_eq!(user.username, "enterprise_user");
    assert_eq!(user.email, "enterprise@example.com");
    assert_eq!(user.tenant_id, "enterprise-tenant-1");
    assert_eq!(user.roles.len(), 3);
    
    // Test user authentication
    let context = user_manager.authenticate_user("enterprise_user");
    assert!(context.is_some());
    let context = context.unwrap();
    assert_eq!(context.user_id, user.id);
    assert_eq!(context.tenant_id, "enterprise-tenant-1");
    
    // Test RBAC permissions
    assert!(user_manager.user_has_permission(&user.id, "manage_users"));
    assert!(user_manager.user_has_permission(&user.id, "execute_trades"));
    assert!(user_manager.user_has_permission(&user.id, "view_portfolio"));
    assert!(!user_manager.user_has_permission(&user.id, "view_audit_logs"));
    
    // Test adding additional role
    user_manager.add_user_role(&user.id, UserRole::Auditor)
        .expect("Failed to add auditor role");
    
    // Verify new permission is available
    assert!(user_manager.user_has_permission(&user.id, "view_audit_logs"));
}

#[test]
fn test_audit_logging_enterprise_features() {
    let mut user_manager = UserManager::new();
    
    // Create a user
    let roles = vec![UserRole::Admin];
    let user = user_manager.create_user(
        "audit_user",
        "audit@example.com",
        roles,
        "audit-tenant-1",
    ).expect("Failed to create audit user");
    
    // Perform actions that should be logged
    user_manager.authenticate_user("audit_user");
    user_manager.add_user_role(&user.id, UserRole::Trader)
        .expect("Failed to add trader role");
    
    // Check audit logs
    let user_logs = user_manager.get_user_audit_logs(&user.id);
    assert!(!user_logs.is_empty());
    assert_eq!(user_logs.len(), 3); // create, authenticate, add_role
    
    let all_logs = user_manager.get_all_audit_logs();
    assert!(!all_logs.is_empty());
    assert!(all_logs.len() >= 3);
    
    // Check log content
    let first_log = &user_logs[0];
    assert_eq!(first_log.user_id, user.id);
    assert_eq!(first_log.action, "CREATE_USER");
    assert_eq!(first_log.resource, "users");
}

#[test]
fn test_multi_tenant_isolation() {
    let mut user_manager = UserManager::new();
    
    // Create users for different tenants
    let tenant1_roles = vec![UserRole::Admin];
    let tenant2_roles = vec![UserRole::Trader];
    
    let user1 = user_manager.create_user(
        "tenant1_user",
        "tenant1@example.com",
        tenant1_roles,
        "tenant-1",
    ).expect("Failed to create tenant1 user");
    
    let user2 = user_manager.create_user(
        "tenant2_user",
        "tenant2@example.com",
        tenant2_roles,
        "tenant-2",
    ).expect("Failed to create tenant2 user");
    
    // Verify tenant isolation through user contexts
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