# Sniper-RS Service Ports and Run Commands

## Service Classification

The Sniper-RS services can be classified into two categories:
1. **Web-based services** - Services that expose HTTP APIs
2. **Terminal-based services** - Services that run in the terminal without web interfaces

## Web-Based Services

Currently, the Sniper-RS project has web-based capabilities planned but not fully implemented:

| Service Name | Port | Description | Status |
|--------------|------|-------------|--------|
| svc-gateway | None | HTTP API Gateway | Planned (no HTTP server implemented yet) |
| svc-portfolio | 8080 | Portfolio management REST API | Planned (no HTTP server implemented yet) |
| svc-orders | 8081 | Advanced order types REST API | Planned (no HTTP server implemented yet) |

Note: These services accept `--port` arguments but currently don't implement HTTP servers. A complete implementation would use the Axum web framework (which is included in workspace dependencies) to expose REST APIs.

## Terminal-Based Services

These services run in the terminal and communicate via the message bus:

| Service Name | Port | Description |
|--------------|------|-------------|
| svc-signals | None | Signal processing service |
| svc-strategy | None | Trading strategy engine |
| svc-executor | None | Trade execution service |
| svc-risk | None | Risk management service |
| svc-nft | None | NFT marketplace integration |
| svc-cex | None | Centralized exchange integration |
| svc-policy | None | Policy enforcement service |
| svc-storage | None | Data storage service |

## Message Bus

| Service | Port | Description |
|---------|------|-------------|
| nats | 4222, 8222 | Message bus for inter-service communication |

## Running Services

### From the Root Directory

All services can be run from the root directory of the project using cargo commands.

#### Running Individual Services

```bash
# Run the gateway service (terminal-based currently)
cargo run -p svc-gateway

# Run the signals service
cargo run -p svc-signals

# Run the strategy service
cargo run -p svc-strategy

# Run the executor service
cargo run -p svc-executor

# Run the risk service
cargo run -p svc-risk

# Run the NFT service
cargo run -p svc-nft

# Run the CEX service
cargo run -p svc-cex

# Run the policy service
cargo run -p svc-policy

# Run the storage service
cargo run -p svc-storage

# Run the portfolio service (accepts --port but runs in terminal)
cargo run -p svc-portfolio

# Run the orders service (accepts --port but runs in terminal)
cargo run -p svc-orders
```

#### Running Services with Custom Ports

The portfolio and orders services accept a `--port` argument for when they are fully implemented with HTTP servers:

```bash
# Run the portfolio service on port 9000
cargo run -p svc-portfolio -- --port 9000

# Run the orders service on port 9001
cargo run -p svc-orders -- --port 9001
```

#### Running Multiple Services Concurrently

You can run multiple services in separate terminals or use tools like `tmux` or `screen` to manage them.

Example using multiple terminal sessions:
```bash
# Terminal 1 - Start NATS first
docker run -p 4222:4222 -p 8222:8222 nats:2 -js -m 8222

# Terminal 2
cargo run -p svc-signals

# Terminal 3
cargo run -p svc-strategy

# Terminal 4
cargo run -p svc-executor

# Terminal 5
cargo run -p svc-portfolio
```

### Using the Development Script

The project includes a development script to run some services locally:

```bash
# Make the script executable (first time only)
chmod +x scripts/dev_run_local.sh

# Run services using the script (starts signals, strategy, executor, and gateway)
./scripts/dev_run_local.sh
```

Note: The script runs svc-signals, svc-strategy, svc-executor, and svc-gateway concurrently, with logs redirected to `/tmp/` files.

### Docker Deployment

Services can also be deployed using Docker Compose. The default configuration includes NATS and several core services:

```bash
# Navigate to the infra directory
cd infra

# Build and run all services
docker-compose up

# Build all services
docker-compose build

# Run specific services
docker-compose up nats gateway strategy executor
```

The default Docker Compose configuration includes:
- NATS message bus on ports 4222 and 8222
- Gateway service (without exposed ports in current implementation)
- Signals, strategy, and executor services (without exposed ports)

### Service Dependencies

Services communicate through a message bus (NATS). Make sure the message bus is running before starting the services:

```bash
# Using Docker (recommended)
docker run -p 4222:4222 -p 8222:8222 nats:2 -js -m 8222

# Or run NATS locally if installed
nats-server -js -m 8222
```

### Environment Variables

Services may require environment variables to be set. Check the `.env.example` file in the root directory for required variables:

```bash
# Copy the example environment file
cp .env.example .env

# Edit the .env file with your configuration
# Then source it (in bash/zsh)
source .env

# Or export variables directly
export NATS_URL=nats://localhost:4222
export RUST_LOG=info
```

Common environment variables include:
- `RUST_LOG`: Log level (info, debug, warn, error)
- `NATS_URL`: NATS message bus URL (nats://localhost:4222)
- `RPC_HTTP`: HTTP RPC endpoint for blockchain
- `RPC_WS`: WebSocket RPC endpoint for blockchain
- `POSTGRES_DSN`: PostgreSQL database connection string
- `REDIS_URL`: Redis connection URL

## Health Checks

Services that expose HTTP endpoints typically provide a health check endpoint at `/health`:

```bash
# Check if a service with HTTP endpoint is running (when implemented)
curl http://localhost:8080/health
```

Note: Currently, no services have HTTP endpoints implemented.

## Logging

Services use structured logging. You can control the log level with the `RUST_LOG` environment variable:

```bash
# Set log level to info
RUST_LOG=info cargo run -p svc-strategy

# Set log level to debug for specific modules
RUST_LOG=svc_strategy=debug,sniper_core=info cargo run -p svc-strategy

# Set log level to trace for everything
RUST_LOG=trace cargo run -p svc-signals
```

## Testing Services

Each service includes unit and integration tests:

```bash
# Run tests for a specific service
cargo test -p svc-strategy

# Run all tests
cargo test

# Run tests for all services
cargo test --workspace
```

## Debugging Services

For debugging, you can use the following approaches:

```bash
# Run with debug logging
RUST_LOG=debug cargo run -p svc-executor

# Run with backtrace on panic
RUST_BACKTRACE=1 cargo run -p svc-risk

# Run with all optimizations disabled for better debugging
cargo run -p svc-signals --profile dev
```

### IDE Debugging

For debugging in IDEs like VS Code or IntelliJ:
1. Set breakpoints in the code
2. Use the "Debug" run configuration
3. The debugger will stop at breakpoints and allow variable inspection

## Stopping Services

When running services in the foreground, use `Ctrl+C` to stop them gracefully.

When running services in the background or using Docker:
```bash
# Stop Docker Compose services
docker-compose down

# Kill background processes (if you noted the PIDs)
kill $PID

# Kill all cargo processes (use with caution)
pkill -f cargo

Web-Based Services (Planned)
svc-gateway - Intended as the main HTTP API Gateway (currently no HTTP implementation)
svc-portfolio - Planned REST API for portfolio management (currently no HTTP implementation)
svc-orders - Planned REST API for advanced order types (currently no HTTP implementation)
Terminal-Based Services (Active)
svc-signals - Signal processing service
svc-strategy - Trading strategy engine
svc-executor - Trade execution service
svc-risk - Risk management service
svc-nft - NFT marketplace integration
svc-cex - Centralized exchange integration
svc-policy - Policy enforcement service
svc-storage - Data storage service

```