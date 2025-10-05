use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceMappings {
    #[serde(default)]
    pub lights: HashMap<String, String>,
    #[serde(default)]
    pub blinds: HashMap<String, String>,
    #[serde(default)]
    pub dimmers: HashMap<String, String>,
    #[serde(default)]
    pub ventilation: HashMap<String, String>,
    #[serde(default)]
    pub scenes: HashMap<String, String>,
    #[serde(default)]
    pub switches: HashMap<String, String>,
    #[serde(default)]
    pub sensors: HashMap<String, String>,
}

pub struct CommandMapper {
    mappings: DeviceMappings,
    pub command_cache: HashMap<String, String>,
}

impl CommandMapper {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = fs::read_to_string(path.as_ref())
            .context("Failed to read device mappings file")?;
        let mappings: DeviceMappings = toml::from_str(&contents)
            .context("Failed to parse device mappings")?;

        let mut command_cache = HashMap::new();
        command_cache.extend(mappings.lights.iter().map(|(k, v)| (k.clone(), v.clone())));
        command_cache.extend(mappings.blinds.iter().map(|(k, v)| (k.clone(), v.clone())));
        command_cache.extend(mappings.dimmers.iter().map(|(k, v)| (k.clone(), v.clone())));
        command_cache.extend(mappings.ventilation.iter().map(|(k, v)| (k.clone(), v.clone())));
        command_cache.extend(mappings.scenes.iter().map(|(k, v)| (k.clone(), v.clone())));
        command_cache.extend(mappings.switches.iter().map(|(k, v)| (k.clone(), v.clone())));
        command_cache.extend(mappings.sensors.iter().map(|(k, v)| (k.clone(), v.clone())));

        info!("Loaded {} total command mappings", command_cache.len());

        Ok(Self {
            mappings,
            command_cache,
        })
    }

    pub fn device_key(device_id: &str, page: &str) -> String {
        if device_id.contains("_page") {
            device_id.to_string()
        } else {
            format!("{}_page{}", device_id, page)
        }
    }

    pub fn get_command(&self, device_id: &str, page: &str) -> Option<&str> {
        let key = Self::device_key(device_id, page);

        if let Some(cmd) = self.command_cache.get(&key) {
            if cmd == "READONLY" {
                debug!("Device {} is read-only", key);
                None
            } else {
                Some(cmd.as_str())
            }
        } else {
            debug!("No command mapping found for device: {}", key);
            None
        }
    }

    pub fn get_blind_commands(&self, device_id: &str, page: &str) -> Option<BlindCommands> {
        let base_key = Self::device_key(device_id, page);

        let up = self.command_cache.get(&format!("{}_up", base_key))?;
        let stop = self.command_cache.get(&format!("{}_stop", base_key))?;
        let down = self.command_cache.get(&format!("{}_down", base_key))?;

        if up == "READONLY" || stop == "READONLY" || down == "READONLY" {
            return None;
        }

        Some(BlindCommands {
            up: up.clone(),
            stop: stop.clone(),
            down: down.clone(),
        })
    }

    pub fn is_readonly(&self, device_id: &str, page: &str) -> bool {
        let key = Self::device_key(device_id, page);
        self.command_cache.get(&key).map(|cmd| cmd == "READONLY").unwrap_or(false)
    }

    pub fn all_keys(&self) -> Vec<String> {
        self.command_cache.keys().cloned().collect()
    }
}

#[derive(Debug, Clone)]
pub struct BlindCommands {
    pub up: String,
    pub stop: String,
    pub down: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_key() {
        assert_eq!(
            CommandMapper::device_key("Single_1", "02"),
            "Single_1_page02"
        );
        assert_eq!(
            CommandMapper::device_key("Single_1_page02", "02"),
            "Single_1_page02"
        );
    }
}
