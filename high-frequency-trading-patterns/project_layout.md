high-frequency-trading-patterns/
├── Cargo.toml
├── README.md
├── LICENSE
└── crates/
    ├── hft-common/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── models.rs
    │       ├── config.rs
    │       └── prelude.rs
    ├── market-making/
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
    ├── statistical-arbitrage/
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
    ├── index-etf-basis-arb/
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
    ├── triangular-cross-exchange-arb/
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
    ├── latency-arbitrage/
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
    ├── event-news-algo/
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
    ├── dark-midpoint-arb/
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
    ├── liquidity-detection-scout/
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
    ├── rebate-fee-arb/
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
    ├── queue-dynamics/
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
    ├── auction-imbalance-alpha/
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
    ├── options-vol-arb/
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
    ├── inventory-aware-exec/
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
    ├── orderbook-ml-microstructure/
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
    ├── sor-venue-alpha/
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
    ├── flow-anticipation/
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
    ├── liquidity-mirroring/
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
    ├── momentum-ignition/                # abusive → simulator/surveillance focus only
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
    ├── spoofing-layering/                # illegal → negative test/surveillance
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
    ├── quote-stuffing/                   # abusive → negative test/surveillance
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
    ├── wash-painting/                    # illegal → negative test/surveillance
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
    ├── last-look-fx/                     # venue-controversial
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
    ├── stop-trigger-hunting/             # abusive → negative test/surveillance
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
    ├── opening-gap-fade/
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
    ├── cross-asset-latency-lead/
    │   ├── Cargo.toml
    │   └── src/ { main.rs, simulator.rs, strategy.rs, risk.rs }
