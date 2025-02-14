use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Local};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub id: String,
    pub filename: String,
    pub created_at: DateTime<Local>,
    pub device_name: String,
    pub description: Option<String>,
    pub backup_type: String,
    pub backup_method: String,  // "luci" or "ubus"
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMeta {
    pub backups: Vec<BackupInfo>,
}

impl BackupMeta {
    pub fn new() -> Self {
        Self {
            backups: Vec::new(),
        }
    }

    pub fn add_backup(&mut self, backup: BackupInfo) {
        self.backups.push(backup);
    }

    pub fn get_backup(&self, id: &str) -> Option<&BackupInfo> {
        self.backups.iter().find(|b| b.id == id)
    }

    pub fn remove_backup(&mut self, id: &str) -> bool {
        if let Some(pos) = self.backups.iter().position(|b| b.id == id) {
            self.backups.remove(pos);
            true
        } else {
            false
        }
    }
}

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

    pub fn luci_url(&self) -> String {
        format!("http://{}", self.ip)
    }
}
