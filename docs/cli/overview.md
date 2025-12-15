# Pangolin CLI Tools Overview

Pangolin provides two command-line interface (CLI) tools to interact with the catalog ecosystem:

1.  **`pangolin-admin`**: For Root Admins and Tenant Admins to manage infrastructure, users, and tenants.
2.  **`pangolin-user`**: For data engineers and analysts to discover data, request access, and generate code.

## Architecture
Both tools are built on a shared core (`pangolin_cli_common`) and provide an interactive REPL (Read-Eval-Print Loop) experience.

## Configuration
Session configuration is stored in your user configuration directory:
- Linux: `~/.config/pangolin/config.json`
- macOS: `~/Library/Application Support/com.pangolin.cli/config.json`
- Windows: `C:\Users\<User>\AppData\Roaming\pangolin\cli\config.json`
