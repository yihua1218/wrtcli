use crate::models::{Config, Device, BackupMeta, BackupInfo};
use anyhow::{Context, Result};
use dirs;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use chrono::Local;

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::home_dir()
            .context("Could not find home directory")?
            .join(".wrtcli");
        
        fs::create_dir_all(&config_dir)?;
        fs::create_dir_all(config_dir.join("backups"))?;
        
        Ok(Self {
            config_path: config_dir.join("config.toml"),
        })
    }

    pub fn get_backup_dir(&self, device_name: &str) -> Result<PathBuf> {
        let backup_dir = self.config_path.parent().unwrap()
            .join("backups")
            .join(device_name);
        fs::create_dir_all(&backup_dir)?;
        Ok(backup_dir)
    }

    pub fn load_backup_meta(&self, device_name: &str) -> Result<BackupMeta> {
        let meta_path = self.get_backup_dir(device_name)?.join("metadata.json");
        
        if !meta_path.exists() {
            return Ok(BackupMeta::new());
        }

        let content = fs::read_to_string(&meta_path)
            .context("Failed to read backup metadata file")?;
        
        serde_json::from_str(&content)
            .context("Failed to parse backup metadata")
    }

    pub fn save_backup_meta(&self, device_name: &str, meta: &BackupMeta) -> Result<()> {
        let meta_path = self.get_backup_dir(device_name)?.join("metadata.json");
        let content = serde_json::to_string_pretty(meta)
            .context("Failed to serialize backup metadata")?;
        
        let mut file = File::create(&meta_path)
            .context("Failed to create backup metadata file")?;
        
        file.write_all(content.as_bytes())
            .context("Failed to write backup metadata file")?;
        
        Ok(())
    }

    pub fn add_backup(&self, device_name: &str, description: Option<String>, backup_path: PathBuf, backup_method: String) -> Result<BackupInfo> {
        let mut meta = self.load_backup_meta(device_name)?;
        let timestamp = Local::now();
        let id = timestamp.format("%Y%m%d_%H%M%S").to_string();
        let filename = format!("{}_full_backup.tar.gz", &id);
        
        let backup_info = BackupInfo {
            id,
            filename: filename.clone(),
            created_at: timestamp,
            device_name: device_name.to_string(),
            description,
            backup_type: "full".to_string(),
            backup_method,
            size: fs::metadata(&backup_path)?.len(),
        };

        // Move backup file to storage location
        let dest_path = self.get_backup_dir(device_name)?.join(&filename);
        fs::rename(backup_path, dest_path)?;

        meta.add_backup(backup_info.clone());
        self.save_backup_meta(device_name, &meta)?;

        Ok(backup_info)
    }

    pub fn load_config(&self) -> Result<Config> {
        if !self.config_path.exists() {
            return Ok(Config::new());
        }

        let content = fs::read_to_string(&self.config_path)
            .context("Failed to read config file")?;
        
        toml::from_str(&content)
            .context("Failed to parse config file")
    }

    pub fn save_config(&self, config: &Config) -> Result<()> {
        let content = toml::to_string_pretty(config)
            .context("Failed to serialize config")?;
        
        let mut file = File::create(&self.config_path)
            .context("Failed to create config file")?;
        
        file.write_all(content.as_bytes())
            .context("Failed to write config file")?;
        
        Ok(())
    }

    pub fn remove_backup_file(&self, device_name: &str, backup_id: &str) -> Result<()> {
        let mut meta = self.load_backup_meta(device_name)?;
        
        if let Some(backup) = meta.get_backup(backup_id) {
            let backup_path = self.get_backup_dir(device_name)?.join(&backup.filename);
            fs::remove_file(&backup_path)
                .context(format!("Failed to remove backup file: {}", backup_path.display()))?;
            
            meta.remove_backup(backup_id);
            self.save_backup_meta(device_name, &meta)?;
            Ok(())
        } else {
            anyhow::bail!("Backup '{}' not found for device '{}'", backup_id, device_name)
        }
    }

    pub fn add_device(
        &self,
        name: &str,
        ip: &str,
        user: &str,
        password: &str,
    ) -> Result<()> {
        let mut config = self.load_config()?;
        
        let device = Device::new(
            name.to_string(),
            ip.to_string(),
            user.to_string(),
            password.to_string(),
        );
        
        config.add_device(device);
        self.save_config(&config)?;
        
        Ok(())
    }

    pub fn get_device(&self, name: &str) -> Result<Option<Device>> {
        let config = self.load_config()?;
        Ok(config.get_device(name).cloned())
    }

    pub fn get_all_devices(&self) -> Result<Vec<Device>> {
        let config = self.load_config()?;
        Ok(config.devices.values().cloned().collect())
    }
}
