pub mod models;
pub mod config;
pub mod prelude;
pub mod enhanced_risk;
pub mod enhanced_mm;
pub mod enhanced_arb;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_side_enum() {
        assert_eq!(models::Side::Buy, models::Side::Buy);
        assert_eq!(models::Side::Sell, models::Side::Sell);
        assert_ne!(models::Side::Buy, models::Side::Sell);
    }

    #[test]
    fn test_quote_struct() {
        let quote = models::Quote {
            bid: 99.50,
            ask: 100.50,
            ts: std::time::Instant::now(),
        };
        
        assert_eq!(quote.bid, 99.50);
        assert_eq!(quote.ask, 100.50);
    }

    #[test]
    fn test_order_struct() {
        let order = models::Order {
            side: models::Side::Buy,
            qty: 100.0,
            px: 100.50,
        };
        
        assert_eq!(order.side, models::Side::Buy);
        assert_eq!(order.qty, 100.0);
        assert_eq!(order.px, 100.50);
    }

    #[test]
    fn test_fill_struct() {
        let fill = models::Fill {
            side: models::Side::Sell,
            qty: 100.0,
            px: 99.50,
            ts: std::time::Instant::now(),
        };
        
        assert_eq!(fill.side, models::Side::Sell);
        assert_eq!(fill.qty, 100.0);
        assert_eq!(fill.px, 99.50);
    }

    #[test]
    fn test_config_default() {
        let cfg = config::Cfg::default();
        
        assert_eq!(cfg.symbol, "XYZ");
        assert_eq!(cfg.tick_ms, 1);
        assert_eq!(cfg.tick_sz, 0.01);
        assert_eq!(cfg.max_pos, 10_000.0);
        assert_eq!(cfg.max_orders_s, 50_000);
        // New risk management parameters
        assert_eq!(cfg.max_drawdown, 1_000.0);
        assert_eq!(cfg.max_order_value, 100_000.0);
        assert_eq!(cfg.circuit_breaker_pct, 5.0);
        assert_eq!(cfg.circuit_breaker_duration, 60);
    }

    #[test]
    fn test_config_custom() {
        let cfg = config::Cfg {
            symbol: "ABC".to_string(),
            tick_ms: 5,
            tick_sz: 0.05,
            max_pos: 5_000.0,
            max_orders_s: 25_000,
            // New risk management parameters
            max_drawdown: 500.0,
            max_order_value: 50_000.0,
            circuit_breaker_pct: 3.0,
            circuit_breaker_duration: 30,
        };
        
        assert_eq!(cfg.symbol, "ABC");
        assert_eq!(cfg.tick_ms, 5);
        assert_eq!(cfg.tick_sz, 0.05);
        assert_eq!(cfg.max_pos, 5_000.0);
        assert_eq!(cfg.max_orders_s, 25_000);
        // New risk management parameters
        assert_eq!(cfg.max_drawdown, 500.0);
        assert_eq!(cfg.max_order_value, 50_000.0);
        assert_eq!(cfg.circuit_breaker_pct, 3.0);
        assert_eq!(cfg.circuit_breaker_duration, 30);
    }
}