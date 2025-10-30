mod api_server;
mod auto_discovery;
mod command_mapper;
mod config;
mod device;
mod knx_client;
mod state_manager;

use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::command_mapper::CommandMapper;
use crate::config::Config;
use crate::knx_client::KnxClient;
use crate::state_manager::StateManager;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,knx_homekit_bridge=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();


    let args: Vec<String> = std::env::args().collect();
    let headless = args.contains(&"--headless".to_string());

    if args.contains(&"--discover".to_string()) {
        info!("🔍 Running in AUTO-DISCOVERY mode");
        info!("This will automatically find all device commands");
        if headless {
            info!("🤖 Headless mode: Chrome will run in background (no window)");
        } else {
            info!("🖥️  GUI mode: Chrome window will appear for manual login");
        }
        info!("");

        let discovery = auto_discovery::AutoDiscovery::new(headless)?;
        let pages = vec!["01".to_string(), "02".to_string(), "03".to_string(), "04".to_string()];

        discovery.discover_all_mappings(&pages).await?;

        info!("");
        info!("✅ Auto-discovery complete!");
        info!("Review device_mappings_auto.toml and rename to device_mappings.toml");
        return Ok(());
    }

    info!("Starting KNX-HomeKit Bridge");

    let config = Config::load_from_env().context("Failed to load configuration from .env")?;
    info!("Configuration loaded from .env");

    let command_mapper = Arc::new(
        CommandMapper::load("device_mappings.toml")
            .context("Failed to load device mappings")?
    );
    info!("Device mappings loaded successfully");

    let knx_config = Arc::new(config.knx.clone());
    let client = Arc::new(KnxClient::new(knx_config, headless)?);
    info!("KNX client initialized");
    if headless {
        info!("Running in headless mode (Chrome in background)");
    }

    client.ensure_valid_session().await?;

    let state_manager = Arc::new(StateManager::new(client.clone(), command_mapper.clone()));

    state_manager.initialize().await?;
    info!("Device discovery completed");

    let devices = state_manager.get_all_devices().await;
    info!("Discovered devices:");
    for device in &devices {
        info!(
            "  - {} ({}) - Type: {:?}, Page: {}, Index: {}",
            device.name, device.id, device.device_type, device.page, device.index
        );
    }

    info!("State polling: DISABLED (command-only mode)");

    let state_manager_api = state_manager.clone();
    let api_port = config.homekit.port;
    tokio::spawn(async move {
        if let Err(e) = api_server::start_api_server(state_manager_api, api_port).await {
            error!("API server failed: {}", e);
        }
    });

    info!("");
    info!("✅ KNX-HomeKit Bridge is running!");
    info!("   - KNX devices: {} discovered", devices.len());
    info!("   - Command mappings: {} loaded", command_mapper.command_cache.len());
    info!("   - HTTP API: http://localhost:{}", api_port);
    info!("");
    info!("📱 Connect Homebridge:");
    info!("   1. Install the homebridge-knx-bridge plugin");
    info!("   2. Configure bridge URL: http://localhost:{}", api_port);
    info!("   3. Add to Home app and pair");
    info!("");
    info!("Press Ctrl+C to exit.");

    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    Ok(())
}
