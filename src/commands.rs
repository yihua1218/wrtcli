use crate::config::ConfigManager;
use crate::models::Device;
use anyhow::{Context, Result};
use reqwest::{Client, multipart};
use serde_json::json;
use serde::Serialize;
use std::time::Duration;
use std::fs;
use std::io::{Read, Write};
use tempfile::NamedTempFile;
use tar::Builder;
use flate2::{write::GzEncoder, Compression};
use tracing::{debug};

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

// Helper function to get LuCI session token
async fn get_luci_session(client: &Client, device: &Device) -> Result<String> {
    let response = client
        .post(&format!("{}/cgi-bin/luci/rpc/auth", device.luci_url()))
        .form(&[
            ("username", &device.user),
            ("password", &device.password),
        ])
        .send()
        .await?;

    let data = response.json::<serde_json::Value>().await?;
    data["result"]
        .as_str()
        .context("Failed to get LuCI session token")
        .map(|s| s.to_string())
}

pub async fn create_backup(name: &str, description: Option<String>, use_ubus: bool) -> Result<()> {
    debug!("Starting create_backup for device: {}", name);
    let config = ConfigManager::new()?;
    let device = config
        .get_device(name)?
        .context(format!("Device '{}' not found", name))?;
    debug!("Device found: {:?}", device);

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;
    debug!("HTTP client created");

    let temp_file = NamedTempFile::new()?;
    let backup_info;

    if use_ubus {
        debug!("Using UBUS for backup");
        // Original UBUS-based backup implementation
        let encoder = GzEncoder::new(temp_file.reopen()?, Compression::default());
        let mut archive = Builder::new(encoder);
        let tcp = tokio::net::TcpStream::connect(&format!("{}:22", device.ip)).await?;
        let tcp = tcp.into_std()?;
        let mut sess = ssh2::Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;
        sess.userauth_password(&device.user, &device.password)?;
        debug!("SSH session established");

        let config_files = vec![
            "system", "wireless", "network", "dhcp", "firewall"
        ];

        for config_name in config_files {
            debug!("Backing up config: {}", config_name);
            // Try to read and backup each config file
            if let Ok(mut channel) = sess.channel_session() {
                channel.exec(&format!("cat /etc/config/{}", config_name))?;
                let mut content = String::new();
                channel.read_to_string(&mut content)?;
                channel.wait_close()?;

                if channel.exit_status()? == 0 && !content.is_empty() {
                    println!("✅ Backing up config: {}", config_name);
                    let mut header = tar::Header::new_gnu();
                    header.set_size(content.len() as u64);
                    header.set_mode(0o644);
                    archive.append_data(&mut header, format!("etc/config/{}", config_name), content.as_bytes())?;
                } else {
                    println!("❌ Failed to read config: {}", config_name);
                }
            }
        }

        // Get system info from board.json
        debug!("Getting system info from board.json");
        if let Ok(mut channel) = sess.channel_session() {
            channel.exec("cat /etc/board.json")?;
            let mut content = String::new();
            channel.read_to_string(&mut content)?;
            channel.wait_close()?;

            if channel.exit_status()? == 0 && !content.is_empty() {
                println!("✅ Backing up system info");
                let mut header = tar::Header::new_gnu();
                header.set_size(content.len() as u64);
                header.set_mode(0o644);
                archive.append_data(&mut header, "system_info.json", content.as_bytes())?;
            } else {
                println!("❌ Failed to read system info");
            }
        }

        // Make sure to finish the archive before dropping it
        archive.finish()?;
        debug!("Archive finished");

        // Make sure the file is complete before moving it
        temp_file.as_file().sync_all()?;
        debug!("Temporary file sync complete");

        backup_info = config.add_backup(
            name,
            description,
            temp_file.path().to_path_buf(),
            "ubus".to_string(),
        )?;
    } else {
        debug!("Using LuCI API for backup");
        // LuCI API backup implementation
        let session = get_luci_session(&client, &device).await?;
        debug!("Obtained LuCI session token: {}", session);
        
        let response = client
            .get(&format!("{}/cgi-bin/luci/admin/system/flashops/backup", device.luci_url()))
            .header("Cookie", format!("sysauth={}", session))
            .send()
            .await?;
        debug!("Backup request sent, status: {}", response.status());

        let response_text = response.text().await?;
        debug!("Backup response body: {}", response_text);

        let content = response_text.into_bytes();
        debug!("Backup content received, size: {} bytes", content.len());

        let mut file = temp_file.reopen()?;
        file.write_all(&content)?;
        file.sync_all()?;
        debug!("Backup content written to temporary file");

        backup_info = config.add_backup(
            name,
            description,
            temp_file.path().to_path_buf(),
            "luci".to_string(),
        )?;
        debug!("Backup information saved to config");
    }
    
    debug!("Backup created successfully");
    println!("ID: {}", backup_info.id);
    println!("Filename: {}", backup_info.filename);
    println!("Created: {}", backup_info.created_at.format("%Y-%m-%d %H:%M:%S"));
    if let Some(desc) = backup_info.description {
        println!("Description: {}", desc);
    }
    println!("Size: {:.2} MB", backup_info.size as f64 / (1024.0 * 1024.0));
    
    Ok(())
}

pub async fn list_backups(name: &str) -> Result<()> {
    let config = ConfigManager::new()?;
    let meta = config.load_backup_meta(name)?;

    if meta.backups.is_empty() {
        println!("No backups found for device '{}'", name);
        return Ok(());
    }

    println!("Backups for device '{}':", name);
    println!("------------------------");
    for backup in meta.backups.iter() {
        println!("📦 {}", backup.id);
        println!("   Created: {}", backup.created_at.format("%Y-%m-%d %H:%M:%S"));
        if let Some(desc) = &backup.description {
            println!("   Description: {}", desc);
        }
        println!("   Size: {:.2} MB", backup.size as f64 / (1024.0 * 1024.0));
        println!();
    }

    Ok(())
}

pub async fn show_backup(name: &str, backup_id: &str) -> Result<()> {
    let config = ConfigManager::new()?;
    let meta = config.load_backup_meta(name)?;

    if let Some(backup) = meta.get_backup(backup_id) {
        println!("Backup details:");
        println!("--------------");
        println!("ID: {}", backup.id);
        println!("Device: {}", backup.device_name);
        println!("Created: {}", backup.created_at.format("%Y-%m-%d %H:%M:%S"));
        println!("Type: {}", backup.backup_type);
        println!("Size: {:.2} MB", backup.size as f64 / (1024.0 * 1024.0));
        if let Some(desc) = &backup.description {
            println!("Description: {}", desc);
        }
    } else {
        anyhow::bail!("Backup '{}' not found for device '{}'", backup_id, name);
    }

    Ok(())
}

pub async fn remove_backup(name: &str, backup_id: &str) -> Result<()> {
    let config = ConfigManager::new()?;
    config.remove_backup_file(name, backup_id)?;
    println!("✅ Backup '{}' removed successfully", backup_id);
    Ok(())
}

pub async fn restore_backup(name: &str, backup_id: &str, use_ubus: bool) -> Result<()> {
    let config = ConfigManager::new()?;
    let device = config
        .get_device(name)?
        .context(format!("Device '{}' not found", name))?;

    let meta = config.load_backup_meta(name)?;
    let backup = meta.get_backup(backup_id)
        .context(format!("Backup '{}' not found for device '{}'", backup_id, name))?;

    let client = Client::builder()
        .timeout(Duration::from_secs(30))  // Longer timeout for restore
        .build()?;

    // Read backup file
    let backup_path = config.get_backup_dir(name)?.join(&backup.filename);
    let backup_data = fs::read(&backup_path)?;

    if use_ubus {
        // Original UBUS-based restore implementation
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

        // Send restore command with backup data
        client
            .post(&device.ubus_url())
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "call",
                "params": [
                    session,
                    "system",
                    "restore",
                    {
                        "backup": backup_data
                    }
                ]
            }))
            .send()
            .await?;
    } else {
        // LuCI API restore implementation
        let session = get_luci_session(&client, &device).await?;
        
        let form = multipart::Form::new()
            .part("archive", multipart::Part::bytes(backup_data));

        client
            .post(&format!("{}/cgi-bin/luci/admin/system/flashops/restore", device.luci_url()))
            .header("Cookie", format!("sysauth={}", session))
            .multipart(form)
            .send()
            .await?;

        // Trigger reboot after restore
        client
            .post(&format!("{}/cgi-bin/luci/admin/system/reboot", device.luci_url()))
            .header("Cookie", format!("sysauth={}", session))
            .send()
            .await?;
    }

    println!("✅ Backup '{}' restored successfully to device '{}'", backup_id, name);
    println!("ℹ️  The device will reboot to apply the restored configuration");
    
    Ok(())
}
