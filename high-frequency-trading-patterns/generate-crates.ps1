# PowerShell script to generate all HFT pattern crates
param(
    [string]$WorkspacePath = "c:\Users\RMT\Documents\vscodium\Master-Test-Cases-Rust\high-frequency-trading-patterns"
)

# 25 pattern crate names (kebab-case)
$Patterns = @(
    "market-making"
    "statistical-arbitrage"
    "index-etf-basis-arb"
    "triangular-cross-exchange-arb"
    "latency-arbitrage"
    "event-news-algo"
    "dark-midpoint-arb"
    "liquidity-detection-scout"
    "rebate-fee-arb"
    "queue-dynamics"
    "auction-imbalance-alpha"
    "options-vol-arb"
    "inventory-aware-exec"
    "orderbook-ml-microstructure"
    "sor-venue-alpha"
    "flow-anticipation"
    "liquidity-mirroring"
    "momentum-ignition"
    "spoofing-layering"
    "quote-stuffing"
    "wash-painting"
    "last-look-fx"
    "stop-trigger-hunting"
    "opening-gap-fade"
    "cross-asset-latency-lead"
)

# Function to create Cargo.toml for each pattern
function Create-PatternCargoToml {
    param([string]$Name)
    return @"
[package]
name = "$Name"
version = "0.1.0"
edition = "2021"
license = "MIT"

[dependencies]
hft-common = { path = "../hft-common" }
anyhow = "1.0"
tokio = { version = "1.38", features = ["full"] }
tracing = "0.1"
tracing-subscriber = "0.3"
"@
}

# Function to create main.rs for each pattern
function Create-PatternMainRs {
    return @"
//! pattern crate
mod simulator;
mod strategy;
mod risk;

use anyhow::Result;
use hft_common::prelude::*;

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
"@
}

# Function to create simulator.rs for each pattern
function Create-PatternSimulatorRs {
    return @"
use hft_common::prelude::*;
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
"@
}

# Function to create strategy.rs for each pattern
function Create-PatternStrategyRs {
    return @"
use hft_common::prelude::*;

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
"@
}

# Function to create risk.rs for each pattern
function Create-PatternRiskRs {
    return @"
use hft_common::prelude::*;
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
"@
}

# Generate all 25 pattern crates
foreach ($PatternName in $Patterns) {
    $DirPath = "$WorkspacePath\crates\$PatternName"
    $SrcPath = "$DirPath\src"
    
    # Create directories
    New-Item -ItemType Directory -Path $DirPath -Force | Out-Null
    New-Item -ItemType Directory -Path $SrcPath -Force | Out-Null
    
    # Create files
    Create-PatternCargoToml -Name $PatternName | Out-File -FilePath "$DirPath\Cargo.toml" -Encoding UTF8
    Create-PatternMainRs | Out-File -FilePath "$SrcPath\main.rs" -Encoding UTF8
    Create-PatternSimulatorRs | Out-File -FilePath "$SrcPath\simulator.rs" -Encoding UTF8
    Create-PatternStrategyRs | Out-File -FilePath "$SrcPath\strategy.rs" -Encoding UTF8
    Create-PatternRiskRs | Out-File -FilePath "$SrcPath\risk.rs" -Encoding UTF8
    
    Write-Host "Created crate: $PatternName"
}

# Update workspace Cargo.toml to include all members
$CargoTomlPath = "$WorkspacePath\Cargo.toml"
$CargoTomlContent = Get-Content -Path $CargoTomlPath
$NewMembers = $Patterns | ForEach-Object { "  ""crates/$_""," }
$CargoTomlContent = $CargoTomlContent -replace "members = \[", "members = [`n$($NewMembers -join "`n")"

# Ensure closing bracket exists
if ($CargoTomlContent -notmatch "\]`nresolver = ""2""") {
    $CargoTomlContent += "`n]`nresolver = ""2"""
}

$CargoTomlContent | Out-File -FilePath $CargoTomlPath -Encoding UTF8

Write-Host "✅ All 25 pattern crates generated successfully!"
Write-Host "➡  Next: cargo build --release"