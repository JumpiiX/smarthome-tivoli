use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub page: String,
    pub index: String,
    pub state: DeviceState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeviceType {
    Light,
    Dimmer,
    WindowCovering,
    TemperatureSensor,
    Fan,
    Scene,
    Switch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceState {
    OnOff(bool),
    Brightness { on: bool, level: u8 },
    WindowCovering { position: u8, state: WindowCoveringState },
    Temperature(f32),
    FanSpeed(u8),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WindowCoveringState {
    Stopped,
    Opening,
    Closing,
}

impl Device {
    pub fn key(&self) -> String {
        crate::command_mapper::CommandMapper::device_key(&self.id, &self.page)
    }

    pub fn new(id: String, name: String, device_type: DeviceType, page: String, index: String) -> Self {
        let state = match device_type {
            DeviceType::Light | DeviceType::Switch | DeviceType::Scene | DeviceType::Fan => {
                DeviceState::OnOff(false)
            }
            DeviceType::Dimmer => DeviceState::Brightness { on: false, level: 0 },
            DeviceType::WindowCovering => DeviceState::WindowCovering {
                position: 0,
                state: WindowCoveringState::Stopped,
            },
            DeviceType::TemperatureSensor => DeviceState::Temperature(0.0),
        };

        Device {
            id,
            name,
            device_type,
            page,
            index,
            state,
        }
    }

    pub fn is_on(&self) -> bool {
        match &self.state {
            DeviceState::OnOff(on) => *on,
            DeviceState::Brightness { on, .. } => *on,
            _ => false,
        }
    }

    pub fn set_on(&mut self, value: bool) {
        match &mut self.state {
            DeviceState::OnOff(on) => *on = value,
            DeviceState::Brightness { on, .. } => *on = value,
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeviceRegistry {
    devices: HashMap<String, Device>,
}

impl DeviceRegistry {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }

    pub fn add(&mut self, device: Device) {
        let key = device.key();
        self.devices.insert(key, device);
    }

    pub fn get(&self, key: &str) -> Option<&Device> {
        self.devices.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Device> {
        self.devices.get_mut(key)
    }

    #[allow(dead_code)]
    pub fn get_by_id_page(&self, id: &str, page: &str) -> Option<&Device> {
        let key = crate::command_mapper::CommandMapper::device_key(id, page);
        self.devices.get(&key)
    }

    #[allow(dead_code)]
    pub fn get_mut_by_id_page(&mut self, id: &str, page: &str) -> Option<&mut Device> {
        let key = crate::command_mapper::CommandMapper::device_key(id, page);
        self.devices.get_mut(&key)
    }

    pub fn all(&self) -> impl Iterator<Item = &Device> {
        self.devices.values()
    }

    #[allow(dead_code)]
    pub fn all_mut(&mut self) -> impl Iterator<Item = &mut Device> {
        self.devices.values_mut()
    }

    pub fn count(&self) -> usize {
        self.devices.len()
    }
}

impl Default for DeviceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
