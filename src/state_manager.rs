use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::command_mapper::CommandMapper;
use crate::device::{Device, DeviceRegistry, DeviceState};
use crate::knx_client::KnxClient;

pub struct StateManager {
    registry: Arc<RwLock<DeviceRegistry>>,
    client: Arc<KnxClient>,
    pub command_mapper: Arc<CommandMapper>,
}

impl StateManager {
    pub fn new(
        client: Arc<KnxClient>,
        command_mapper: Arc<CommandMapper>,
    ) -> Self {
        Self {
            registry: Arc::new(RwLock::new(DeviceRegistry::new())),
            client,
            command_mapper,
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing state manager");
        let devices = self.client.discover_devices().await?;

        let mut registry = self.registry.write().await;
        for device in devices {
            let key = device.key();
            info!("Registered device: {} ({}) [key: {}]", device.name, device.id, key);
            registry.add(device);
        }

        info!("Initialized {} devices", registry.count());
        Ok(())
    }

    pub async fn get_device(&self, id: &str) -> Option<Device> {
        let registry = self.registry.read().await;
        registry.get(id).cloned()
    }

    pub async fn get_all_devices(&self) -> Vec<Device> {
        let registry = self.registry.read().await;
        registry.all().cloned().collect()
    }

    pub async fn toggle_device(&self, device_key: &str, target_state: bool) -> Result<()> {
        let current_state = {
            let registry = self.registry.read().await;
            registry.get(device_key).map(super::device::Device::is_on)
        };

        let Some(current) = current_state else {
                return Err(anyhow::anyhow!("Device not found: {device_key}"));
            };

        let (device_id, page) = {
            let registry = self.registry.read().await;
            let device = registry.get(device_key).ok_or_else(|| {
                anyhow::anyhow!("Device not found: {device_key}")
            })?;
            (device.id.clone(), device.page.clone())
        };

        if current == target_state {
            debug!(
                "Device {} [key: {}] already in desired state: {}",
                device_id, device_key, target_state
            );
        } else {
            let command = self.command_mapper.get_command(&device_id, &page).ok_or_else(|| {
                anyhow::anyhow!("No command mapping found for device: {device_id} (page: {page})")
            })?;

            info!(
                "Toggling device {} [key: {}] from {} to {}",
                device_id, device_key, current, target_state
            );

            self.client.send_command(command).await?;

            let mut registry = self.registry.write().await;
            if let Some(device) = registry.get_mut(device_key) {
                device.set_on(target_state);
            }
        }

        Ok(())
    }

    pub async fn set_blind_position(&self, device_key: &str, position: u8) -> Result<()> {
        let (device_id, page) = {
            let registry = self.registry.read().await;
            let device = registry.get(device_key).ok_or_else(|| {
                anyhow::anyhow!("Device not found: {device_key}")
            })?;
            (device.id.clone(), device.page.clone())
        };

        let command_suffix = if position <= 10 {
            "down"
        } else if position >= 90 {
            "up"
        } else {
            "stop"
        };

        let base_key = CommandMapper::device_key(&device_id, &page);
        let command_key = format!("{base_key}_{command_suffix}");

        let command = self.command_mapper.command_cache.get(&command_key).ok_or_else(|| {
            anyhow::anyhow!("No command mapping found for blind: {device_key} ({command_suffix})")
        })?;

        info!(
            "Setting blind {} [key: {}] to {}% (command: {})",
            device_id, device_key, position, command_suffix
        );

        self.client.send_command(command).await?;

        let mut registry = self.registry.write().await;
        if let Some(device) = registry.get_mut(device_key) {
            use crate::device::WindowCoveringState;
            let covering_state = if position <= 10 {
                WindowCoveringState::Closing
            } else if position >= 90 {
                WindowCoveringState::Opening
            } else {
                WindowCoveringState::Stopped
            };
            device.state = DeviceState::WindowCovering {
                position,
                state: covering_state,
            };
        }

        Ok(())
    }
}

