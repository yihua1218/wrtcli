use crate::config::ConfigManager;
use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::{json, Value};
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize)]
struct StatusOutput {
    device_name: String,
    model: String,
    hostname: String,
    uptime: UptimeInfo,
    load: f64,
    memory: MemoryInfo,
}

#[derive(Serialize)]
struct UptimeInfo {
    raw_seconds: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    formatted: Option<String>,
}

#[derive(Serialize)]
struct MemoryInfo {
    total_kb: u64,
    free_kb: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_mb: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    free_mb: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    used_percentage: Option<f64>,
}

// Helper function to format uptime into a human readable format
fn format_uptime(seconds: u64) -> String {
    let days = seconds / (24 * 3600);
    let hours = (seconds % (24 * 3600)) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{} days", days));
    }
    if hours > 0 {
        parts.push(format!("{} hours", hours));
    }
    if minutes > 0 {
        parts.push(format!("{} minutes", minutes));
    }
    if secs > 0 || parts.is_empty() {
        parts.push(format!("{} seconds", secs));
    }

    parts.join(", ")
}

// Helper function to format memory into human readable format
fn format_memory(total_kb: u64, free_kb: u64) -> (f64, f64, f64) {
    let total_mb = total_kb as f64 / 1024.0;
    let free_mb = free_kb as f64 / 1024.0;
    let used_percentage = ((total_kb - free_kb) as f64 / total_kb as f64) * 100.0;
    
    (total_mb, free_mb, used_percentage)
}

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

pub async fn get_status(name: &str, raw: bool, json_output: bool) -> Result<()> {
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

    let uptime = system_info["uptime"].as_u64().unwrap_or(0);
    let total_memory = system_info["memory"]["total"].as_u64().unwrap_or(0);
    let free_memory = system_info["memory"]["free"].as_u64().unwrap_or(0);
    let load = system_info["load"][0].as_f64().unwrap_or(0.0);
    let model = board_info["model"].as_str().unwrap_or("Unknown").to_string();
    let hostname = board_info["hostname"].as_str().unwrap_or("Unknown").to_string();

    if json_output {
        let (total_mb, free_mb, used_percentage) = if !raw {
            let (t, f, u) = format_memory(total_memory, free_memory);
            (Some(t), Some(f), Some(u))
        } else {
            (None, None, None)
        };

        let status = StatusOutput {
            device_name: name.to_string(),
            model,
            hostname,
            uptime: UptimeInfo {
                raw_seconds: uptime,
                formatted: if !raw { Some(format_uptime(uptime)) } else { None },
            },
            load,
            memory: MemoryInfo {
                total_kb: total_memory,
                free_kb: free_memory,
                total_mb,
                free_mb,
                used_percentage,
            },
        };

        println!("{}", serde_json::to_string_pretty(&status)?);
    } else {
        println!("Device Status: {}", device.name);
        println!("----------------");
        println!("📍 Model: {}", model);
        println!("🏷️  Hostname: {}", hostname);

        if raw {
            println!("⏰ Uptime: {} seconds", uptime);
            println!("🔄 Load: {:.2}", load);
            println!("💾 Memory:");
            println!("   Total: {} KB", total_memory);
            println!("   Free: {} KB", free_memory);
        } else {
            println!("⏰ Uptime: {}", format_uptime(uptime));
            println!("🔄 Load: {:.2}", load);
            
            let (total_mb, free_mb, used_percentage) = format_memory(total_memory, free_memory);
            println!("💾 Memory:");
            println!("   Total: {:.1} MB", total_mb);
            println!("   Free: {:.1} MB", free_mb);
            println!("   Used: {:.1}%", used_percentage);
        }
    }

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
