//! Gas bidding performance benchmarks
//! 
//! This benchmark suite tests the performance of gas bidding algorithms
//! under various conditions and load scenarios.

use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};
use sniper_exec::gas::{GasBidder, GasPolicy};
use tokio::runtime::Runtime;

/// Benchmark basic gas bidding calculation
fn bench_basic_gas_bidding(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let bidder = GasBidder::new();
    let policy = GasPolicy {
        max_fee_gwei: 50,
        max_priority_gwei: 2,
    };
    
    c.bench_function("basic_gas_bidding", |b: &mut Bencher| {
        b.iter(|| {
            let result = rt.block_on(bidder.calculate_bid(&policy, 100));
            black_box(result)
        })
    });
}

/// Benchmark gas bidding with network congestion
fn bench_congested_gas_bidding(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let bidder = GasBidder::new();
    let policy = GasPolicy {
        max_fee_gwei: 200,
        max_priority_gwei: 10,
    };
    
    c.bench_function("congested_gas_bidding", |b: &mut Bencher| {
        b.iter(|| {
            let result = rt.block_on(bidder.calculate_bid(&policy, 900));
            black_box(result)
        })
    });
}

/// Benchmark gas bidding with conservative policy
fn bench_conservative_gas_bidding(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let bidder = GasBidder::new();
    let policy = GasPolicy {
        max_fee_gwei: 30,
        max_priority_gwei: 1,
    };
    
    c.bench_function("conservative_gas_bidding", |b: &mut Bencher| {
        b.iter(|| {
            let result = rt.block_on(bidder.calculate_bid(&policy, 50));
            black_box(result)
        })
    });
}

criterion_group!(
    benches,
    bench_basic_gas_bidding,
    bench_congested_gas_bidding,
    bench_conservative_gas_bidding
);
criterion_main!(benches);