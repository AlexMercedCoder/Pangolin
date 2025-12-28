# Pangolin CLI Tools Overview

Pangolin provides two command-line interface (CLI) tools designed for different operational personas. Both tools feature a synchronized **REPL (Interactive)** mode and a **Non-Interactive** mode for scripting.

---

## 🛠️ The Tools

### 1. `pangolin-admin` (The Platform Tool)
Targeted at Root and Tenant Admins for infrastructure and identity management.

**Core Commands:**
| Command | Usage | Role |
| :--- | :--- | :--- |
| `list-tenants` | Manage platform organizations and isolation. | Root |
| `create-user` | Onboard Tenant Admins or standard Users. | Root / Admin |
| `create-warehouse`| Connect S3/Azure/GCS storage providers. | Admin |
| `create-catalog` | Provision Iceberg or Federated catalogs. | Admin |
| `grant-permission`| Assign RBAC/TBAC roles and access levels. | Admin |
| `list-audit-events`| Comprehensive forensic and security tracking. | Admin |
| `create-service-user`| Manage machine-to-machine API identities. | Admin |
| `update-system-settings`| Configure global platform defaults. | Root |

### 2. `pangolin-user` (The Data Tool)
Targeted at Data Engineers and Analysts for discovery and versioning.

**Core Commands:**
| Command | Usage | Role |
| :--- | :--- | :--- |
| `search` | Discover tables/views via metadata engines. | All Users |
| `create-branch` | Fork catalogs for dev/testing isolation. | Engineer |
| `merge-branch` | Promote changes across catalog environments. | Engineer |
| `create-tag` | Mark immutable snapshot versions. | Analyst |
| `generate-code` | Get Spark/PyIceberg connection config. | All Users |
| `get-token` | Manage personal API keys and sessions. | All Users |

---

## 📂 Architecture & Config
Both tools share the same configuration layer.
- **Interactive Mode**: Simply run `pangolin-admin` or `pangolin-user` to enter the shell.
- **Non-Interactive Mode**: Pass the command directly: `pangolin-user search --query "sales"`.

**Config Paths:**
- **Linux**: `~/.config/pangolin/config.json`
- **macOS**: `~/Library/Application Support/com.pangolin.cli/config.json`
- **Windows**: `C:\Users\<User>\AppData\Roaming\pangolin\cli\config.json`

### Pagination
Most list commands support standard pagination flags to handle large datasets effectively:
- `--limit <N>`: Restrict the number of results (default: 100).
- `--offset <N>`: Skip the first N results.

```bash
# Get the first 50 tables
pangolin-user search --query "sales" --limit 50

# Get the next 50
pangolin-user search --query "sales" --limit 50 --offset 50
```

---

## 🚀 Quick Tip
Use the `--profile` flag to manage multiple Pangolin environments (e.g., `dev` vs `prod`).
```bash
pangolin-user login --profile prod --username "me@company.com"
```
