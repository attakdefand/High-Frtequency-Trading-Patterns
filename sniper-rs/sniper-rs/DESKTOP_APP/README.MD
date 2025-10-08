# Sniper-RS Desktop Application

This directory contains the desktop application for configuring and managing Sniper-RS trading systems.

## Architecture

The desktop application is built using Tauri (Rust + Web Technologies) and provides:
- Configuration management
- Service orchestration
- Strategy deployment
- System monitoring
- Log analysis

## Features

### Configuration Management
- Service configuration editing
- Strategy parameter tuning
- Risk limit setting
- Policy management

### Service Orchestration
- Start/stop services
- Service status monitoring
- Dependency management
- Health checks

### Strategy Deployment
- Strategy packaging
- Plugin management
- Version control
- Rollback capabilities

### System Monitoring
- Resource usage tracking
- Performance metrics
- Error log analysis
- Alert management

### Reporting
- Trading performance reports
- Risk analysis reports
- Compliance reports
- System health reports

## Technical Implementation

### Tauri Project Structure
```
src/
├── main.rs              # Rust backend
├── App.tsx              # Main React component
├── components/
│   ├── ConfigEditor.tsx
│   ├── ServiceManager.tsx
│   ├── StrategyDeployer.tsx
│   ├── MonitorDashboard.tsx
│   └── ReportGenerator.tsx
├── pages/
│   ├── ConfigurationPage.tsx
│   ├── ServicesPage.tsx
│   ├── DeploymentPage.tsx
│   ├── MonitoringPage.tsx
│   └── ReportsPage.tsx
├── services/
│   ├── config_service.ts
│   ├── service_manager.ts
│   ├── deployment_service.ts
│   └── monitoring_service.ts
├── models/
│   ├── service_config.ts
│   ├── strategy_config.ts
│   ├── deployment.ts
│   └── metrics.ts
└── utils/
    ├── constants.ts
    └── helpers.ts

src-tauri/
├── Cargo.toml
├── tauri.conf.json
└── src/
    ├── main.rs
    └── cmd.rs
```

### Backend Implementation (Rust)
The Rust backend provides:
- File system access for configuration files
- Process management for services
- System metrics collection
- Secure API endpoints

### Frontend Implementation (React/TypeScript)
The frontend provides:
- Modern UI with Material Design
- Real-time updates
- Form validation
- Responsive design

## Development Setup

### Prerequisites
- Rust toolchain
- Node.js and npm
- Tauri CLI
- Sniper-RS services

### Installation
```bash
# Clone the repository
git clone https://github.com/your-org/sniper-rs-desktop.git
cd sniper-rs-desktop

# Install frontend dependencies
npm install

# Install Tauri CLI
cargo install tauri-cli

# Run in development mode
cargo tauri dev
```

## Building for Production

### Windows
```bash
cargo tauri build --target x86_64-pc-windows-msvc
```

### macOS
```bash
cargo tauri build --target x86_64-apple-darwin
```

### Linux
```bash
cargo tauri build --target x86_64-unknown-linux-gnu
```

## Configuration

The desktop app can be configured through:
1. Settings dialog in the app
2. Configuration files
3. Environment variables

Key configuration options:
- Service executable paths
- Configuration file locations
- Logging settings
- Update preferences

## Security

### Authentication
- User authentication
- Role-based access control
- Audit logging

### Data Protection
- Encrypted configuration storage
- Secure communication with services
- Regular security updates

## Integration with Sniper-RS Services

### REST API Communication
The desktop app communicates with Sniper-RS services through:
- svc-gateway for general operations
- svc-monitoring for metrics
- svc-users for authentication
- svc-compliance for audit logs

### File System Integration
Direct file system access for:
- Configuration files
- Log files
- Strategy files
- Report exports

## Support

For support, please contact:
- Email: desktop-support@sniper-rs.com
- GitHub Issues: https://github.com/your-org/sniper-rs-desktop/issues