//! Router price impact benchmarks
//! 
//! This benchmark suite tests the performance of AMM routing algorithms
//! and price impact calculations under various market conditions.

use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};
use sniper_amm::{cpmm, stableswap, univ3, Router};
use sniper_core::types::{ChainRef, TradePlan, ExecMode, GasPolicy, ExitRules};
use tokio::runtime::Runtime;

/// Benchmark CPMM router price impact calculation
fn bench_cpmm_price_impact(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let router = cpmm::Router::new();
    let plan = create_test_plan();
    
    c.bench_function("cpmm_price_impact", |b: &mut Bencher| {
        b.iter(|| {
            let result = router.get_quote(&plan);
            black_box(result)
        })
    });
}

/// Benchmark StableSwap router price impact calculation
fn bench_stableswap_price_impact(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let router = stableswap::Router::new();
    let plan = create_test_plan();
    
    c.bench_function("stableswap_price_impact", |b: &mut Bencher| {
        b.iter(|| {
            let result = router.get_quote(&plan);
            black_box(result)
        })
    });
}

/// Benchmark UniV3 router price impact calculation
fn bench_univ3_price_impact(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let router = univ3::Router::new();
    let plan = create_test_plan();
    
    c.bench_function("univ3_price_impact", |b: &mut Bencher| {
        b.iter(|| {
            let result = router.get_quote(&plan);
            black_box(result)
        })
    });
}

/// Benchmark main router with multiple AMM protocols
fn bench_main_router(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let router = Router::new();
    let plan = create_test_plan();
    
    c.bench_function("main_router", |b: &mut Bencher| {
        b.iter(|| {
            let result = router.get_quote(&plan);
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
    bench_cpmm_price_impact,
    bench_stableswap_price_impact,
    bench_univ3_price_impact,
    bench_main_router
);
criterion_main!(benches);