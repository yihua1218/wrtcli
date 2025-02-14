use crate::config::ConfigManager;
use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;
use std::time::Duration;

pub async fn add_device(name: &str, ip: &str, user: &str, password: &str) -> Result<()> {
    let config = ConfigManager::new()?;
    config.add_device(name, ip, user, password)?;
    println!("✅ Device '{}' added successfully", name);
    Ok(())
}

pub async fn list_devices() -> Result<()> {
    let config = ConfigManager::new()?;
    let devices = config.get_all_devices()?;

    if devices.is_empty() {
        println!("No devices registered. Use 'wrtcli add' to add a device.");
        return Ok(());
    }

    println!("Registered OpenWrt devices:");
    println!("---------------------------");
    for device in devices {
        println!("📱 {} ({})", device.name, device.ip);
    }

    Ok(())
}

pub async fn get_status(name: &str) -> Result<()> {
    let config = ConfigManager::new()?;
    let device = config
        .get_device(name)?
        .context(format!("Device '{}' not found", name))?;

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    // Call ubus session login first
    let login_response = client
        .post(&device.ubus_url())
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "call",
            "params": [
                "00000000000000000000000000000000",
                "session",
                "login",
                {
                    "username": device.user,
                    "password": device.password
                }
            ]
        }))
        .send()
        .await?;

    let login_data = login_response.json::<serde_json::Value>().await?;
    let session = login_data["result"][1]["ubus_rpc_session"]
        .as_str()
        .context("Failed to get session token")?;

    // Get system info
    let system_response = client
        .post(&device.ubus_url())
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "call",
            "params": [
                session,
                "system",
                "board",
                {}
            ]
        }))
        .send()
        .await?;

    let system_data = system_response.json::<serde_json::Value>().await?;
    let board_info = &system_data["result"][1];

    // Get system status
    let status_response = client
        .post(&device.ubus_url())
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "call",
            "params": [
                session,
                "system",
                "info",
                {}
            ]
        }))
        .send()
        .await?;

    let status_data = status_response.json::<serde_json::Value>().await?;
    let system_info = &status_data["result"][1];

    println!("Device Status: {}", device.name);
    println!("----------------");
    println!("📍 Model: {}", board_info["model"].as_str().unwrap_or("Unknown"));
    println!("🏷️  Hostname: {}", board_info["hostname"].as_str().unwrap_or("Unknown"));
    println!("⏰ Uptime: {} seconds", system_info["uptime"].as_u64().unwrap_or(0));
    println!("🔄 Load: {:.2}", system_info["load"][0].as_f64().unwrap_or(0.0));
    println!("💾 Memory:");
    println!("   Total: {} KB", system_info["memory"]["total"].as_u64().unwrap_or(0));
    println!("   Free: {} KB", system_info["memory"]["free"].as_u64().unwrap_or(0));

    Ok(())
}

pub async fn reboot_device(name: &str) -> Result<()> {
    let config = ConfigManager::new()?;
    let device = config
        .get_device(name)?
        .context(format!("Device '{}' not found", name))?;

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    // Login first
    let login_response = client
        .post(&device.ubus_url())
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "call",
            "params": [
                "00000000000000000000000000000000",
                "session",
                "login",
                {
                    "username": device.user,
                    "password": device.password
                }
            ]
        }))
        .send()
        .await?;

    let login_data = login_response.json::<serde_json::Value>().await?;
    let session = login_data["result"][1]["ubus_rpc_session"]
        .as_str()
        .context("Failed to get session token")?;

    // Send reboot command
    client
        .post(&device.ubus_url())
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "call",
            "params": [
                session,
                "system",
                "reboot",
                {}
            ]
        }))
        .send()
        .await?;

    println!("🔄 Rebooting device '{}'...", name);
    Ok(())
}
