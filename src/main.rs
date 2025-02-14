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
    /// Backup commands for managing device backups
    Backup {
        #[command(subcommand)]
        command: BackupCommands,
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
    }

    Ok(())
}
