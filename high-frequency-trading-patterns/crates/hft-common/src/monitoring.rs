//! Performance monitoring and metrics collection for HFT patterns
use crate::prelude::*;
use std::sync::atomic::{AtomicU64, AtomicI64, Ordering};
use std::time::Instant;
use tracing::{info, debug};

/// Performance metrics tracker
#[derive(Debug)]
pub struct PerformanceMonitor {
    /// Total quotes processed
    quotes_processed: AtomicU64,
    /// Total orders sent
    orders_sent: AtomicU64,
    /// Total fills received
    fills_received: AtomicU64,
    /// Total PnL (stored as i64 with scaling factor)
    total_pnl: AtomicI64,
    /// Average latency in microseconds (stored as i64 with scaling factor)
    avg_latency_us: AtomicI64,
    /// Maximum latency observed in microseconds
    max_latency_us: AtomicU64,
    /// Start time of monitoring
    start_time: Instant,
    /// Pattern name for identification
    pattern_name: String,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(pattern_name: &str) -> Self {
        Self {
            quotes_processed: AtomicU64::new(0),
            orders_sent: AtomicU64::new(0),
            fills_received: AtomicU64::new(0),
            total_pnl: AtomicI64::new(0),
            avg_latency_us: AtomicI64::new(0),
            max_latency_us: AtomicU64::new(0),
            start_time: Instant::now(),
            pattern_name: pattern_name.to_string(),
        }
    }

    /// Record a quote processing event
    pub fn record_quote(&self) {
        self.quotes_processed.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an order sending event
    pub fn record_order(&self) {
        self.orders_sent.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a fill receiving event
    pub fn record_fill(&self, fill: &Fill) {
        self.fills_received.fetch_add(1, Ordering::Relaxed);
        
        // Update PnL (scale by 10000 to preserve precision)
        let value = fill.qty * fill.px;
        let pnl_change = if fill.side == Side::Buy { -value } else { value };
        let scaled_pnl = (pnl_change * 10000.0) as i64;
        self.total_pnl.fetch_add(scaled_pnl, Ordering::Relaxed);
    }

    /// Record latency measurement
    pub fn record_latency(&self, latency_us: u128) {
        // Update maximum latency
        let current_max = self.max_latency_us.load(Ordering::Relaxed);
        if latency_us > current_max as u128 {
            self.max_latency_us.store(latency_us as u64, Ordering::Relaxed);
        }

        // Update average latency using exponential moving average (scale by 1000)
        let current_avg = self.avg_latency_us.load(Ordering::Relaxed);
        let current_avg_f64 = current_avg as f64 / 1000.0;
        let new_avg_f64 = if current_avg_f64 == 0.0 {
            latency_us as f64
        } else {
            current_avg_f64 * 0.99 + (latency_us as f64) * 0.01
        };
        let scaled_new_avg = (new_avg_f64 * 1000.0) as i64;
        self.avg_latency_us.store(scaled_new_avg, Ordering::Relaxed);
    }

    /// Get current metrics snapshot
    pub fn get_metrics(&self) -> PerformanceMetrics {
        let scaled_pnl = self.total_pnl.load(Ordering::Relaxed) as f64 / 10000.0;
        let scaled_avg_latency = self.avg_latency_us.load(Ordering::Relaxed) as f64 / 1000.0;
        
        PerformanceMetrics {
            quotes_processed: self.quotes_processed.load(Ordering::Relaxed),
            orders_sent: self.orders_sent.load(Ordering::Relaxed),
            fills_received: self.fills_received.load(Ordering::Relaxed),
            total_pnl: scaled_pnl,
            avg_latency_us: scaled_avg_latency,
            max_latency_us: self.max_latency_us.load(Ordering::Relaxed),
            uptime_seconds: self.start_time.elapsed().as_secs_f64(),
            pattern_name: self.pattern_name.clone(),
        }
    }

    /// Log current metrics
    pub fn log_metrics(&self) {
        let metrics = self.get_metrics();
        info!(
            "[{}] Metrics - Quotes: {}, Orders: {}, Fills: {}, PnL: {:.2}, Avg Latency: {:.2}μs, Max Latency: {}μs, Uptime: {:.1}s",
            metrics.pattern_name,
            metrics.quotes_processed,
            metrics.orders_sent,
            metrics.fills_received,
            metrics.total_pnl,
            metrics.avg_latency_us,
            metrics.max_latency_us,
            metrics.uptime_seconds
        );
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.quotes_processed.store(0, Ordering::Relaxed);
        self.orders_sent.store(0, Ordering::Relaxed);
        self.fills_received.store(0, Ordering::Relaxed);
        self.total_pnl.store(0, Ordering::Relaxed);
        self.avg_latency_us.store(0, Ordering::Relaxed);
        self.max_latency_us.store(0, Ordering::Relaxed);
        // Note: We don't reset start_time to maintain uptime tracking
    }
}

/// Snapshot of performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub quotes_processed: u64,
    pub orders_sent: u64,
    pub fills_received: u64,
    pub total_pnl: f64,
    pub avg_latency_us: f64,
    pub max_latency_us: u64,
    pub uptime_seconds: f64,
    pub pattern_name: String,
}

impl PerformanceMetrics {
    /// Calculate orders per second
    pub fn orders_per_second(&self) -> f64 {
        if self.uptime_seconds > 0.0 {
            self.orders_sent as f64 / self.uptime_seconds
        } else {
            0.0
        }
    }

    /// Calculate fills per second
    pub fn fills_per_second(&self) -> f64 {
        if self.uptime_seconds > 0.0 {
            self.fills_received as f64 / self.uptime_seconds
        } else {
            0.0
        }
    }

    /// Calculate quotes per second
    pub fn quotes_per_second(&self) -> f64 {
        if self.uptime_seconds > 0.0 {
            self.quotes_processed as f64 / self.uptime_seconds
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new("test_pattern");
        let metrics = monitor.get_metrics();
        
        assert_eq!(metrics.quotes_processed, 0);
        assert_eq!(metrics.orders_sent, 0);
        assert_eq!(metrics.fills_received, 0);
        assert_eq!(metrics.total_pnl, 0.0);
        assert_eq!(metrics.pattern_name, "test_pattern");
    }

    #[test]
    fn test_recording_events() {
        let monitor = PerformanceMonitor::new("test_pattern");
        
        // Record some events
        monitor.record_quote();
        monitor.record_quote();
        monitor.record_order();
        
        let fill = Fill {
            side: Side::Buy,
            qty: 100.0,
            px: 100.0,
            ts: Instant::now(),
        };
        monitor.record_fill(&fill);
        
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.quotes_processed, 2);
        assert_eq!(metrics.orders_sent, 1);
        assert_eq!(metrics.fills_received, 1);
        assert_eq!(metrics.total_pnl, -10000.0); // 100 shares at $100, bought
    }

    #[test]
    fn test_latency_tracking() {
        let monitor = PerformanceMonitor::new("test_pattern");
        
        monitor.record_latency(1000); // 1ms
        monitor.record_latency(2000); // 2ms
        monitor.record_latency(500);  // 0.5ms
        
        let metrics = monitor.get_metrics();
        assert!(metrics.max_latency_us >= 2000);
        assert!(metrics.avg_latency_us > 0.0);
    }

    #[test]
    fn test_rate_calculations() {
        let monitor = PerformanceMonitor::new("test_pattern");
        
        // These tests just verify the methods don't panic
        let metrics = monitor.get_metrics();
        let _ops = metrics.orders_per_second();
        let _fps = metrics.fills_per_second();
        let _qps = metrics.quotes_per_second();
    }
}