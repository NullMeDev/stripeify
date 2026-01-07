# Stripeify - Shopify Card Checker (Rust)

High-performance card validation tool for Shopify donation sites, written in Rust.

## 🚀 Quick Start

```bash
# 1. Start ChromeDriver
chromedriver --port=9515 &

# 2. Run checker with your cards
./target/release/shopify_checker rotate \
  --gates donation_gates.json \
  --cards-file 42000Dump.txt \
  --output results.json \
  --telegram-config telegram_config.json
```

## 📁 Project Structure

```
Stripeify/
├── src/                    # Rust source code
│   ├── main.rs            # CLI entry point
│   ├── checker_v3.rs      # Rotational checker
│   ├── analyzer.rs        # Gate analyzer
│   ├── telegram.rs        # Telegram notifications
│   └── ...
├── docs/                   # Documentation
│   ├── README.md          # Main documentation
│   ├── AUTHORIZATION_ONLY_GUIDE.md  # Authorization-only implementation
│   └── ...
├── deprecated/             # Old Python scripts and deprecated code
├── Cargo.toml             # Rust dependencies
├── donation_gates.json    # Donation sites database
├── production_gates.json  # Production gates
└── telegram_config.json   # Telegram bot configuration
```

## 📚 Documentation

All documentation is in the `docs/` folder:

- **[AUTHORIZATION_ONLY_GUIDE.md](docs/AUTHORIZATION_ONLY_GUIDE.md)** - How to check cards without charging
- **[QUICK_START.md](docs/QUICK_START.md)** - Getting started guide
- **[RUST_UNIFIED.md](docs/RUST_UNIFIED.md)** - Complete implementation guide
- **[ROTATIONAL_GATE_STRATEGY.md](docs/ROTATIONAL_GATE_STRATEGY.md)** - Smart gate rotation explained
- **[TELEGRAM_USAGE_GUIDE.md](docs/TELEGRAM_USAGE_GUIDE.md)** - Telegram integration

## 🎯 Features

- ✅ **Authorization-Only Mode** - Check cards without charging
- ✅ **Smart Gate Rotation** - Finds working gate, uses for all cards
- ✅ **Hybrid Approach** - HTTP pre-screening + browser validation
- ✅ **Exponential Backoff** - $35 → $25 → $14.99 → $4.99 → $2 → $1
- ✅ **Telegram Notifications** - Instant alerts for hits
- ✅ **BIN Lookup** - Card type identification
- ✅ **7.5x Faster** - Than Python implementation

## 🔧 Commands

### Analyze Gates
```bash
./target/release/shopify_checker analyze \
  --input ShopifyGatesAndChunks/ \
  --output donation_gates.json
```

### Test Cards (Rotational Mode)
```bash
./target/release/shopify_checker rotate \
  --gates donation_gates.json \
  --cards-file 42000Dump.txt \
  --output results.json
```

## 📊 Authorization vs Charging

The checker focuses on **authorization checks** rather than actual charges:

- **CVV_MISMATCH** - ✅ Card valid, CVV wrong (NO CHARGE)
- **INSUFFICIENT_FUNDS** - ✅ Card valid, no funds (NO CHARGE)
- **CHARGED** - ⚠️ Full authorization (MAY CHARGE)
- **DECLINED** - ❌ Card rejected

See [AUTHORIZATION_ONLY_GUIDE.md](docs/AUTHORIZATION_ONLY_GUIDE.md) for implementation details.

## 🛠️ Build from Source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build --release

# Binary at: target/release/shopify_checker
```

## 📝 License

For authorized testing only. Personal use.

## 🤝 Support

Check the `docs/` folder for detailed guides on every aspect of the tool.
