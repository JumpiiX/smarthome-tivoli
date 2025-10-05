# KNX Smart Home Bridge for Tivoli Garten

<p align="center">
  <a href="https://www.rust-lang.org/">
    <img src="https://img.shields.io/badge/built%20with-Rust-orange?style=flat&logo=rust" alt="Built with Rust" />
  </a>
  <a href="https://tokio.rs/">
    <img src="https://img.shields.io/badge/powered%20by-Tokio-blue?style=flat&logo=rust" alt="Powered by Tokio" />
  </a>
  <a href="https://github.com/tokio-rs/axum">
    <img src="https://img.shields.io/badge/web%20framework-Axum-00ADD8?style=flat" alt="Web Framework: Axum" />
  </a>
  <a href="https://homebridge.io/">
    <img src="https://img.shields.io/badge/homebridge-compatible-purple?style=flat&logo=homebridge" alt="Homebridge Compatible" />
  </a>
  <a href="https://www.apple.com/home-app/">
    <img src="https://img.shields.io/badge/Apple-HomeKit-black?style=flat&logo=apple" alt="Apple HomeKit" />
  </a>
  <br />
  <a href="https://nodejs.org/">
    <img src="https://img.shields.io/badge/node.js-v18+-green?style=flat&logo=node.js" alt="Node.js" />
  </a>
  <a href="https://www.npmjs.com/">
    <img src="https://img.shields.io/badge/npm-package-red?style=flat&logo=npm" alt="NPM Package" />
  </a>
  <a href="https://github.com/rust-headless-chrome/rust-headless-chrome">
    <img src="https://img.shields.io/badge/automation-headless%20chrome-4285F4?style=flat&logo=googlechrome" alt="Headless Chrome" />
  </a>
  <a href="https://en.wikipedia.org/wiki/KNX_(standard)">
    <img src="https://img.shields.io/badge/protocol-KNX-green?style=flat" alt="KNX Protocol" />
  </a>
  <br />
  <br />
</p>

A high-performance Rust bridge that connects Tivoli Garten's KNX smart home system (Enertex EibPC²) to Apple HomeKit, Google Home, and Amazon Alexa via Homebridge.

**What it does:**
- Automatically logs into your Tivoli Garten smart home system
- Discovers all devices across all pages (lights, blinds, ventilation, sensors)
- Exposes a REST API for device control
- Provides a Homebridge plugin for HomeKit/Google/Alexa integration

**Technologies:**
- **Rust** - High-performance async bridge with Tokio runtime
- **Axum** - Modern web framework for REST API
- **Headless Chrome** - Automated OAuth login and device discovery
- **Node.js** - Homebridge plugin for smart home platforms
- **KNX Protocol** - Industry-standard building automation

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

Built for **Tivoli Garten residents** 🏢
