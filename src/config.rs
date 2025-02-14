use crate::models::{Config, Device};
use anyhow::{Context, Result};
use dirs;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::home_dir()
            .context("Could not find home directory")?
            .join(".wrtcli");
        
        fs::create_dir_all(&config_dir)?;
        
        Ok(Self {
            config_path: config_dir.join("config.toml"),
        })
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
