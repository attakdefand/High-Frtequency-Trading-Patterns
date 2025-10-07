# Master Run Script for High-Frequency Trading Patterns
# This script runs all 25 HFT patterns concurrently on Windows systems

# Colors for output
$RESET = [System.ConsoleColor]::White
$INFO = [System.ConsoleColor]::Blue
$SUCCESS = [System.ConsoleColor]::Green
$WARNING = [System.ConsoleColor]::Yellow
$ERROR = [System.ConsoleColor]::Red

# Function to print colored output
function Print-Status {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor $INFO
}

function Print-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor $SUCCESS
}

function Print-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor $WARNING
}

function Print-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor $ERROR
}

# Check if cargo is installed
try {
    $cargoVersion = cargo --version 2>$null
    if (-not $cargoVersion) {
        throw "Cargo not found"
    }
} catch {
    Print-Error "Cargo is not installed. Please install Rust from https://www.rust-lang.org/"
    exit 1
}

# Check if we're in the correct directory
if (-not (Test-Path "Cargo.toml")) {
    Print-Error "This script must be run from the root of the high-frequency-trading-patterns directory"
    exit 1
}

# List of all 25 pattern names
$patterns = @(
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
function Run-Pattern {
    param([string]$Pattern)
    
    $logFile = "logs\$Pattern.log"
    Print-Status "Starting $Pattern..."
    
    # Create logs directory if it doesn't exist
    if (-not (Test-Path "logs")) {
        New-Item -ItemType Directory -Path "logs" | Out-Null
    }
    
    # Start the pattern process in release mode
    $process = Start-Process -FilePath "cargo" -ArgumentList "run", "--release", "-p", $Pattern -NoNewWindow -PassThru -RedirectStandardOutput $logFile -RedirectStandardError $logFile
    
    # Save the process ID
    if (-not (Test-Path "pids")) {
        New-Item -ItemType Directory -Path "pids" | Out-Null
    }
    
    $process.Id | Out-File -FilePath "pids\$Pattern.pid" -Encoding UTF8
    
    Print-Success "$Pattern started with PID $($process.Id) (logging to $logFile)"
}

# Function to stop all patterns
function Stop-All-Patterns {
    Print-Status "Stopping all patterns..."
    
    if (Test-Path "pids") {
        Get-ChildItem "pids\*.pid" | ForEach-Object {
            $pidFile = $_.FullName
            $patternName = $_.BaseName
            
            if (Test-Path $pidFile) {
                $pid = Get-Content $pidFile
                
                # Check if process is still running
                try {
                    $process = Get-Process -Id $pid -ErrorAction Stop
                    if ($process) {
                        Stop-Process -Id $pid -Force
                        Print-Status "Stopped process $pid for pattern $patternName"
                    }
                } catch {
                    Print-Warning "Process $pid for pattern $patternName is not running"
                }
                
                Remove-Item $pidFile -Force
            }
        }
    }
    
    # Kill any remaining cargo processes
    Get-Process -Name "cargo" -ErrorAction SilentlyContinue | Stop-Process -Force
    
    Print-Success "All patterns stopped"
}

# Function to show status of all patterns
function Show-Status {
    Print-Status "Pattern Status:"
    
    foreach ($pattern in $patterns) {
        $pidFile = "pids\$pattern.pid"
        
        if (Test-Path $pidFile) {
            $pid = Get-Content $pidFile
            
            try {
                $process = Get-Process -Id $pid -ErrorAction Stop
                if ($process) {
                    Print-Success "  $pattern`: RUNNING (PID $pid)"
                } else {
                    Print-Error "  $pattern`: STOPPED (PID $pid)"
                    Remove-Item $pidFile -Force
                }
            } catch {
                Print-Error "  $pattern`: STOPPED (PID $pid)"
                Remove-Item $pidFile -Force
            }
        } else {
            Print-Warning "  $pattern`: NOT RUNNING"
        }
    }
}

# Function to build all patterns
function Build-All {
    Print-Status "Building all patterns in release mode..."
    
    $result = cargo build --release 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        Print-Success "All patterns built successfully"
    } else {
        Print-Error "Build failed"
        Print-Error $result
        exit 1
    }
}

# Function to show help
function Show-Help {
    Write-Host "High-Frequency Trading Patterns Master Run Script"
    Write-Host ""
    Write-Host "Usage: .\master-run.ps1 [COMMAND]"
    Write-Host ""
    Write-Host "Commands:"
    Write-Host "  start     Start all 25 HFT patterns concurrently"
    Write-Host "  stop      Stop all running patterns"
    Write-Host "  status    Show status of all patterns"
    Write-Host "  build     Build all patterns in release mode"
    Write-Host "  help      Show this help message"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\master-run.ps1 start    # Start all patterns"
    Write-Host "  .\master-run.ps1 stop     # Stop all patterns"
    Write-Host "  .\master-run.ps1 status   # Show pattern status"
}

# Handle script arguments
$command = if ($args.Count -gt 0) { $args[0] } else { "start" }

switch ($command) {
    "start" {
        Print-Status "Starting all 25 High-Frequency Trading Patterns..."
        
        # Build all patterns first
        Build-All
        
        # Start all patterns
        foreach ($pattern in $patterns) {
            Run-Pattern -Pattern $pattern
            # Small delay to prevent overwhelming the system
            Start-Sleep -Milliseconds 100
        }
        
        Print-Success "All patterns started! Check logs\ directory for output."
        Print-Status "Use '.\master-run.ps1 status' to check pattern status"
        Print-Status "Use '.\master-run.ps1 stop' to stop all patterns"
    }
    
    "stop" {
        Stop-All-Patterns
    }
    
    "status" {
        Show-Status
    }
    
    "build" {
        Build-All
    }
    
    "help" {
        Show-Help
    }
    
    "--help" {
        Show-Help
    }
    
    "-h" {
        Show-Help
    }
    
    default {
        Print-Error "Unknown command: $command"
        Show-Help
        exit 1
    }
}