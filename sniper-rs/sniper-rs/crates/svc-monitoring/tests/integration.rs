//! Integration tests for the monitoring service and enterprise features

use sniper_monitoring::{
    MonitoringSystem,
    DashboardPanel,
    IncidentSeverity,
    IncidentStatus
};

#[test]
fn test_monitoring_dashboards_enterprise_features() {
    let mut monitoring_system = MonitoringSystem::new()
        .expect("Failed to create monitoring system");
    
    // Test creating a comprehensive dashboard with multiple panels
    let panels = vec![
        DashboardPanel {
            id: "cpu-panel".to_string(),
            title: "CPU Usage".to_string(),
            description: "Real-time CPU utilization metrics".to_string(),
            metric_name: "cpu_usage_percent".to_string(),
            panel_type: "graph".to_string(),
            query: "avg(rate(cpu_usage_percent[5m])) by (instance)".to_string(),
        },
        DashboardPanel {
            id: "memory-panel".to_string(),
            title: "Memory Usage".to_string(),
            description: "Memory consumption across services".to_string(),
            metric_name: "memory_usage_bytes".to_string(),
            panel_type: "graph".to_string(),
            query: "avg(memory_usage_bytes) by (service)".to_string(),
        },
        DashboardPanel {
            id: "latency-panel".to_string(),
            title: "Request Latency".to_string(),
            description: "API response time metrics".to_string(),
            metric_name: "request_latency_ms".to_string(),
            panel_type: "graph".to_string(),
            query: "histogram_quantile(0.95, sum(rate(request_latency_ms_bucket[5m])) by (le))".to_string(),
        },
        DashboardPanel {
            id: "error-panel".to_string(),
            title: "Error Rate".to_string(),
            description: "Application error rate monitoring".to_string(),
            metric_name: "error_count".to_string(),
            panel_type: "stat".to_string(),
            query: "sum(rate(error_count[5m]))".to_string(),
        }
    ];
    
    let dashboard = monitoring_system.dashboard_manager().create_dashboard(
        "System Performance Overview",
        "Comprehensive dashboard for monitoring system performance",
        panels,
        "monitoring-tenant-1",
    );
    
    assert_eq!(dashboard.name, "System Performance Overview");
    assert_eq!(dashboard.tenant_id, "monitoring-tenant-1");
    assert_eq!(dashboard.panels.len(), 4);
    
    // Test retrieving dashboard
    let retrieved_dashboard = monitoring_system.dashboard_manager_ref().get_dashboard(&dashboard.id);
    assert!(retrieved_dashboard.is_some());
    assert_eq!(retrieved_dashboard.unwrap().id, dashboard.id);
    
    // Test panel content
    let first_panel = &dashboard.panels[0];
    assert_eq!(first_panel.id, "cpu-panel");
    assert_eq!(first_panel.title, "CPU Usage");
    assert_eq!(first_panel.metric_name, "cpu_usage_percent");
}

#[test]
fn test_incident_management_enterprise_features() {
    let mut monitoring_system = MonitoringSystem::new()
        .expect("Failed to create monitoring system");
    
    // Test creating incidents with different severity levels
    let critical_incident = monitoring_system.incident_manager().create_incident(
        "Database Connection Failure",
        "Critical database connection failure affecting all services",
        IncidentSeverity::Critical,
        "monitoring-tenant-1",
    );
    
    let high_incident = monitoring_system.incident_manager().create_incident(
        "High CPU Usage",
        "CPU usage exceeded 95% threshold on trading services",
        IncidentSeverity::High,
        "monitoring-tenant-1",
    );
    
    let medium_incident = monitoring_system.incident_manager().create_incident(
        "Slow API Response",
        "API response time degradation noticed in portfolio service",
        IncidentSeverity::Medium,
        "monitoring-tenant-1",
    );
    
    let low_incident = monitoring_system.incident_manager().create_incident(
        "Disk Space Warning",
        "Disk space usage approaching 80% threshold",
        IncidentSeverity::Low,
        "monitoring-tenant-1",
    );
    
    // Verify incident properties
    assert_eq!(critical_incident.title, "Database Connection Failure");
    assert_eq!(critical_incident.severity, IncidentSeverity::Critical);
    assert_eq!(critical_incident.status, IncidentStatus::Open);
    assert_eq!(critical_incident.tenant_id, "monitoring-tenant-1");
    
    assert_eq!(high_incident.severity, IncidentSeverity::High);
    assert_eq!(medium_incident.severity, IncidentSeverity::Medium);
    assert_eq!(low_incident.severity, IncidentSeverity::Low);
    
    // Test retrieving incidents
    let retrieved_critical = monitoring_system.incident_manager_ref().get_incident(&critical_incident.id);
    let retrieved_high = monitoring_system.incident_manager_ref().get_incident(&high_incident.id);
    
    assert!(retrieved_critical.is_some());
    assert!(retrieved_high.is_some());
    assert_eq!(retrieved_critical.unwrap().id, critical_incident.id);
    assert_eq!(retrieved_high.unwrap().id, high_incident.id);
    
    // Test updating incident status
    monitoring_system.incident_manager().update_incident_status(
        &critical_incident.id,
        IncidentStatus::InProgress,
        Some("Investigating database connection issues".to_string()),
    ).expect("Failed to update incident status");
    
    let updated_incident = monitoring_system.incident_manager_ref().get_incident(&critical_incident.id).unwrap();
    assert_eq!(updated_incident.status, IncidentStatus::InProgress);
    assert_eq!(updated_incident.resolution_notes, Some("Investigating database connection issues".to_string()));
    
    // Test assigning incident to user
    monitoring_system.incident_manager().assign_incident(
        &high_incident.id,
        "ops-engineer-123",
    ).expect("Failed to assign incident");
    
    let assigned_incident = monitoring_system.incident_manager_ref().get_incident(&high_incident.id).unwrap();
    assert_eq!(assigned_incident.assigned_to, Some("ops-engineer-123".to_string()));
}

#[test]
fn test_alert_rules_enterprise_features() {
    let mut monitoring_system = MonitoringSystem::new()
        .expect("Failed to create monitoring system");
    
    // Test creating alert rules with different configurations
    let cpu_alert = monitoring_system.incident_manager().create_alert_rule(
        "High CPU Usage Alert",
        "Trigger alert when CPU usage exceeds 90%",
        "avg(cpu_usage_percent) > 90",
        90.0,
        IncidentSeverity::High,
        "monitoring-tenant-1",
    );
    
    let memory_alert = monitoring_system.incident_manager().create_alert_rule(
        "Low Memory Alert",
        "Trigger alert when available memory drops below 10%",
        "memory_available_percent < 10",
        10.0,
        IncidentSeverity::Medium,
        "monitoring-tenant-1",
    );
    
    let latency_alert = monitoring_system.incident_manager().create_alert_rule(
        "High Latency Alert",
        "Trigger alert when 95th percentile latency exceeds 200ms",
        "histogram_quantile(0.95, request_latency_ms_bucket) > 200",
        200.0,
        IncidentSeverity::Low,
        "monitoring-tenant-1",
    );
    
    // Verify alert rule properties
    assert_eq!(cpu_alert.name, "High CPU Usage Alert");
    assert_eq!(cpu_alert.threshold, 90.0);
    assert_eq!(cpu_alert.severity, IncidentSeverity::High);
    assert_eq!(cpu_alert.tenant_id, "monitoring-tenant-1");
    assert!(cpu_alert.enabled);
    
    assert_eq!(memory_alert.threshold, 10.0);
    assert_eq!(memory_alert.severity, IncidentSeverity::Medium);
    
    assert_eq!(latency_alert.threshold, 200.0);
    assert_eq!(latency_alert.severity, IncidentSeverity::Low);
    
    // Test alert evaluation (mock implementation)
    let _incidents = monitoring_system.incident_manager().evaluate_alerts()
        .expect("Failed to evaluate alerts");
    // In a real implementation, this might create incidents based on metric values
    // For now, we're just testing that it doesn't error
}

#[test]
fn test_metrics_collection_enterprise_features() {
    let monitoring_system = MonitoringSystem::new()
        .expect("Failed to create monitoring system");
    
    // Test metrics registry functionality
    let registry = monitoring_system.metrics_registry();
    {
        let mut registry = registry.lock().unwrap();
        
        // Test counter metrics with unique names
        registry.register_counter("test_http_requests_total", "Total HTTP requests for testing")
            .expect("Failed to register counter");
        registry.increment_counter("test_http_requests_total")
            .expect("Failed to increment counter");
        registry.increment_counter("test_http_requests_total")
            .expect("Failed to increment counter");
        
        // Test gauge metrics with unique names
        registry.register_gauge("test_active_users", "Number of active users for testing")
            .expect("Failed to register gauge");
        registry.set_gauge("test_active_users", 42.0)
            .expect("Failed to set gauge");
        
        // Test histogram metrics with unique names
        registry.register_histogram("test_request_duration_seconds", "HTTP request duration for testing")
            .expect("Failed to register histogram");
        registry.observe_histogram("test_request_duration_seconds", 0.05)
            .expect("Failed to observe histogram");
        registry.observe_histogram("test_request_duration_seconds", 0.1)
            .expect("Failed to observe histogram");
        registry.observe_histogram("test_request_duration_seconds", 0.2)
            .expect("Failed to observe histogram");
    }
    
    // Test metrics text output
    let metrics_text = monitoring_system.get_metrics_text()
        .expect("Failed to get metrics text");
    assert!(metrics_text.contains("test_http_requests_total"));
    assert!(metrics_text.contains("test_active_users"));
    assert!(metrics_text.contains("test_request_duration_seconds"));
    
    // Verify metric values
    assert!(metrics_text.contains("test_http_requests_total 2"));
    assert!(metrics_text.contains("test_active_users 42"));
}

#[test]
fn test_multi_tenant_monitoring_isolation() {
    let mut monitoring_system = MonitoringSystem::new()
        .expect("Failed to create monitoring system");
    
    // Create resources for tenant 1
    let tenant1_panels = vec![
        DashboardPanel {
            id: "tenant1-panel".to_string(),
            title: "Tenant 1 Metrics".to_string(),
            description: "Metrics for tenant 1".to_string(),
            metric_name: "tenant1_metric".to_string(),
            panel_type: "graph".to_string(),
            query: "tenant1_query".to_string(),
        }
    ];
    
    let tenant1_dashboard = monitoring_system.dashboard_manager().create_dashboard(
        "Tenant 1 Dashboard",
        "Dashboard for tenant 1",
        tenant1_panels,
        "monitoring-tenant-1",
    );
    
    let tenant1_incident = monitoring_system.incident_manager().create_incident(
        "Tenant 1 Issue",
        "Issue specific to tenant 1",
        IncidentSeverity::Medium,
        "monitoring-tenant-1",
    );
    
    // Create resources for tenant 2
    let tenant2_panels = vec![
        DashboardPanel {
            id: "tenant2-panel".to_string(),
            title: "Tenant 2 Metrics".to_string(),
            description: "Metrics for tenant 2".to_string(),
            metric_name: "tenant2_metric".to_string(),
            panel_type: "graph".to_string(),
            query: "tenant2_query".to_string(),
        }
    ];
    
    let tenant2_dashboard = monitoring_system.dashboard_manager().create_dashboard(
        "Tenant 2 Dashboard",
        "Dashboard for tenant 2",
        tenant2_panels,
        "monitoring-tenant-2",
    );
    
    let tenant2_incident = monitoring_system.incident_manager().create_incident(
        "Tenant 2 Issue",
        "Issue specific to tenant 2",
        IncidentSeverity::High,
        "monitoring-tenant-2",
    );
    
    // Verify tenant isolation for dashboards
    let tenant1_dashboards = monitoring_system.dashboard_manager_ref().list_tenant_dashboards("monitoring-tenant-1");
    let tenant2_dashboards = monitoring_system.dashboard_manager_ref().list_tenant_dashboards("monitoring-tenant-2");
    
    assert_eq!(tenant1_dashboards.len(), 1);
    assert_eq!(tenant2_dashboards.len(), 1);
    assert_eq!(tenant1_dashboards[0].id, tenant1_dashboard.id);
    assert_eq!(tenant2_dashboards[0].id, tenant2_dashboard.id);
    
    // Verify tenant isolation for incidents
    let tenant1_incidents = monitoring_system.incident_manager_ref().list_tenant_incidents("monitoring-tenant-1");
    let tenant2_incidents = monitoring_system.incident_manager_ref().list_tenant_incidents("monitoring-tenant-2");
    
    assert_eq!(tenant1_incidents.len(), 1);
    assert_eq!(tenant2_incidents.len(), 1);
    assert_eq!(tenant1_incidents[0].id, tenant1_incident.id);
    assert_eq!(tenant2_incidents[0].id, tenant2_incident.id);
}