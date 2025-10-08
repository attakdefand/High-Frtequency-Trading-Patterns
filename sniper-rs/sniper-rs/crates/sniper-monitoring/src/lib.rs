//! Advanced monitoring system for the sniper-rs enterprise features.
//! 
//! This module provides functionality for advanced monitoring dashboards,
//! automated incident response, and comprehensive system metrics.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use chrono::{DateTime, Utc};
use prometheus::{Counter, Gauge, Histogram, HistogramOpts, Registry, TextEncoder, Encoder};

/// System metric types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

/// System metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetric {
    pub name: String,
    pub metric_type: MetricType,
    pub value: f64,
    pub labels: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

/// Dashboard panel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardPanel {
    pub id: String,
    pub title: String,
    pub description: String,
    pub metric_name: String,
    pub panel_type: String, // "graph", "table", "singlestat", etc.
    pub query: String,
}

/// Monitoring dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringDashboard {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub panels: Vec<DashboardPanel>,
    pub tenant_id: String,
}

/// Incident severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IncidentSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Incident status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IncidentStatus {
    Open,
    InProgress,
    Resolved,
    Closed,
}

/// Incident report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: IncidentSeverity,
    pub status: IncidentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub assigned_to: Option<String>,
    pub resolution_notes: Option<String>,
    pub tenant_id: String,
}

/// Alert rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub query: String,
    pub threshold: f64,
    pub severity: IncidentSeverity,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub tenant_id: String,
}

/// Metrics registry wrapper
pub struct MetricsRegistry {
    registry: Registry,
    counters: HashMap<String, Counter>,
    gauges: HashMap<String, Gauge>,
    histograms: HashMap<String, Histogram>,
}

impl MetricsRegistry {
    /// Create a new metrics registry
    pub fn new() -> Self {
        Self {
            registry: Registry::new(),
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
        }
    }
    
    /// Register a counter metric
    pub fn register_counter(&mut self, name: &str, help: &str) -> Result<()> {
        let counter = Counter::new(name, help)?;
        self.registry.register(Box::new(counter.clone()))?;
        self.counters.insert(name.to_string(), counter);
        Ok(())
    }
    
    /// Register a gauge metric
    pub fn register_gauge(&mut self, name: &str, help: &str) -> Result<()> {
        let gauge = Gauge::new(name, help)?;
        self.registry.register(Box::new(gauge.clone()))?;
        self.gauges.insert(name.to_string(), gauge);
        Ok(())
    }
    
    /// Register a histogram metric
    pub fn register_histogram(&mut self, name: &str, help: &str) -> Result<()> {
        let opts = HistogramOpts::new(name, help);
        let histogram = Histogram::with_opts(opts)?;
        self.registry.register(Box::new(histogram.clone()))?;
        self.histograms.insert(name.to_string(), histogram);
        Ok(())
    }
    
    /// Increment a counter
    pub fn increment_counter(&self, name: &str) -> Result<()> {
        if let Some(counter) = self.counters.get(name) {
            counter.inc();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Counter not found: {}", name))
        }
    }
    
    /// Set a gauge value
    pub fn set_gauge(&self, name: &str, value: f64) -> Result<()> {
        if let Some(gauge) = self.gauges.get(name) {
            gauge.set(value);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Gauge not found: {}", name))
        }
    }
    
    /// Observe a histogram value
    pub fn observe_histogram(&self, name: &str, value: f64) -> Result<()> {
        if let Some(histogram) = self.histograms.get(name) {
            histogram.observe(value);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Histogram not found: {}", name))
        }
    }
    
    /// Get metrics in Prometheus text format
    pub fn get_metrics_text(&self) -> Result<String> {
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }
}

/// Dashboard manager for advanced monitoring
pub struct DashboardManager {
    dashboards: HashMap<String, MonitoringDashboard>,
}

impl DashboardManager {
    /// Create a new dashboard manager
    pub fn new() -> Self {
        Self {
            dashboards: HashMap::new(),
        }
    }
    
    /// Create a monitoring dashboard
    pub fn create_dashboard(
        &mut self,
        name: &str,
        description: &str,
        panels: Vec<DashboardPanel>,
        tenant_id: &str,
    ) -> MonitoringDashboard {
        let dashboard = MonitoringDashboard {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: description.to_string(),
            created_at: Utc::now(),
            panels,
            tenant_id: tenant_id.to_string(),
        };
        
        self.dashboards.insert(dashboard.id.clone(), dashboard.clone());
        dashboard
    }
    
    /// Get a dashboard by ID
    pub fn get_dashboard(&self, dashboard_id: &str) -> Option<&MonitoringDashboard> {
        self.dashboards.get(dashboard_id)
    }
    
    /// List dashboards for a tenant
    pub fn list_tenant_dashboards(&self, tenant_id: &str) -> Vec<&MonitoringDashboard> {
        self.dashboards
            .values()
            .filter(|dashboard| dashboard.tenant_id == tenant_id)
            .collect()
    }
    
    /// Add a panel to a dashboard
    pub fn add_panel(&mut self, dashboard_id: &str, panel: DashboardPanel) -> Result<()> {
        if let Some(dashboard) = self.dashboards.get_mut(dashboard_id) {
            dashboard.panels.push(panel);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Dashboard not found"))
        }
    }
    
    /// Remove a panel from a dashboard
    pub fn remove_panel(&mut self, dashboard_id: &str, panel_id: &str) -> Result<()> {
        if let Some(dashboard) = self.dashboards.get_mut(dashboard_id) {
            dashboard.panels.retain(|panel| panel.id != panel_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Dashboard not found"))
        }
    }
}

/// Incident manager for automated incident response
pub struct IncidentManager {
    incidents: HashMap<String, Incident>,
    alert_rules: HashMap<String, AlertRule>,
}

impl IncidentManager {
    /// Create a new incident manager
    pub fn new() -> Self {
        Self {
            incidents: HashMap::new(),
            alert_rules: HashMap::new(),
        }
    }
    
    /// Create an incident
    pub fn create_incident(
        &mut self,
        title: &str,
        description: &str,
        severity: IncidentSeverity,
        tenant_id: &str,
    ) -> Incident {
        let incident = Incident {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            description: description.to_string(),
            severity,
            status: IncidentStatus::Open,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            assigned_to: None,
            resolution_notes: None,
            tenant_id: tenant_id.to_string(),
        };
        
        self.incidents.insert(incident.id.clone(), incident.clone());
        incident
    }
    
    /// Get an incident by ID
    pub fn get_incident(&self, incident_id: &str) -> Option<&Incident> {
        self.incidents.get(incident_id)
    }
    
    /// List incidents for a tenant
    pub fn list_tenant_incidents(&self, tenant_id: &str) -> Vec<&Incident> {
        self.incidents
            .values()
            .filter(|incident| incident.tenant_id == tenant_id)
            .collect()
    }
    
    /// Update incident status
    pub fn update_incident_status(
        &mut self,
        incident_id: &str,
        status: IncidentStatus,
        resolution_notes: Option<String>,
    ) -> Result<()> {
        if let Some(incident) = self.incidents.get_mut(incident_id) {
            incident.status = status;
            incident.updated_at = Utc::now();
            incident.resolution_notes = resolution_notes;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Incident not found"))
        }
    }
    
    /// Assign incident to a user
    pub fn assign_incident(&mut self, incident_id: &str, user_id: &str) -> Result<()> {
        if let Some(incident) = self.incidents.get_mut(incident_id) {
            incident.assigned_to = Some(user_id.to_string());
            incident.updated_at = Utc::now();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Incident not found"))
        }
    }
    
    /// Create an alert rule
    pub fn create_alert_rule(
        &mut self,
        name: &str,
        description: &str,
        query: &str,
        threshold: f64,
        severity: IncidentSeverity,
        tenant_id: &str,
    ) -> AlertRule {
        let rule = AlertRule {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: description.to_string(),
            query: query.to_string(),
            threshold,
            severity,
            enabled: true,
            created_at: Utc::now(),
            tenant_id: tenant_id.to_string(),
        };
        
        self.alert_rules.insert(rule.id.clone(), rule.clone());
        rule
    }
    
    /// Evaluate alert rules and create incidents when thresholds are exceeded
    pub fn evaluate_alerts(&mut self) -> Result<Vec<Incident>> {
        let mut new_incidents = Vec::new();
        
        // Create a copy of alert rules to avoid borrowing conflicts
        let rule_ids: Vec<String> = self.alert_rules.keys().cloned().collect();
        
        // In a real implementation, this would evaluate actual metrics against alert rules
        // For now, we'll simulate this with a simple check
        for rule_id in rule_ids {
            if let Some(rule) = self.alert_rules.get(&rule_id).cloned() {
                if rule.enabled {
                    // Simulate metric evaluation
                    let metric_value = 0.0; // Placeholder value
                    
                    if metric_value > rule.threshold {
                        let incident = self.create_incident(
                            &format!("Alert: {}", rule.name),
                            &format!("Alert rule '{}' triggered. Metric value {} exceeded threshold {}", 
                                    rule.name, metric_value, rule.threshold),
                            rule.severity.clone(),
                            &rule.tenant_id,
                        );
                        
                        new_incidents.push(incident);
                    }
                }
            }
        }
        
        Ok(new_incidents)
    }
}

/// Main monitoring system
pub struct MonitoringSystem {
    metrics_registry: Arc<Mutex<MetricsRegistry>>,
    dashboard_manager: DashboardManager,
    incident_manager: IncidentManager,
}

impl MonitoringSystem {
    /// Create a new monitoring system
    pub fn new() -> Result<Self> {
        let mut metrics_registry = MetricsRegistry::new();
        
        // Register default metrics
        metrics_registry.register_counter("http_requests_total", "Total HTTP requests")?;
        metrics_registry.register_gauge("active_users", "Number of active users")?;
        metrics_registry.register_histogram("request_duration_seconds", "HTTP request duration")?;
        
        Ok(Self {
            metrics_registry: Arc::new(Mutex::new(metrics_registry)),
            dashboard_manager: DashboardManager::new(),
            incident_manager: IncidentManager::new(),
        })
    }
    
    /// Get metrics registry
    pub fn metrics_registry(&self) -> Arc<Mutex<MetricsRegistry>> {
        self.metrics_registry.clone()
    }
    
    /// Get dashboard manager (mutable access)
    pub fn dashboard_manager(&mut self) -> &mut DashboardManager {
        &mut self.dashboard_manager
    }
    
    /// Get dashboard manager (immutable access)
    pub fn dashboard_manager_ref(&self) -> &DashboardManager {
        &self.dashboard_manager
    }
    
    /// Get incident manager (mutable access)
    pub fn incident_manager(&mut self) -> &mut IncidentManager {
        &mut self.incident_manager
    }
    
    /// Get incident manager (immutable access)
    pub fn incident_manager_ref(&self) -> &IncidentManager {
        &self.incident_manager
    }
    
    /// Get metrics in Prometheus text format
    pub fn get_metrics_text(&self) -> Result<String> {
        let registry = self.metrics_registry.lock().unwrap();
        registry.get_metrics_text()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_registry() {
        let mut registry = MetricsRegistry::new();
        
        // Test counter registration and increment
        registry.register_counter("test_counter", "A test counter").unwrap();
        registry.increment_counter("test_counter").unwrap();
        registry.increment_counter("test_counter").unwrap();
        
        // Test gauge registration and setting
        registry.register_gauge("test_gauge", "A test gauge").unwrap();
        registry.set_gauge("test_gauge", 42.0).unwrap();
        
        // Test histogram registration and observation
        registry.register_histogram("test_histogram", "A test histogram").unwrap();
        registry.observe_histogram("test_histogram", 1.0).unwrap();
        registry.observe_histogram("test_histogram", 2.0).unwrap();
        
        // Test metrics text output
        let metrics_text = registry.get_metrics_text().unwrap();
        assert!(metrics_text.contains("test_counter"));
        assert!(metrics_text.contains("test_gauge"));
        assert!(metrics_text.contains("test_histogram"));
    }

    #[test]
    fn test_dashboard_management() {
        let mut dashboard_manager = DashboardManager::new();
        let panels = vec![
            DashboardPanel {
                id: "panel-1".to_string(),
                title: "Test Panel".to_string(),
                description: "A test panel".to_string(),
                metric_name: "test_metric".to_string(),
                panel_type: "graph".to_string(),
                query: "test_query".to_string(),
            }
        ];
        
        let dashboard = dashboard_manager.create_dashboard(
            "Test Dashboard",
            "A test dashboard",
            panels,
            "tenant-1",
        );
        
        assert_eq!(dashboard.name, "Test Dashboard");
        assert_eq!(dashboard.tenant_id, "tenant-1");
        assert_eq!(dashboard.panels.len(), 1);
        
        let retrieved_dashboard = dashboard_manager.get_dashboard(&dashboard.id);
        assert!(retrieved_dashboard.is_some());
        assert_eq!(retrieved_dashboard.unwrap().id, dashboard.id);
    }

    #[test]
    fn test_incident_management() {
        let mut incident_manager = IncidentManager::new();
        
        let incident = incident_manager.create_incident(
            "Test Incident",
            "A test incident",
            IncidentSeverity::High,
            "tenant-1",
        );
        
        assert_eq!(incident.title, "Test Incident");
        assert_eq!(incident.severity, IncidentSeverity::High);
        assert_eq!(incident.status, IncidentStatus::Open);
        assert_eq!(incident.tenant_id, "tenant-1");
        
        let retrieved_incident = incident_manager.get_incident(&incident.id);
        assert!(retrieved_incident.is_some());
        assert_eq!(retrieved_incident.unwrap().id, incident.id);
        
        // Test updating incident status
        incident_manager.update_incident_status(
            &incident.id,
            IncidentStatus::Resolved,
            Some("Issue fixed".to_string()),
        ).unwrap();
        
        let updated_incident = incident_manager.get_incident(&incident.id).unwrap();
        assert_eq!(updated_incident.status, IncidentStatus::Resolved);
        assert_eq!(updated_incident.resolution_notes, Some("Issue fixed".to_string()));
    }

    #[test]
    fn test_alert_rules() {
        let mut incident_manager = IncidentManager::new();
        
        let rule = incident_manager.create_alert_rule(
            "High CPU Usage",
            "Alert when CPU usage exceeds 80%",
            "cpu_usage > 80",
            80.0,
            IncidentSeverity::High,
            "tenant-1",
        );
        
        assert_eq!(rule.name, "High CPU Usage");
        assert_eq!(rule.threshold, 80.0);
        assert_eq!(rule.tenant_id, "tenant-1");
        assert!(rule.enabled);
        
        let rules = &incident_manager.alert_rules;
        assert!(rules.contains_key(&rule.id));
    }

    #[test]
    fn test_tenant_isolation() {
        let mut dashboard_manager = DashboardManager::new();
        let mut incident_manager = IncidentManager::new();
        
        // Create resources for tenant-1
        let panels1 = vec![
            DashboardPanel {
                id: "panel-1".to_string(),
                title: "Tenant 1 Panel".to_string(),
                description: "Panel for tenant 1".to_string(),
                metric_name: "metric1".to_string(),
                panel_type: "graph".to_string(),
                query: "query1".to_string(),
            }
        ];
        
        let dashboard1 = dashboard_manager.create_dashboard(
            "Tenant 1 Dashboard",
            "Dashboard for tenant 1",
            panels1,
            "tenant-1",
        );
        
        let incident1 = incident_manager.create_incident(
            "Tenant 1 Incident",
            "Incident for tenant 1",
            IncidentSeverity::Medium,
            "tenant-1",
        );
        
        // Create resources for tenant-2
        let panels2 = vec![
            DashboardPanel {
                id: "panel-2".to_string(),
                title: "Tenant 2 Panel".to_string(),
                description: "Panel for tenant 2".to_string(),
                metric_name: "metric2".to_string(),
                panel_type: "graph".to_string(),
                query: "query2".to_string(),
            }
        ];
        
        let dashboard2 = dashboard_manager.create_dashboard(
            "Tenant 2 Dashboard",
            "Dashboard for tenant 2",
            panels2,
            "tenant-2",
        );
        
        let incident2 = incident_manager.create_incident(
            "Tenant 2 Incident",
            "Incident for tenant 2",
            IncidentSeverity::Low,
            "tenant-2",
        );
        
        // Verify tenant isolation
        let tenant1_dashboards = dashboard_manager.list_tenant_dashboards("tenant-1");
        let tenant2_dashboards = dashboard_manager.list_tenant_dashboards("tenant-2");
        assert_eq!(tenant1_dashboards.len(), 1);
        assert_eq!(tenant2_dashboards.len(), 1);
        assert_ne!(tenant1_dashboards[0].id, tenant2_dashboards[0].id);
        
        let tenant1_incidents = incident_manager.list_tenant_incidents("tenant-1");
        let tenant2_incidents = incident_manager.list_tenant_incidents("tenant-2");
        assert_eq!(tenant1_incidents.len(), 1);
        assert_eq!(tenant2_incidents.len(), 1);
        assert_ne!(tenant1_incidents[0].id, tenant2_incidents[0].id);
    }
}