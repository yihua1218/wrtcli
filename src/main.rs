use clap::{Parser, Subcommand};
use tracing_subscriber;
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
        /// Name of the device to get status from, or use --all
        name: Option<String>,
        /// Get status from all registered devices
        #[arg(long, conflicts_with = "name")]
        all: bool,
        /// Display raw values (KB, seconds) instead of human readable format
        #[arg(long)]
        raw: bool,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Reboot an OpenWrt device
    Reboot {
        /// Name of the device to reboot, or use --all
        name: Option<String>,
        /// Reboot all registered devices
        #[arg(long, conflicts_with = "name")]
        all: bool,
    },
    /// Backup commands for managing device backups
    Backup {
        #[command(subcommand)]
        command: BackupCommands,
    },
    /// Wi-Fi management commands
    Wifi {
        #[command(subcommand)]
        command: WifiCommands,
    },
    /// DHCP management commands
    Dhcp {
        #[command(subcommand)]
        command: DhcpCommands,
    },
    /// DNS management commands
    Dns {
        #[command(subcommand)]
        command: DnsCommands,
    },
    /// Firewall management commands
    Firewall {
        #[command(subcommand)]
        command: FirewallCommands,
    },
}

#[derive(Subcommand)]
enum BackupCommands {
    /// Create a new backup
    Create {
        /// Name of the device
        name: String,
        /// Optional description for the backup
        #[arg(long)]
        description: Option<String>,
    },
    /// List all backups for a device
    List {
        /// Name of the device
        name: String,
    },
    /// Show details of a specific backup
    Show {
        /// Name of the device
        name: String,
        /// ID of the backup to show
        backup_id: String,
    },
    /// Restore a backup
    Restore {
        /// Name of the device
        name: String,
        /// ID of the backup to restore
        backup_id: String,
    },
    /// Remove a backup
    Remove {
        /// Name of the device
        name: String,
        /// ID of the backup to remove
        backup_id: String,
    },
}

#[derive(Subcommand)]
enum WifiCommands {
    /// Get Wi-Fi status
    Status {
        /// Name of the device to get status from, or use --all
        name: Option<String>,
        /// Get Wi-Fi status from all registered devices
        #[arg(long, conflicts_with = "name")]
        all: bool,
    },
    /// Turn on a Wi-Fi interface
    On {
        /// Name of the device, or use --all
        name: Option<String>,
        /// Turn on Wi-Fi for all registered devices
        #[arg(long, conflicts_with = "name")]
        all: bool,
        /// Name of the Wi-Fi interface (e.g., radio0)
        interface: String,
    },
    /// Turn off a Wi-Fi interface
    Off {
        /// Name of the device, or use --all
        name: Option<String>,
        /// Turn off Wi-Fi for all registered devices
        #[arg(long, conflicts_with = "name")]
        all: bool,
        /// Name of the Wi-Fi interface (e.g., radio0)
        interface: String,
    },
}

#[derive(Subcommand)]
enum DhcpCommands {
    /// Get DHCP leases
    Leases {
        /// Name of the device to get leases from, or use --all
        name: Option<String>,
        /// Get DHCP leases from all registered devices
        #[arg(long, conflicts_with = "name")]
        all: bool,
    },
}

#[derive(Subcommand)]
enum DnsCommands {
    /// Show DNS settings
    Show {
        /// Name of the device to show settings from, or use --all
        name: Option<String>,
        /// Show DNS settings from all registered devices
        #[arg(long, conflicts_with = "name")]
        all: bool,
    },
}

#[derive(Subcommand)]
enum FirewallCommands {
    /// Show firewall status
    Status {
        /// Name of the device to get status from, or use --all
        name: Option<String>,
        /// Get firewall status from all registered devices
        #[arg(long, conflicts_with = "name")]
        all: bool,
    },
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { name, ip, user, password } => {
            commands::add_device(&name, &ip, &user, &password).await?;
        }
        Commands::List => {
            commands::list_devices().await?;
        }
        Commands::Status { name, all, raw, json } => {
            commands::get_status(name.as_deref(), all, raw, json).await?;
        }
        Commands::Reboot { name, all } => {
            commands::reboot_device(name.as_deref(), all).await?;
        }
        Commands::Backup { command } => {
            match command {
                BackupCommands::Create { name, description } => {
                    commands::create_backup(&name, description, false).await?;
                }
                BackupCommands::List { name } => {
                    commands::list_backups(&name).await?;
                }
                BackupCommands::Show { name, backup_id } => {
                    commands::show_backup(&name, &backup_id).await?;
                }
                BackupCommands::Restore { name, backup_id } => {
                    commands::restore_backup(&name, &backup_id, false).await?;
                }
                BackupCommands::Remove { name, backup_id } => {
                    commands::remove_backup(&name, &backup_id).await?;
                }
            }
        }
        Commands::Wifi { command } => match command {
            WifiCommands::Status { name, all } => {
                commands::get_wifi_status(name.as_deref(), all).await?;
            }
            WifiCommands::On { name, all, interface } => {
                commands::set_wifi_state(name.as_deref(), all, &interface, true).await?;
            }
            WifiCommands::Off { name, all, interface } => {
                commands::set_wifi_state(name.as_deref(), all, &interface, false).await?;
            }
        },
        Commands::Dhcp { command } => match command {
            DhcpCommands::Leases { name, all } => {
                commands::get_dhcp_leases(name.as_deref(), all).await?;
            }
        },
        Commands::Dns { command } => match command {
            DnsCommands::Show { name, all } => {
                commands::show_dns_settings(name.as_deref(), all).await?;
            }
        },
        Commands::Firewall { command } => match command {
            FirewallCommands::Status { name, all } => {
                commands::get_firewall_status(name.as_deref(), all).await?;
            }
        },
    }

    Ok(())
}