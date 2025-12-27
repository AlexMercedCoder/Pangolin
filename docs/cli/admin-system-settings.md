# System Settings Management

The `pangolin-admin` tool provides commands to view and update global system configuration. These settings are stored in the backend database and override default environment variables where applicable.

## Commands

### `get-system-settings`

Retrieve the current global system configuration.

**Usage:**
```bash
pangolin-admin get-system-settings
```

**Output:**
```json
{
  "allow_public_signup": false,
  "default_warehouse_bucket": "warehouse",
  "default_retention_days": 30,
  "smtp_host": "smtp.example.com",
  "smtp_user": "admin"
}
```

### `update-system-settings`

Update one or more global configuration parameters.

**Usage:**
```bash
pangolin-admin update-system-settings \
  --allow-public-signup true \
  --default-retention-days 90
```

**Options:**
- `--allow-public-signup <bool>`: Enable or disable public user registration.
- `--default-warehouse-bucket <string>`: Default bucket name for new warehouses.
- `--default-retention-days <int>`: Default data retention period.
- `--smtp-host <string>`: SMTP server hostname.
- `--smtp-port <int>`: SMTP server port.
- `--smtp-user <string>`: SMTP username.
- `--smtp-password <string>`: SMTP password (write-only).

**Example:**
```bash
# Enable public signup and set default retention to 60 days
pangolin-admin update-system-settings --allow-public-signup true --default-retention-days 60
```
