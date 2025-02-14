use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub name: String,
    pub ip: String,
    pub user: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub devices: HashMap<String, Device>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub hostname: String,
    pub model: String,
    pub uptime: u64,
    pub load: Vec<f64>,
    pub memory: MemoryStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatus {
    pub total: u64,
    pub free: u64,
    pub buffered: u64,
    pub cached: u64,
}

impl Config {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }

    pub fn add_device(&mut self, device: Device) {
        self.devices.insert(device.name.clone(), device);
    }

    pub fn get_device(&self, name: &str) -> Option<&Device> {
        self.devices.get(name)
    }
}

impl Device {
    pub fn new(name: String, ip: String, user: String, password: String) -> Self {
        Self {
            name,
            ip,
            user,
            password,
        }
    }

    pub fn ubus_url(&self) -> String {
        format!("http://{}/ubus", self.ip)
    }
}
