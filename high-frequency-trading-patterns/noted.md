#!/usr/bin/env bash
set -euo pipefail

# workspace name
WS=high-frequency-trading-patterns

# 25 pattern crate names (kebab-case)
PATTERNS=(
  market-making
  statistical-arbitrage
  index-etf-basis-arb
  triangular-cross-exchange-arb
  latency-arbitrage
  event-news-algo
  dark-midpoint-arb
  liquidity-detection-scout
  rebate-fee-arb
  queue-dynamics
  auction-imbalance-alpha
  options-vol-arb
  inventory-aware-exec
  orderbook-ml-microstructure
  sor-venue-alpha
  flow-anticipation
  liquidity-mirroring
  momentum-ignition
  spoofing-layering
  quote-stuffing
  wash-painting
  last-look-fx
  stop-trigger-hunting
  opening-gap-fade
  cross-asset-latency-lead
)

# 1) workspace skeleton
mkdir -p "$WS"/crates
cd "$WS"

# Root Cargo.toml (members appended later)
cat > Cargo.toml <<'TOML'
[workspace]
members = [
  "crates/hft-common",
]
resolver = "2"
TOML

# README & LICENSE
cat > README.md <<'MD'
# high-frequency-trading-patterns

Rust workspace hosting 25 HFT strategy patterns. Each pattern is a small micro-service:
- `simulator.rs` — toy market / environment
- `strategy.rs`  — strategy logic
- `risk.rs`      — simple risk caps
- `main.rs`      — wiring

Shared types in `crates/hft-common`.

> Educational only. Some patterns (e.g., spoofing, stuffing) are illegal/abusive in real markets.
MD

cat > LICENSE <<'LIC'
MIT License
Copyright (c) 2025
Permission is hereby granted, free of charge, to any person obtaining a copy...
LIC

#############################################
# 2) shared crate: hft-common (lightweight) #
#############################################
mkdir -p crates/hft-common/src

cat > crates/hft-common/Cargo.toml <<'TOML'
[package]
name = "hft-common"
version = "0.1.0"
edition = "2021"
license = "MIT"

[dependencies]
tokio   = { version = "1.38", features = ["full"] }
tracing = "0.1"
serde   = { version = "1.0", features = ["derive"] }
TOML

cat > crates/hft-common/src/lib.rs <<'RS'
pub mod models;
pub mod config;
pub mod prelude;
RS

cat > crates/hft-common/src/models.rs <<'RS'
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side { Buy, Sell }

#[derive(Clone, Debug)]
pub struct Quote {
    pub bid: f64,
    pub ask: f64,
    pub ts:  Instant,
}

#[derive(Clone, Debug)]
pub struct Order {
    pub side: Side,
    pub qty:  f64,
    pub px:   f64,
}

#[derive(Clone, Debug)]
pub struct Fill {
    pub side: Side,
    pub qty:  f64,
    pub px:   f64,
    pub ts:   Instant,
}
RS

cat > crates/hft-common/src/config.rs <<'RS'
use serde::Deserialize;

/// Minimal config (kept pure-Rust so all crates compile immediately)
#[derive(Debug, Clone, Deserialize)]
pub struct Cfg {
    pub symbol:  String,
    pub tick_ms: u64,
    pub tick_sz: f64,
    pub max_pos: f64,
    pub max_orders_s: usize,
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            symbol: "XYZ".into(),
            tick_ms: 1,
            tick_sz: 0.01,
            max_pos: 10_000.0,
            max_orders_s: 50_000,
        }
    }
}
RS

cat > crates/hft-common/src/prelude.rs <<'RS'
pub use crate::{config::Cfg, models::*};
pub use tokio::sync::mpsc;
pub use tracing::{info, warn, debug};
RS

#############################################
# 3) template contents for each pattern     #
#############################################
pattern_cargo_toml() {
cat <<'TOML'
[package]
name = "__NAME__"
version = "0.1.0"
edition = "2021"
license = "MIT"

[dependencies]
hft-common = { path = "../hft-common" }
anyhow = "1.0"
tokio = { version = "1.38", features = ["full"] }
tracing = "0.1"
TOML
}

pattern_main_rs() {
cat <<'RS'
//! pattern crate
mod simulator;
mod strategy;
mod risk;

use anyhow::Result;
use hft-common::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cfg = Cfg::default();

    let (md_tx,   mut md_rx) = mpsc::channel::<Quote>(1024);
    let (od_tx,   mut od_rx) = mpsc::channel::<Order>(1024);
    let (fill_tx, mut fills) = mpsc::channel::<Fill>(1024);

    tokio::spawn(simulator::run(cfg.clone(), md_tx, od_rx, fill_tx));

    let mut strat = strategy::Logic::new(cfg.clone());
    let mut risk  = risk::Risk::new(&cfg);

    loop {
        tokio::select! {
            Some(q) = md_rx.recv() => {
                if let Some(o) = strat.on_quote(&q) {
                    if risk.allow(&o) { od_tx.send(o).await?; }
                }
            },
            Some(f) = fills.recv() => {
                strat.on_fill(&f);
                info!("FILL {:?} {:.0} @ {:.2}", f.side, f.qty, f.px);
            },
            else => break,
        }
    }
    Ok(())
}
RS
}

pattern_simulator_rs() {
cat <<'RS'
use hft-common::prelude::*;
use tokio::time::{interval, Duration};

pub async fn run(
    cfg: Cfg,
    md_tx: mpsc::Sender<Quote>,
    mut od_rx: mpsc::Receiver<Order>,
    fill_tx: mpsc::Sender<Fill>,
) {
    let mut mid = 100.00;
    let mut clock = interval(Duration::from_millis(cfg.tick_ms));

    loop {
        tokio::select! {
            _ = clock.tick() => {
                // simple deterministic walk (no RNG -> Send-safe)
                mid += cfg.tick_sz;
                let q = Quote {
                    bid: mid - cfg.tick_sz/2.0,
                    ask: mid + cfg.tick_sz/2.0,
                    ts:  std::time::Instant::now(),
                };
                let _ = md_tx.send(q).await;
            },
            Some(o) = od_rx.recv() => {
                let f = Fill { side: o.side, qty: o.qty, px: o.px, ts: std::time::Instant::now() };
                let _ = fill_tx.send(f).await;
            },
            else => break,
        }
    }
}
RS
}

pattern_strategy_rs() {
cat <<'RS'
use hft-common::prelude::*;

pub struct Logic { cfg: Cfg, ctr: u64 }
impl Logic {
    pub fn new(cfg: Cfg) -> Self { Self { cfg, ctr: 0 } }

    pub fn on_quote(&mut self, q:&Quote) -> Option<Order> {
        self.ctr += 1;
        // placeholder: cross every 256th quote
        if self.ctr % 256 == 0 {
            Some(Order{ side: Side::Buy, qty: 100.0, px: q.ask })
        } else { None }
    }
    pub fn on_fill(&mut self, _f:&Fill) {}
}
RS
}

pattern_risk_rs() {
cat <<'RS'
use hft-common::prelude::*;
use std::time::Instant;

pub struct Risk {
    max_pos: f64,
    max_orders: usize,
    sent_this_sec: usize,
    last_sec: Instant,
    pos: f64,
}
impl Risk {
    pub fn new(cfg:&Cfg) -> Self {
        Self{ max_pos:cfg.max_pos, max_orders:cfg.max_orders_s,
              sent_this_sec:0, last_sec:Instant::now(), pos:0.0 }
    }
    pub fn allow(&mut self, o:&Order) -> bool {
        if self.last_sec.elapsed().as_secs() >= 1 {
            self.last_sec = Instant::now(); self.sent_this_sec = 0;
        }
        if self.sent_this_sec >= self.max_orders { return false; }
        let delta = if o.side==Side::Buy { o.qty } else { -o.qty };
        if (self.pos + delta).abs() > self.max_pos { return false; }
        self.pos += delta; self.sent_this_sec += 1; true
    }
}
RS
}

# 4) generate all 25 pattern crates
for NAME in "${PATTERNS[@]}"; do
  DIR="crates/$NAME"
  mkdir -p "$DIR/src"

  pattern_cargo_toml | sed "s/__NAME__/$NAME/" > "$DIR/Cargo.toml"
  pattern_main_rs      > "$DIR/src/main.rs"
  pattern_simulator_rs > "$DIR/src/simulator.rs"
  pattern_strategy_rs  > "$DIR/src/strategy.rs"
  pattern_risk_rs      > "$DIR/src/risk.rs"

  # add to workspace members
  awk -v add="  \"crates/$NAME\"," '
    BEGIN{printed=0}
    /^\[workspace\]/ { print; next }
    /^\]$/ && printed==0 { print; next }
    /members = \[/ {
      print;
      print add;
      printed=1;
      next
    }
    { print }
  ' Cargo.toml > Cargo.toml.tmp
  mv Cargo.toml.tmp Cargo.toml
done

# Ensure closing bracket exists (in case awk flow missed)
if ! grep -q ']' Cargo.toml; then
  echo ']' >> Cargo.toml
fi

echo "✅ Workspace created: $WS"
echo "➡  Next: cargo build --release"
