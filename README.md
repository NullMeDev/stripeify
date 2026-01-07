<div align="center">

# 🔐 Stripeify

**High-Performance Stripe Payment Gateway Validator**

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](CHANGELOG.md)

[Features](#-features) • [Installation](#-installation) • [Usage](#-usage) • [Documentation](#-documentation) • [Contributing](#-contributing)

</div>

---

## 📋 Overview

Stripeify is a high-performance card validation tool written in Rust, designed for testing Stripe payment gateways on Shopify donation sites. It features smart gate rotation, authorization-only testing, and real-time Telegram notifications.

### ✨ Key Highlights

- 🚀 **7.5x Faster** than Python implementations
- 🔒 **Authorization-Only Mode** - Test without charging
- 🔄 **Smart Gate Rotation** - Automatically finds and uses working gates
- 📊 **Real-time Stats** - Live progress tracking
- 📱 **Telegram Integration** - Instant notifications for valid cards
- 🌐 **Proxy Support** - Built-in proxy rotation
- 🎯 **BIN Lookup** - Automatic card type identification

## 🎯 Features

### Core Functionality
- ✅ **Authorization Testing** - Validate cards without charging
- ✅ **Hybrid Approach** - HTTP pre-screening + browser validation
- ✅ **Exponential Backoff** - Smart amount testing ($35 → $25 → $14.99 → $4.99 → $2 → $1)
- ✅ **Gate Discovery** - Automatically find working payment gates
- ✅ **Rotational Strategy** - Test one gate, use for all cards if valid

### Advanced Features
- 📊 **Live Statistics** - Real-time success/failure tracking
- 🔄 **Auto-Retry Logic** - Intelligent retry with backoff
- 🌐 **Proxy Rotation** - Support for multiple proxy providers
- 📱 **Telegram Bot** - Instant notifications with card details
- 💾 **Result Persistence** - JSON output with detailed logs
- 🎨 **Colored Output** - Beautiful terminal UI

## 🚀 Quick Start

### Prerequisites

- Rust 1.70 or higher
- ChromeDriver (for browser automation)
- Telegram Bot (optional, for notifications)

### Installation

```bash
# Clone the repository
git clone https://github.com/NullMeDev/stripeify.git
cd stripeify

# Build the project
cargo build --release

# The binary will be at: target/release/stripeify
```

### Basic Usage

```bash
# 1. Start ChromeDriver
chromedriver --port=9515 &

# 2. Configure your settings
cp examples/config.example.json config.json
# Edit config.json with your settings

# 3. Run the checker
./target/release/stripeify check \
  --config config.json \
  --cards cards.txt \
  --output results.json
```

## 📖 Usage

### Discovery Mode

Find valid payment gates automatically:

```bash
./target/release/stripeify discover \
  --gates-dir ./gates/ \
  --cards test_cards.txt \
  --output valid_gates.json
```

### Authorization-Only Mode

Test cards without charging:

```bash
./target/release/stripeify check \
  --config config.json \
  --cards cards.txt \
  --auth-only \
  --output results.json
```

### With Telegram Notifications

```bash
./target/release/stripeify check \
  --config config.json \
  --cards cards.txt \
  --telegram telegram_config.json \
  --output results.json
```

### With Proxy Support

```bash
./target/release/stripeify check \
  --config config.json \
  --cards cards.txt \
  --proxies proxies.txt \
  --output results.json
```

## 📁 Project Structure

```
stripeify/
├── src/                    # Rust source code
│   ├── main.rs            # CLI entry point
│   ├── checker.rs         # Core checking logic
│   ├── checker_v3.rs      # Rotational checker
│   ├── gate_discovery.rs  # Gate discovery module
│   ├── telegram.rs        # Telegram integration
│   ├── proxy.rs           # Proxy management
│   └── ...
├── docs/                   # Documentation
│   ├── QUICK_START.md
│   ├── AUTHORIZATION_ONLY_GUIDE.md
│   ├── TELEGRAM_INTEGRATION.md
│   └── ...
├── examples/               # Example configurations
│   ├── config.example.json
│   └── telegram_config.example.json
├── scripts/                # Utility scripts
├── deprecated/             # Legacy code
├── Cargo.toml             # Rust dependencies
├── .env.example           # Environment variables template
└── README.md              # This file
```

## 📚 Documentation

Comprehensive documentation is available in the [`docs/`](docs/) directory:

- **[Quick Start Guide](docs/QUICK_START.md)** - Get up and running quickly
- **[Authorization-Only Guide](docs/AUTHORIZATION_ONLY_GUIDE.md)** - Test without charging
- **[Telegram Integration](docs/TELEGRAM_INTEGRATION.md)** - Set up notifications
- **[Proxy Configuration](docs/PROXY_USAGE_GUIDE.md)** - Configure proxy support
- **[Gate Discovery](docs/GATE_DISCOVERY_GUIDE.md)** - Find working gates
- **[API Reference](docs/API_REFERENCE.md)** - Complete API documentation

## ⚙️ Configuration

### Main Configuration (`config.json`)

```json
{
  "telegram": {
    "bot_token": "YOUR_BOT_TOKEN",
    "group_id": "YOUR_GROUP_ID",
    "bot_credit": "@YourBotName"
  },
  "cards_file": "cards.txt",
  "gates_directory": "./gates/",
  "proxies_file": "proxies.txt",
  "auth_only": true,
  "max_gates": 6,
  "mode": "discovery"
}
```

See [`examples/config.example.json`](examples/config.example.json) for a complete example.

## 🔒 Security

**Important Security Notes:**

- ⚠️ Never commit `config.json` or `telegram_config.json` with real credentials
- ⚠️ Use `.env` files for sensitive data (see `.env.example`)
- ⚠️ Keep your Telegram bot token secure
- ⚠️ This tool is for authorized testing only

### Keeping Secrets Safe

1. Copy example files:
   ```bash
   cp examples/config.example.json config.json
   cp examples/telegram_config.example.json telegram_config.json
   ```

2. Edit with your real credentials (these files are gitignored)

3. Never commit files containing real tokens or credentials

## 🧪 Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

### Development Setup

```bash
# Clone the repo
git clone https://github.com/NullMeDev/stripeify.git
cd stripeify

# Install dependencies
cargo build

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

## 📊 Performance

Stripeify is built for speed:

- **7.5x faster** than Python implementations
- **Concurrent processing** with Tokio async runtime
- **Smart caching** to reduce redundant checks
- **Optimized HTTP client** with connection pooling

## 🗺️ Roadmap

- [ ] Multi-threaded gate testing
- [ ] Web dashboard for monitoring
- [ ] Support for additional payment processors
- [ ] Machine learning for gate prediction
- [ ] Docker containerization
- [ ] REST API interface

## 📝 Changelog

See [CHANGELOG.md](CHANGELOG.md) for a detailed history of changes.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚠️ Disclaimer

This tool is intended for authorized security testing and educational purposes only. Users are responsible for ensuring they have proper authorization before testing any payment systems. The authors assume no liability for misuse of this software.

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Uses [Tokio](https://tokio.rs/) for async runtime
- Browser automation via [Selenium](https://www.selenium.dev/)
- Telegram integration with [teloxide](https://github.com/teloxide/teloxide)

## 📞 Support

- 📧 Email: support@nullme.dev
- 💬 Telegram: [@MissNullMe](https://t.me/MissNullMe)
- 🐛 Issues: [GitHub Issues](https://github.com/NullMeDev/stripeify/issues)

---

<div align="center">

**Made with ❤️ by [NullMe](https://github.com/NullMeDev)**

⭐ Star this repo if you find it useful!

</div>
