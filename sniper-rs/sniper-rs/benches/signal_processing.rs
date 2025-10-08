//! Signal processing benchmarks
//! 
//! This benchmark suite tests the performance of signal processing
//! and normalization components.

use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};
use sniper_signals::{normalize, Signal};
use sniper_core::types::{ChainRef};
use serde_json::json;
use tokio::runtime::Runtime;

/// Benchmark signal normalization
fn bench_signal_normalization(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let signal = create_test_signal();
    
    c.bench_function("signal_normalization", |b: &mut Bencher| {
        b.iter(|| {
            let result = normalize::normalize_signal(&signal);
            black_box(result)
        })
    });
}

/// Benchmark signal filtering
fn bench_signal_filtering(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let signal = create_test_signal();
    
    c.bench_function("signal_filtering", |b: &mut Bencher| {
        b.iter(|| {
            let result = normalize::should_process_signal(&signal);
            black_box(result)
        })
    });
}

/// Create a test signal for benchmarking
fn create_test_signal() -> Signal {
    Signal {
        source: "dex".to_string(),
        kind: "pair_created".to_string(),
        chain: ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        },
        token0: Some("0xToken0".to_string()),
        token1: Some("0xToken1".to_string()),
        extra: json!({
            "liquidity": 1000000,
            "creator": "0xCreator",
            "timestamp": 1234567890
        }),
        seen_at_ms: 1234567890,
    }
}

criterion_group!(
    benches,
    bench_signal_normalization,
    bench_signal_filtering
);
criterion_main!(benches);