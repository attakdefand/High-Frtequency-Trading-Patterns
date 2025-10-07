# Master Run Scripts for High-Frequency Trading Patterns

This directory contains master run scripts that allow you to easily start, stop, and manage all 25 high-frequency trading patterns across different operating systems.

## Available Scripts

### For Windows Systems
- [master-run.ps1](file:///c%3A/Users/RMT/Documents/vscodium/Master-Test-Cases-Rust/high-frequency-trading-patterns/master-run.ps1) - PowerShell script for Windows
- [master-run.sh](file:///c%3A/Users/RMT/Documents/vscodium/Master-Test-Cases-Rust/high-frequency-trading-patterns/master-run.sh) - Bash script for Windows (requires WSL or Git Bash)

### For macOS and Linux Systems
- [master-run.sh](file:///c%3A/Users/RMT/Documents/vscodium/Master-Test-Cases-Rust/high-frequency-trading-patterns/master-run.sh) - Bash script for macOS and Linux

## Prerequisites

1. **Rust Toolchain** - Install from [https://www.rust-lang.org/](https://www.rust-lang.org/)
2. **Git** - For version control (optional but recommended)

## Usage

### Windows

You can use either PowerShell or Bash on Windows:

#### Using PowerShell (Recommended for Windows):

Open PowerShell as Administrator and navigate to the project root directory:

```powershell
# Start all 25 patterns
.\master-run.ps1 start

# Stop all patterns
.\master-run.ps1 stop

# Check status of all patterns
.\master-run.ps1 status

# Build all patterns
.\master-run.ps1 build

# Show help
.\master-run.ps1 help
```

#### Using Bash on Windows (with WSL or Git Bash):

Open WSL, Git Bash, or similar and navigate to the project root directory:

```bash
# Make the script executable
chmod +x master-run.sh

# Start all 25 patterns
./master-run.sh start

# Stop all patterns
./master-run.sh stop

# Check status of all patterns
./master-run.sh status

# Build all patterns
./master-run.sh build

# Show help
./master-run.sh help
```

### macOS/Linux

Open Terminal and navigate to the project root directory:

```bash
# Make the script executable
chmod +x master-run.sh

# Start all 25 patterns
./master-run.sh start

# Stop all patterns
./master-run.sh stop

# Check status of all patterns
./master-run.sh status

# Build all patterns
./master-run.sh build

# Show help
./master-run.sh help
```

## What the Scripts Do

### Start Command
1. Builds all 25 patterns in release mode (optimized for performance)
2. Starts each pattern as a separate background process
3. Logs output from each pattern to the `logs/` directory
4. Saves process IDs to the `pids/` directory for management

### Stop Command
1. Stops all running pattern processes using their saved PIDs
2. Cleans up PID files
3. Kills any remaining cargo processes

### Status Command
1. Shows the current status of all 25 patterns
2. Indicates which patterns are running and which are stopped
3. Displays process IDs for running patterns

### Build Command
1. Builds all 25 patterns in release mode
2. Does not start the patterns (useful for pre-building)

## Directory Structure

When running the scripts, the following directories will be created:

```
high-frequency-trading-patterns/
├── logs/                    # Log files for each pattern
│   ├── market-making.log
│   ├── statistical-arbitrage.log
│   └── ... (23 more log files)
├── pids/                    # Process ID files for management
│   ├── market-making.pid
│   ├── statistical-arbitrage.pid
│   └── ... (23 more PID files)
├── master-run.ps1          # Windows PowerShell script
├── master-run.sh           # macOS/Linux Bash script
└── MASTER-RUN-README.md    # This file
```

## Monitoring Patterns

Each pattern's output is logged to a separate file in the `logs/` directory. You can monitor individual patterns using:

```bash
# On macOS/Linux or Windows with WSL/Git Bash
tail -f logs/market-making.log

# On Windows (PowerShell)
Get-Content logs\market-making.log -Wait
```

## Performance Considerations

Running all 25 patterns simultaneously can be resource-intensive:

1. **CPU Usage**: Each pattern is designed to be CPU-intensive, especially in release mode
2. **Memory Usage**: Each pattern runs in its own process, consuming system memory
3. **Disk I/O**: Logging output from all patterns may impact disk performance

For systems with limited resources, consider running patterns individually or in smaller groups.

## Troubleshooting

### Patterns Not Starting
- Ensure you have sufficient system resources
- Check log files in the `logs/` directory for error messages
- Verify that Rust and Cargo are properly installed

### Patterns Not Stopping
- Use the `stop` command to properly terminate all patterns
- If patterns continue running, you may need to manually kill processes:
  ```bash
  # On macOS/Linux or Windows with WSL/Git Bash
  pkill -f "cargo run"
  
  # On Windows (PowerShell)
  Get-Process -Name "cargo" | Stop-Process -Force
  ```

### Permission Issues (Windows)
- Run PowerShell as Administrator
- Ensure you have write permissions to the project directory

### Permission Issues (macOS/Linux)
- Ensure the script is executable:
  ```bash
  chmod +x master-run.sh
  ```
- Run with appropriate permissions for process management

## Customization

You can modify the scripts to:
1. Change the build profile (debug vs release)
2. Adjust the delay between starting patterns
3. Modify logging behavior
4. Add additional process management features

The scripts are designed to be easily customizable while maintaining cross-platform compatibility.