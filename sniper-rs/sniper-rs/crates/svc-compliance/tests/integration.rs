//! Integration tests for the compliance service and enterprise features

use sniper_compliance::{
    ComplianceManager, 
    BackupManager, 
    DisasterRecoveryManager, 
    ReportType,
    RecoveryStep
};
use chrono::Utc;

#[test]
fn test_compliance_reporting_enterprise_features() {
    let mut compliance_manager = ComplianceManager::new();
    
    // Test generating different types of compliance reports
    let daily_report = compliance_manager.generate_report(
        ReportType::DailyActivity,
        Utc::now() - chrono::Duration::days(1),
        Utc::now(),
        "compliance_officer",
        "compliance-tenant-1",
    ).expect("Failed to generate daily activity report");
    
    let audit_report = compliance_manager.generate_report(
        ReportType::TradeAudit,
        Utc::now() - chrono::Duration::days(7),
        Utc::now(),
        "compliance_officer",
        "compliance-tenant-1",
    ).expect("Failed to generate trade audit report");
    
    assert_eq!(daily_report.report_type, ReportType::DailyActivity);
    assert_eq!(audit_report.report_type, ReportType::TradeAudit);
    assert_eq!(daily_report.tenant_id, "compliance-tenant-1");
    assert_eq!(audit_report.tenant_id, "compliance-tenant-1");
    assert!(!daily_report.content.is_empty());
    assert!(!audit_report.content.is_empty());
    
    // Test retrieving reports
    let retrieved_daily = compliance_manager.get_report(&daily_report.id);
    let retrieved_audit = compliance_manager.get_report(&audit_report.id);
    
    assert!(retrieved_daily.is_some());
    assert!(retrieved_audit.is_some());
    assert_eq!(retrieved_daily.unwrap().id, daily_report.id);
    assert_eq!(retrieved_audit.unwrap().id, audit_report.id);
}

#[test]
fn test_backup_restore_enterprise_features() {
    let mut backup_manager = BackupManager::new();
    
    // Test creating backups with different components
    let core_components = vec![
        "users".to_string(), 
        "reports".to_string(), 
        "configurations".to_string()
    ];
    
    let full_backup = backup_manager.create_backup(
        core_components,
        "backup-tenant-1",
    ).expect("Failed to create full backup");
    
    assert_eq!(full_backup.tenant_id, "backup-tenant-1");
    assert_eq!(full_backup.components.len(), 3);
    assert!(!full_backup.checksum.is_empty());
    assert!(full_backup.size_bytes > 0);
    
    // Test retrieving backup
    let retrieved_backup = backup_manager.get_backup(&full_backup.id);
    assert!(retrieved_backup.is_some());
    assert_eq!(retrieved_backup.unwrap().id, full_backup.id);
    
    // Test listing tenant backups
    let tenant_backups = backup_manager.list_tenant_backups("backup-tenant-1");
    assert_eq!(tenant_backups.len(), 1);
    assert_eq!(tenant_backups[0].id, full_backup.id);
    
    // Test restore functionality
    backup_manager.restore_from_backup(&full_backup.id)
        .expect("Failed to restore from backup");
}

#[test]
fn test_disaster_recovery_enterprise_features() {
    let mut dr_manager = DisasterRecoveryManager::new();
    
    // Test creating a comprehensive disaster recovery plan
    let recovery_steps = vec![
        RecoveryStep {
            id: "step-1".to_string(),
            order: 1,
            description: "Assess system status and identify failure points".to_string(),
            expected_duration_minutes: 1,
            dependencies: vec![],
        },
        RecoveryStep {
            id: "step-2".to_string(),
            order: 2,
            description: "Safely shutdown all trading services".to_string(),
            expected_duration_minutes: 2,
            dependencies: vec!["step-1".to_string()],
        },
        RecoveryStep {
            id: "step-3".to_string(),
            order: 3,
            description: "Restore system from latest verified backup".to_string(),
            expected_duration_minutes: 30,
            dependencies: vec!["step-2".to_string()],
        },
        RecoveryStep {
            id: "step-4".to_string(),
            order: 4,
            description: "Restart all services in correct order".to_string(),
            expected_duration_minutes: 5,
            dependencies: vec!["step-3".to_string()],
        },
        RecoveryStep {
            id: "step-5".to_string(),
            order: 5,
            description: "Validate system functionality and data integrity".to_string(),
            expected_duration_minutes: 10,
            dependencies: vec!["step-4".to_string()],
        }
    ];
    
    let dr_plan = dr_manager.create_plan(
        "Full System Recovery",
        "Complete disaster recovery plan for full system failure",
        recovery_steps,
        "dr-tenant-1",
    );
    
    assert_eq!(dr_plan.name, "Full System Recovery");
    assert_eq!(dr_plan.tenant_id, "dr-tenant-1");
    assert_eq!(dr_plan.steps.len(), 5);
    
    // Test retrieving plan
    let retrieved_plan = dr_manager.get_plan(&dr_plan.id);
    assert!(retrieved_plan.is_some());
    assert_eq!(retrieved_plan.unwrap().id, dr_plan.id);
    
    // Test listing tenant plans
    let tenant_plans = dr_manager.list_tenant_plans("dr-tenant-1");
    assert_eq!(tenant_plans.len(), 1);
    assert_eq!(tenant_plans[0].id, dr_plan.id);
    
    // Test plan execution
    dr_manager.execute_plan(&dr_plan.id)
        .expect("Failed to execute disaster recovery plan");
}

#[test]
fn test_multi_tenant_compliance_isolation() {
    let mut compliance_manager = ComplianceManager::new();
    let mut backup_manager = BackupManager::new();
    let mut dr_manager = DisasterRecoveryManager::new();
    
    // Create resources for tenant 1
    let tenant1_report = compliance_manager.generate_report(
        ReportType::RegulatoryCompliance,
        Utc::now() - chrono::Duration::days(30),
        Utc::now(),
        "tenant1_officer",
        "compliance-tenant-1",
    ).expect("Failed to generate tenant1 report");
    
    let tenant1_backup = backup_manager.create_backup(
        vec!["compliance_data".to_string()],
        "compliance-tenant-1",
    ).expect("Failed to create tenant1 backup");
    
    let tenant1_plan = dr_manager.create_plan(
        "Tenant 1 DR Plan",
        "DR plan for tenant 1",
        vec![],
        "compliance-tenant-1",
    );
    
    // Create resources for tenant 2
    let tenant2_report = compliance_manager.generate_report(
        ReportType::FinancialSummary,
        Utc::now() - chrono::Duration::days(90),
        Utc::now(),
        "tenant2_officer",
        "compliance-tenant-2",
    ).expect("Failed to generate tenant2 report");
    
    let tenant2_backup = backup_manager.create_backup(
        vec!["financial_data".to_string()],
        "compliance-tenant-2",
    ).expect("Failed to create tenant2 backup");
    
    let tenant2_plan = dr_manager.create_plan(
        "Tenant 2 DR Plan",
        "DR plan for tenant 2",
        vec![],
        "compliance-tenant-2",
    );
    
    // Verify tenant isolation for reports
    let tenant1_reports = compliance_manager.get_tenant_reports("compliance-tenant-1");
    let tenant2_reports = compliance_manager.get_tenant_reports("compliance-tenant-2");
    
    assert_eq!(tenant1_reports.len(), 1);
    assert_eq!(tenant2_reports.len(), 1);
    assert_eq!(tenant1_reports[0].id, tenant1_report.id);
    assert_eq!(tenant2_reports[0].id, tenant2_report.id);
    
    // Verify tenant isolation for backups
    let tenant1_backups = backup_manager.list_tenant_backups("compliance-tenant-1");
    let tenant2_backups = backup_manager.list_tenant_backups("compliance-tenant-2");
    
    assert_eq!(tenant1_backups.len(), 1);
    assert_eq!(tenant2_backups.len(), 1);
    assert_eq!(tenant1_backups[0].id, tenant1_backup.id);
    assert_eq!(tenant2_backups[0].id, tenant2_backup.id);
    
    // Verify tenant isolation for DR plans
    let tenant1_plans = dr_manager.list_tenant_plans("compliance-tenant-1");
    let tenant2_plans = dr_manager.list_tenant_plans("compliance-tenant-2");
    
    assert_eq!(tenant1_plans.len(), 1);
    assert_eq!(tenant2_plans.len(), 1);
    assert_eq!(tenant1_plans[0].id, tenant1_plan.id);
    assert_eq!(tenant2_plans[0].id, tenant2_plan.id);
}