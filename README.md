# wrtcli

A CLI tool for managing OpenWrt devices, written in Rust.

## Features

✅ Unified CLI tool for remote OpenWrt management  
✅ Support for managing multiple OpenWrt devices  
✅ Access via Ubus JSON-RPC API, LuCI REST API or SSH  
✅ Lightweight, no additional packages required on OpenWrt  
✅ Highly extensible for future OpenWrt services

## Installation

```bash
# Clone the repository
git clone https://github.com/yihua1218/wrtcli.git
cd wrtcli

# Build the project
cargo build --release

# Optional: Install the binary
cargo install --path .
```

## Screenshots

![wrtcli execution example](docs/screenshots/wrtcli_execution.png)

The screenshot above demonstrates:
- Command-line interface with clear help information
- Device listing functionality showing registered devices
- Detailed status display including:
  - Device model and hostname
  - System uptime (formatted as days/hours/minutes/seconds)
  - System load
  - Memory usage statistics (with MB and usage percentage)

## Usage

### Device Management

```bash
# Add a new OpenWrt device
wrtcli add router1 --ip 192.168.1.1 --user root --password mypassword

# List all registered devices
wrtcli list

# Get device status (default: human readable format)
wrtcli status router1

# Get status with raw values (seconds for uptime, KB for memory)
wrtcli status router1 --raw

# Get status in JSON format
wrtcli status router1 --json

# Get status in JSON format with raw values
wrtcli status router1 --json --raw

# Reboot a device
wrtcli reboot router1
```

### Configuration

Configuration is stored in `~/.wrtcli/config.toml` and manages device information securely.

## Requirements

- Rust 1.70+
- OpenWrt 19.07+ (with Ubus API support)
- Linux / macOS / Windows

## Development

### Project Structure

```
src/
├── main.rs        # Entry point and CLI structure
├── commands.rs    # Command implementations
├── config.rs      # Configuration management
└── models.rs      # Data structures
```

### Building from Source

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Roadmap

- [x] Core device management
- [x] Basic status monitoring
- [ ] Wi-Fi management
- [ ] DHCP operations
- [ ] DNS management
- [ ] Firewall configuration
- [ ] Batch operations
