//! Risk evaluation benchmarks
//! 
//! This benchmark suite tests the performance of risk evaluation
//! and decision-making components.

use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};
use sniper_risk::{decide, honeypot, lp_quality, owner_powers};
use sniper_core::types::{TradePlan, ChainRef, ExecMode, GasPolicy, ExitRules};
use tokio::runtime::Runtime;

/// Benchmark risk decision making
fn bench_risk_decision(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let plan = create_test_plan();
    
    c.bench_function("risk_decision", |b: &mut Bencher| {
        b.iter(|| {
            let result = decide::evaluate_trade(&plan);
            black_box(result)
        })
    });
}

/// Benchmark honeypot detection
fn bench_honeypot_detection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let plan = create_test_plan();
    
    c.bench_function("honeypot_detection", |b: &mut Bencher| {
        b.iter(|| {
            let result = honeypot::check_honeypot(&plan);
            black_box(result)
        })
    });
}

/// Benchmark LP quality checks
fn bench_lp_quality(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let plan = create_test_plan();
    
    c.bench_function("lp_quality", |b: &mut Bencher| {
        b.iter(|| {
            let result = lp_quality::check_lp_quality(&plan);
            black_box(result)
        })
    });
}

/// Benchmark owner power analysis
fn bench_owner_power(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let plan = create_test_plan();
    
    c.bench_function("owner_power", |b: &mut Bencher| {
        b.iter(|| {
            let result = owner_powers::analyze_owner_powers(&plan);
            black_box(result)
        })
    });
}

/// Create a test trade plan for benchmarking
fn create_test_plan() -> TradePlan {
    TradePlan {
        chain: ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        },
        router: "0xRouter".to_string(),
        token_in: "0xTokenIn".to_string(),
        token_out: "0xTokenOut".to_string(),
        amount_in: 1000000000000000000, // 1 ETH
        min_out: 900000000000000000,    // 0.9 ETH worth of tokens
        mode: ExecMode::Mempool,
        gas: GasPolicy {
            max_fee_gwei: 50,
            max_priority_gwei: 2,
        },
        exits: ExitRules {
            take_profit_pct: Some(10.0),
            stop_loss_pct: Some(5.0),
            trailing_pct: Some(2.0),
        },
        idem_key: "benchmark-key".to_string(),
    }
}

criterion_group!(
    benches,
    bench_risk_decision,
    bench_honeypot_detection,
    bench_lp_quality,
    bench_owner_power
);
criterion_main!(benches);