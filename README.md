# KNX Smart Home Bridge for Tivoli Garten

A Rust-based bridge that connects your Tivoli Garten apartment's KNX smart home system to Apple HomeKit, Google Home, and Amazon Alexa.

**Stop using the Maneth Stiefel website!** Control your lights, blinds, and ventilation from your phone or voice commands.

---

## Features

✅ **100% Automatic** - No manual configuration
✅ **Auto-discovers** all devices (lights, blinds, ventilation, sensors)
✅ **Auto-detects** pages (works for any apartment size)
✅ **Auto-login** and session management
✅ **Works with any apartment** - just change credentials
✅ **Supports HomeKit, Google Home, Alexa** via Homebridge

---

## Quick Start

### Prerequisites

- Rust 1.75+ ([install here](https://rustup.rs))
- Chrome/Chromium browser
- Your Allthings login credentials
- Your apartment's port number (e.g., 7149)

### Installation

1. **Clone the repository:**
```bash
git clone https://github.com/YOUR_USERNAME/smarthome-tivoli.git
cd smarthome-tivoli
```

2. **Create `.env` file:**
```bash
cp .env.example .env
```

Edit `.env` with your credentials:
```env
SMARTHOME_USERNAME=your-email@example.com
SMARTHOME_PASSWORD=your-password
SMARTHOME_BASE_URL=https://tgs-smarthome.masti.ch:7149
```

3. **Run auto-discovery (first time only):**
```bash
cargo run --release -- --discover
```

This will:
- Auto-login with your credentials
- Auto-detect all pages in your apartment
- Extract all device commands
- Generate `device_mappings_auto.toml`

Then rename it:
```bash
mv device_mappings_auto.toml device_mappings.toml
```

4. **Start the bridge:**
```bash
cargo run --release
```

The bridge is now running on `http://localhost:8080`

---

## Connect to HomeKit

### Install Homebridge Plugin

```bash
cd homebridge-knx-bridge
npm install
npm link
```

### Configure Homebridge

Add to `~/.homebridge/config.json`:
```json
{
  "platforms": [
    {
      "platform": "KNXBridge",
      "name": "KNX Bridge",
      "bridgeUrl": "http://localhost:8080"
    }
  ]
}
```

### Start Homebridge

```bash
homebridge
```

### Add to Home App

Open the Home app on your iPhone and add the bridge using PIN: **031-45-154**

---

## Supported Devices

| Type | Support | Features |
|------|---------|----------|
| Lights | ✅ | On/Off |
| Dimmers | ✅ | On/Off, Brightness |
| Blinds | ✅ | Up/Down/Stop |
| Ventilation | ✅ | 3 speed levels |
| Temperature Sensors | ✅ | Read-only |
| Scenes | ✅ | Trigger |

---

## How It Works

1. **Auto-Login:** Bridge logs in using OAuth with your `.env` credentials
2. **Auto-Discovery:** Scans pages 01-99 until empty, extracts all devices
3. **Session Management:** Automatically refreshes session if it expires (401 error)
4. **HTTP API:** Exposes devices via REST API on port 8080
5. **Homebridge:** Connects to API and exposes devices to HomeKit

---

## Architecture

```
┌─────────────────────┐
│   Tivoli Garten     │
│   KNX System        │
│   (Enertex EibPC²)  │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│   Rust Bridge       │
│   - Auto-login      │
│   - Auto-discovery  │
│   - Session mgmt    │
│   - HTTP API :8080  │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│   Homebridge        │
│   (Node.js plugin)  │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│   Apple Home        │
│   Google Home       │
│   Amazon Alexa      │
└─────────────────────┘
```

---

## Multi-Apartment Setup

Each apartment only needs:

1. **Different `.env` file** with their credentials and port
2. **Run `--discover`** once to generate their device mappings

The code is 100% generic - no hardcoded values!

---

## Commands

### Discovery (first time only)
```bash
cargo run --release -- --discover
mv device_mappings_auto.toml device_mappings.toml
```

### Run Bridge
```bash
cargo run --release
```

### Build Release Binary
```bash
cargo build --release
./target/release/knx-homekit-bridge
```

---

## Files

| File | Description | Commit to Git? |
|------|-------------|----------------|
| `.env` | Your credentials (port, username, password) | ❌ NO |
| `.env.example` | Template for credentials | ✅ YES |
| `device_mappings.toml` | Your device commands | ❌ NO |
| `src/` | Rust source code | ✅ YES |
| `homebridge-knx-bridge/` | Homebridge plugin | ✅ YES |

---

## Troubleshooting

**Bridge won't start?**
- Check `.env` has correct credentials
- Make sure Chrome is installed
- Check port number is correct (7xxx)

**Devices not found?**
- Run `--discover` again
- Check you can login at https://tgs-smarthome.masti.ch:7xxx

**Homebridge not connecting?**
- Ensure bridge is running: `curl http://localhost:8080/health`
- Check `bridgeUrl` in Homebridge config

**Session expires?**
- Automatic! Bridge will re-login when it gets 401 error

---

## License

MIT

---

## Credits

Built for **Tivoli Garten residents** 🏢

Tech stack:
- [Rust](https://rust-lang.org) - High-performance bridge
- [Homebridge](https://homebridge.io) - HomeKit integration
- [Headless Chrome](https://github.com/rust-headless-chrome/rust-headless-chrome) - Auto-login
