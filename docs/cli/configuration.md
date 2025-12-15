# CLI Configuration Guide

The Pangolin CLI tools (`pangolin-admin` and `pangolin-user`) share a common configuration file to persist session state and preferences.

## File Location

The configuration file is stored in `config.json` within your system's standard configuration directory:

- **Linux**: `~/.config/pangolin/cli/config.json`
- **macOS**: `~/Library/Application Support/com.pangolin.cli/config.json`
- **Windows**: `C:\Users\<User>\AppData\Roaming\pangolin\cli\config.json`

## Configuration Properties

The file is a standard JSON object with the following properties:

| Property | Type | Description | Default |
|----------|------|-------------|---------|
| `base_url` | String | The URL of the Pangolin API server. | `http://localhost:8080` |
| `auth_token` | String | The JWT token for the currently logged-in session. | `null` |
| `username` | String | The username of the logged-in user. | `null` |
| `tenant_id` | String | The ID of the currently active tenant (for tenant admins). | `null` |

## Example `config.json`

```json
{
  "base_url": "http://localhost:8080",
  "auth_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "username": "admin",
  "tenant_id": "123e4567-e89b-12d3-a456-426614174000"
}
```

## Manual Editing
## Multiple Profiles
You can manage multiple sessions (e.g., `dev`, `prod`, `dummy_user`) using the `--profile` flag.

- **Default**: `~/.config/pangolin/cli/config.json`
- **Profile 'prod'**: `~/.config/pangolin/cli/config-prod.json`

To use a profile:
```bash
pangolin-admin --profile prod login ...
pangolin-user --profile dev list-catalogs
```
Each profile maintains its own `base_url`, `auth_token`, and `username`.
You can manually edit this file to change the target server or clear your session (by removing `auth_token` and `username`). However, it is recommended to use the CLI commands (`login`, `--url`) to manage these settings.
