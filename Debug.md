
# Debugging Instructions

To enable and view debug messages in the `wrtcli` application, follow these steps:

## Enable Debug Logging

Set the `RUST_LOG` environment variable to `debug` before running the application. This will enable debug-level logging.

### Example Command

```sh
RUST_LOG=debug cargo run backup create router1
```

## Viewing Debug Messages

When you run the application with the `RUST_LOG=debug` environment variable, you will see detailed debug messages in the console output. These messages can help you trace the execution flow and identify where errors are occurring.

### Example Output

```sh
$ RUST_LOG=debug cargo run backup create router1
   Compiling wrtcli v0.1.0 (/Users/yihua/wrtcli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.71s
     Running `target/debug/wrtcli backup create router1`
[2023-10-10T10:00:00Z DEBUG wrtcli::commands] Starting create_backup for device: router1
[2023-10-10T10:00:00Z DEBUG wrtcli::commands] Device found: Device { name: "router1", ip: "192.168.1.1", user: "admin", password: "password" }
[2023-10-10T10:00:00Z DEBUG wrtcli::commands] HTTP client created
[2023-10-10T10:00:00Z DEBUG wrtcli::commands] Using LuCI API for backup
[2023-10-10T10:00:00Z DEBUG wrtcli::commands] Obtained LuCI session token: abcdef1234567890
[2023-10-10T10:00:00Z DEBUG wrtcli::commands] Backup request sent, status: 200 OK
[2023-10-10T10:00:00Z DEBUG wrtcli::commands] Backup response body: {"result": "success", "data": "backup data..."}
[2023-10-10T10:00:00Z DEBUG wrtcli::commands] Backup content received, size: 1024 bytes
[2023-10-10T10:00:00Z DEBUG wrtcli::commands] Backup content written to temporary file
[2023-10-10T10:00:00Z DEBUG wrtcli::commands] Backup information saved to config
[2023-10-10T10:00:00Z DEBUG wrtcli::commands] Backup created successfully
ID: 1
Filename: backup_2023-10-10.tar.gz
Created: 2023-10-10 10:00:00
Description: None
Size: 1.00 MB
```

## Additional Debugging Tips

- Ensure that the `tracing` crate is properly initialized in your main function.
- Add `debug!` statements before and after significant operations to log important information.
- Check the status and body of HTTP responses to identify issues with network requests.

By following these instructions, you can enable and view detailed debug messages to assist with troubleshooting and debugging the `wrtcli` application.
