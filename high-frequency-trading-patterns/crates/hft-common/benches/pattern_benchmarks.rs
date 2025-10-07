// Benchmark tests for HFT patterns performance optimization
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hft_common::prelude::*;
use std::time::Instant;

// Benchmark the basic data structures
fn benchmark_models(c: &mut Criterion) {
    c.bench_function("quote_creation", |b| {
        b.iter(|| {
            let quote = black_box(Quote {
                bid: 99.50,
                ask: 100.50,
                ts: Instant::now(),
            });
            quote
        })
    });

    c.bench_function("order_creation", |b| {
        b.iter(|| {
            let order = black_box(Order {
                side: Side::Buy,
                qty: 100.0,
                px: 100.50,
            });
            order
        })
    });

    c.bench_function("fill_creation", |b| {
        b.iter(|| {
            let fill = black_box(Fill {
                side: Side::Sell,
                qty: 100.0,
                px: 99.50,
                ts: Instant::now(),
            });
            fill
        })
    });
}

// Benchmark the basic risk management
fn benchmark_risk_management(c: &mut Criterion) {
    c.bench_function("risk_check_allow", |b| {
        let cfg = Cfg::default();
        let mut risk = hft_common::enhanced_risk::EnhancedRisk::new(&cfg);
        let order = Order {
            side: Side::Buy,
            qty: 100.0,
            px: 100.50,
        };
        
        b.iter(|| {
            let result = black_box(risk.allow(&order));
            result
        })
    });

    c.bench_function("risk_on_fill", |b| {
        let cfg = Cfg::default();
        let mut risk = hft_common::enhanced_risk::EnhancedRisk::new(&cfg);
        let fill = Fill {
            side: Side::Buy,
            qty: 100.0,
            px: 100.50,
            ts: Instant::now(),
        };
        
        b.iter(|| {
            black_box(risk.on_fill(&fill));
        })
    });

    c.bench_function("risk_on_quote", |b| {
        let cfg = Cfg::default();
        let mut risk = hft_common::enhanced_risk::EnhancedRisk::new(&cfg);
        let quote = Quote {
            bid: 99.50,
            ask: 100.50,
            ts: Instant::now(),
        };
        
        b.iter(|| {
            black_box(risk.on_quote(&quote));
        })
    });
}

// Benchmark the enhanced market making
fn benchmark_market_making(c: &mut Criterion) {
    c.bench_function("mm_on_quote", |b| {
        let cfg = Cfg::default();
        let mut mm = hft_common::enhanced_mm::EnhancedMarketMaking::new(cfg);
        let quote = Quote {
            bid: 99.50,
            ask: 100.50,
            ts: Instant::now(),
        };
        
        b.iter(|| {
            let orders = black_box(mm.on_quote(&quote));
            orders
        })
    });

    c.bench_function("mm_on_fill", |b| {
        let cfg = Cfg::default();
        let mut mm = hft_common::enhanced_mm::EnhancedMarketMaking::new(cfg);
        let fill = Fill {
            side: Side::Buy,
            qty: 100.0,
            px: 100.50,
            ts: Instant::now(),
        };
        
        b.iter(|| {
            black_box(mm.on_fill(&fill));
        })
    });
}

// Benchmark the enhanced arbitrage
fn benchmark_arbitrage(c: &mut Criterion) {
    c.bench_function("arb_statistical_quote", |b| {
        let cfg = Cfg::default();
        let mut arb = hft_common::enhanced_arb::EnhancedArbitrage::new(cfg, hft_common::enhanced_arb::ArbitrageType::Statistical);
        let quote = Quote {
            bid: 99.50,
            ask: 100.50,
            ts: Instant::now(),
        };
        
        b.iter(|| {
            let order = black_box(arb.on_statistical_arbitrage_quote(&quote, 99.75));
            order
        })
    });

    c.bench_function("arb_on_fill", |b| {
        let cfg = Cfg::default();
        let mut arb = hft_common::enhanced_arb::EnhancedArbitrage::new(cfg, hft_common::enhanced_arb::ArbitrageType::Statistical);
        let fill = Fill {
            side: Side::Buy,
            qty: 100.0,
            px: 100.50,
            ts: Instant::now(),
        };
        
        b.iter(|| {
            black_box(arb.on_fill(&fill));
        })
    });
}

criterion_group!(
    benches,
    benchmark_models,
    benchmark_risk_management,
    benchmark_market_making,
    benchmark_arbitrage
);
criterion_main!(benches);