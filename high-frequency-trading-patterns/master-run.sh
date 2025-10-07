#!/bin/bash

# Master Run Script for High-Frequency Trading Patterns
# This script runs all 25 HFT patterns concurrently across macOS and Linux systems

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    print_error "Cargo is not installed. Please install Rust from https://www.rust-lang.org/"
    exit 1
fi

# Check if we're in the correct directory
if [[ ! -f "Cargo.toml" ]]; then
    print_error "This script must be run from the root of the high-frequency-trading-patterns directory"
    exit 1
fi

# List of all 25 pattern names
patterns=(
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

# Function to run a pattern
run_pattern() {
    local pattern=$1
    local log_file="logs/${pattern}.log"
    
    print_status "Starting $pattern..."
    
    # Create logs directory if it doesn't exist
    mkdir -p logs
    
    # Run the pattern in release mode and log output
    cargo run --release -p "$pattern" > "$log_file" 2>&1 &
    local pid=$!
    
    echo "$pid" > "pids/${pattern}.pid"
    
    print_success "$pattern started with PID $pid (logging to $log_file)"
}

# Function to stop all patterns
stop_all_patterns() {
    print_status "Stopping all patterns..."
    
    if [[ -d "pids" ]]; then
        for pid_file in pids/*.pid; do
            if [[ -f "$pid_file" ]]; then
                pid=$(cat "$pid_file")
                if kill -0 "$pid" 2>/dev/null; then
                    kill "$pid"
                    print_status "Stopped process $pid"
                fi
                rm "$pid_file"
            fi
        done
    fi
    
    # Kill any remaining cargo processes
    pkill -f "cargo run" 2>/dev/null || true
    
    print_success "All patterns stopped"
}

# Function to show status of all patterns
show_status() {
    print_status "Pattern Status:"
    
    for pattern in "${patterns[@]}"; do
        if [[ -f "pids/${pattern}.pid" ]]; then
            pid=$(cat "pids/${pattern}.pid")
            if kill -0 "$pid" 2>/dev/null; then
                print_success "  $pattern: RUNNING (PID $pid)"
            else
                print_error "  $pattern: STOPPED (PID $pid)"
                rm "pids/${pattern}.pid"
            fi
        else
            print_warning "  $pattern: NOT RUNNING"
        fi
    done
}

# Function to build all patterns
build_all() {
    print_status "Building all patterns in release mode..."
    cargo build --release
    if [[ $? -eq 0 ]]; then
        print_success "All patterns built successfully"
    else
        print_error "Build failed"
        exit 1
    fi
}

# Function to show help
show_help() {
    echo "High-Frequency Trading Patterns Master Run Script"
    echo ""
    echo "Usage: ./master-run.sh [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  start     Start all 25 HFT patterns concurrently"
    echo "  stop      Stop all running patterns"
    echo "  status    Show status of all patterns"
    echo "  build     Build all patterns in release mode"
    echo "  help      Show this help message"
    echo ""
    echo "Examples:"
    echo "  ./master-run.sh start    # Start all patterns"
    echo "  ./master-run.sh stop     # Stop all patterns"
    echo "  ./master-run.sh status   # Show pattern status"
}

# Create pids directory
mkdir -p pids

# Handle script arguments
case "${1:-start}" in
    start)
        print_status "Starting all 25 High-Frequency Trading Patterns..."
        
        # Build all patterns first
        build_all
        
        # Start all patterns
        for pattern in "${patterns[@]}"; do
            run_pattern "$pattern"
            # Small delay to prevent overwhelming the system
            sleep 0.1
        done
        
        print_success "All patterns started! Check logs/ directory for output."
        print_status "Use './master-run.sh status' to check pattern status"
        print_status "Use './master-run.sh stop' to stop all patterns"
        ;;
        
    stop)
        stop_all_patterns
        ;;
        
    status)
        show_status
        ;;
        
    build)
        build_all
        ;;
        
    help|--help|-h)
        show_help
        ;;
        
    *)
        print_error "Unknown command: $1"
        show_help
        exit 1
        ;;
esac