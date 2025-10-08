# Sniper-RS Mobile Application

This directory contains the mobile application for monitoring Sniper-RS trading activities.

## Architecture

The mobile application is built using Flutter/Dart and provides real-time monitoring of:
- Trading performance
- Strategy execution
- Risk metrics
- Market conditions
- Portfolio status

## Features

### Real-time Dashboard
- Live trading performance metrics
- Strategy execution status
- Risk exposure indicators
- Market opportunity alerts

### Strategy Management
- View active strategies
- Monitor strategy performance
- Configure strategy parameters
- Enable/disable strategies

### Risk Monitoring
- Real-time risk metrics
- Position exposure tracking
- Risk limit alerts
- Compliance status

### Portfolio Tracking
- Portfolio value tracking
- Asset allocation visualization
- Performance analytics
- Transaction history

### Notifications
- Trade execution alerts
- Risk limit breaches
- Strategy performance updates
- Market opportunity notifications

## Technical Implementation

### Flutter Project Structure
```
lib/
├── main.dart
├── screens/
│   ├── dashboard_screen.dart
│   ├── strategies_screen.dart
│   ├── risk_screen.dart
│   ├── portfolio_screen.dart
│   └── settings_screen.dart
├── widgets/
│   ├── performance_chart.dart
│   ├── strategy_card.dart
│   ├── risk_indicator.dart
│   └── portfolio_summary.dart
├── services/
│   ├── api_service.dart
│   ├── notification_service.dart
│   └── websocket_service.dart
├── models/
│   ├── strategy.dart
│   ├── trade.dart
│   ├── risk_metrics.dart
│   └── portfolio.dart
└── utils/
    ├── constants.dart
    └── helpers.dart
```

### API Integration
The mobile app integrates with the Sniper-RS REST APIs:
- Gateway service for health checks
- Monitoring service for metrics
- Portfolio service for portfolio data
- Strategy service for strategy information

### WebSocket Connections
Real-time updates are provided through WebSocket connections to:
- svc-monitoring for metrics
- svc-portfolio for portfolio updates
- svc-strategy for strategy status

## Development Setup

### Prerequisites
- Flutter SDK
- Dart SDK
- Android Studio or Xcode
- Sniper-RS services running

### Installation
```bash
# Clone the repository
git clone https://github.com/your-org/sniper-rs-mobile.git
cd sniper-rs-mobile

# Install dependencies
flutter pub get

# Run the app
flutter run
```

## Building for Production

### Android
```bash
flutter build apk --release
```

### iOS
```bash
flutter build ios --release
```

## Configuration

The mobile app can be configured through:
1. Settings screen in the app
2. Environment variables
3. Configuration files

Key configuration options:
- API endpoint URLs
- Authentication tokens
- Notification preferences
- Display settings

## Testing

### Unit Tests
```bash
flutter test
```

### Integration Tests
```bash
flutter test integration_test/
```

## Deployment

### App Stores
- Google Play Store
- Apple App Store

### Enterprise Distribution
- Internal distribution for trading teams
- Custom deployment for institutional clients

## Security

### Authentication
- JWT token authentication
- Biometric authentication (fingerprint, face ID)
- Two-factor authentication

### Data Protection
- Encrypted local storage
- Secure network communication
- Regular security audits

## Support

For support, please contact:
- Email: mobile-support@sniper-rs.com
- GitHub Issues: https://github.com/your-org/sniper-rs-mobile/issues