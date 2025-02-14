use clap::{Parser, Subcommand};
mod config;
mod models;
mod commands;

#[derive(Parser)]
#[command(name = "wrtcli")]
#[command(about = "OpenWrt CLI management tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new OpenWrt device
    Add {
        /// Name of the device
        name: String,
        /// IP address of the device
        #[arg(long)]
        ip: String,
        /// Username for authentication
        #[arg(long)]
        user: String,
        /// Password for authentication
        #[arg(long)]
        password: String,
    },
    /// List all registered devices
    List,
    /// Get status of an OpenWrt device
    Status {
        /// Name of the device
        name: String,
        /// Display raw values (KB, seconds) instead of human readable format
        #[arg(long)]
        raw: bool,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Reboot an OpenWrt device
    Reboot {
        /// Name of the device
        name: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { name, ip, user, password } => {
            commands::add_device(&name, &ip, &user, &password).await?;
        }
        Commands::List => {
            commands::list_devices().await?;
        }
        Commands::Status { name, raw, json } => {
            commands::get_status(&name, raw, json).await?;
        }
        Commands::Reboot { name } => {
            commands::reboot_device(&name).await?;
        }
    }

    Ok(())
}
