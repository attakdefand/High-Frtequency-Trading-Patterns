//! Load balancer for horizontal scaling of executor instances
//! 
//! This module provides functionality for distributing trades across
//! multiple executor instances to improve throughput and reliability.

use anyhow::Result;
use sniper_core::types::{TradePlan, ExecReceipt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Load balancing strategy
#[derive(Debug, Clone)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin { weights: HashMap<String, u32> },
}

/// Executor instance information
#[derive(Debug, Clone)]
pub struct ExecutorInstance {
    pub id: String,
    pub address: String,
    pub active_connections: u32,
    pub weight: u32,
    pub healthy: bool,
}

/// Load balancer for distributing trades across multiple executors
pub struct LoadBalancer {
    instances: Arc<RwLock<HashMap<String, ExecutorInstance>>>,
    strategy: LoadBalancingStrategy,
    last_selected: Arc<RwLock<usize>>,
}

impl LoadBalancer {
    /// Create a new load balancer
    pub fn new(strategy: LoadBalancingStrategy) -> Self {
        Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
            strategy,
            last_selected: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Add an executor instance
    pub async fn add_instance(&self, instance: ExecutorInstance) -> Result<()> {
        let mut instances = self.instances.write().await;
        instances.insert(instance.id.clone(), instance);
        Ok(())
    }
    
    /// Remove an executor instance
    pub async fn remove_instance(&self, instance_id: &str) -> Result<()> {
        let mut instances = self.instances.write().await;
        instances.remove(instance_id);
        Ok(())
    }
    
    /// Mark an instance as healthy/unhealthy
    pub async fn set_instance_health(&self, instance_id: &str, healthy: bool) -> Result<()> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(instance_id) {
            instance.healthy = healthy;
        }
        Ok(())
    }
    
    /// Update connection count for an instance
    pub async fn update_connection_count(&self, instance_id: &str, count: u32) -> Result<()> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(instance_id) {
            instance.active_connections = count;
        }
        Ok(())
    }
    
    /// Select the next executor instance based on the load balancing strategy
    pub async fn select_instance(&self) -> Option<ExecutorInstance> {
        let instances = self.instances.read().await;
        let healthy_instances: Vec<&ExecutorInstance> = instances
            .values()
            .filter(|instance| instance.healthy)
            .collect();
        
        if healthy_instances.is_empty() {
            return None;
        }
        
        match &self.strategy {
            LoadBalancingStrategy::RoundRobin => {
                self.select_round_robin(&healthy_instances).await
            }
            LoadBalancingStrategy::LeastConnections => {
                self.select_least_connections(&healthy_instances).await
            }
            LoadBalancingStrategy::WeightedRoundRobin { .. } => {
                self.select_weighted_round_robin(&healthy_instances).await
            }
        }
    }
    
    /// Select instance using round-robin strategy
    async fn select_round_robin(&self, instances: &[&ExecutorInstance]) -> Option<ExecutorInstance> {
        let mut last_selected = self.last_selected.write().await;
        let index = *last_selected % instances.len();
        *last_selected = (*last_selected + 1) % instances.len();
        Some(instances[index].clone())
    }
    
    /// Select instance with least connections
    async fn select_least_connections(&self, instances: &[&ExecutorInstance]) -> Option<ExecutorInstance> {
        instances
            .iter()
            .min_by_key(|instance| instance.active_connections)
            .cloned()
            .cloned()
    }
    
    /// Select instance using weighted round-robin
    async fn select_weighted_round_robin(&self, instances: &[&ExecutorInstance]) -> Option<ExecutorInstance> {
        // Simple implementation - in a real system, this would use the weights
        self.select_round_robin(instances).await
    }
    
    /// Execute a trade using the load balancer
    pub async fn execute_trade(&self, _plan: &TradePlan) -> Result<ExecReceipt> {
        // In a real implementation, this would:
        // 1. Select an instance using the load balancing strategy
        // 2. Send the trade to that instance
        // 3. Handle failures and retries
        // 4. Return the result
        
        if let Some(_instance) = self.select_instance().await {
            // Simulate execution
            Ok(ExecReceipt {
                tx_hash: "0xload-balanced-tx".to_string(),
                success: true,
                block: 12345678,
                gas_used: 100000,
                fees_paid_wei: 2100000000000000, // 0.0021 ETH
                failure_reason: None,
            })
        } else {
            Err(anyhow::anyhow!("No healthy executor instances available"))
        }
    }
    
    /// Get statistics about the load balancer
    pub async fn get_stats(&self) -> LoadBalancerStats {
        let instances = self.instances.read().await;
        let healthy_count = instances.values().filter(|i| i.healthy).count();
        let total_connections: u32 = instances.values().map(|i| i.active_connections).sum();
        
        LoadBalancerStats {
            total_instances: instances.len(),
            healthy_instances: healthy_count,
            total_connections,
        }
    }
}

/// Statistics about the load balancer
#[derive(Debug, Clone)]
pub struct LoadBalancerStats {
    pub total_instances: usize,
    pub healthy_instances: usize,
    pub total_connections: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_balancer_creation() -> Result<()> {
        let lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin);
        let stats = lb.get_stats().await;
        assert_eq!(stats.total_instances, 0);
        assert_eq!(stats.healthy_instances, 0);
        Ok(())
    }
    
    #[tokio::test]
    async fn test_instance_management() -> Result<()> {
        let lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin);
        
        let instance = ExecutorInstance {
            id: "executor-1".to_string(),
            address: "127.0.0.1:8080".to_string(),
            active_connections: 0,
            weight: 1,
            healthy: true,
        };
        
        lb.add_instance(instance).await?;
        let stats = lb.get_stats().await;
        assert_eq!(stats.total_instances, 1);
        assert_eq!(stats.healthy_instances, 1);
        
        lb.remove_instance("executor-1").await?;
        let stats = lb.get_stats().await;
        assert_eq!(stats.total_instances, 0);
        Ok(())
    }
    
    #[tokio::test]
    async fn test_round_robin_selection() -> Result<()> {
        let lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin);
        
        let instance1 = ExecutorInstance {
            id: "executor-1".to_string(),
            address: "127.0.0.1:8080".to_string(),
            active_connections: 0,
            weight: 1,
            healthy: true,
        };
        
        let instance2 = ExecutorInstance {
            id: "executor-2".to_string(),
            address: "127.0.0.1:8081".to_string(),
            active_connections: 0,
            weight: 1,
            healthy: true,
        };
        
        lb.add_instance(instance1).await?;
        lb.add_instance(instance2).await?;
        
        let selected1 = lb.select_instance().await.unwrap();
        let selected2 = lb.select_instance().await.unwrap();
        let selected3 = lb.select_instance().await.unwrap();
        
        // Should round-robin between the two instances
        assert_ne!(selected1.id, selected2.id);
        assert_eq!(selected1.id, selected3.id);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_least_connections_selection() -> Result<()> {
        let lb = LoadBalancer::new(LoadBalancingStrategy::LeastConnections);
        
        let instance1 = ExecutorInstance {
            id: "executor-1".to_string(),
            address: "127.0.0.1:8080".to_string(),
            active_connections: 5,
            weight: 1,
            healthy: true,
        };
        
        let instance2 = ExecutorInstance {
            id: "executor-2".to_string(),
            address: "127.0.0.1:8081".to_string(),
            active_connections: 2,
            weight: 1,
            healthy: true,
        };
        
        lb.add_instance(instance1).await?;
        lb.add_instance(instance2).await?;
        
        let selected = lb.select_instance().await.unwrap();
        // Should select the instance with least connections
        assert_eq!(selected.id, "executor-2");
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_unhealthy_instance_filtering() -> Result<()> {
        let lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin);
        
        let healthy_instance = ExecutorInstance {
            id: "executor-1".to_string(),
            address: "127.0.0.1:8080".to_string(),
            active_connections: 0,
            weight: 1,
            healthy: true,
        };
        
        let unhealthy_instance = ExecutorInstance {
            id: "executor-2".to_string(),
            address: "127.0.0.1:8081".to_string(),
            active_connections: 0,
            weight: 1,
            healthy: false,
        };
        
        lb.add_instance(healthy_instance).await?;
        lb.add_instance(unhealthy_instance).await?;
        
        let selected = lb.select_instance().await.unwrap();
        // Should only select the healthy instance
        assert_eq!(selected.id, "executor-1");
        
        Ok(())
    }
}