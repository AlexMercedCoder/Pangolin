# Evaluating Pangolin

The easiest way to evaluate Pangolin and test its features (like PyIceberg integration, branching, and multi-tenancy) is to use the **`NO_AUTH`** mode locally.

This mode disables strict authentication requirements and enables an **Auto-Provisioning** system that sets up a ready-to-use environment for you.

## 1. Quick Start

Run the Pangolin server with the `PANGOLIN_NO_AUTH` environment variable set to `true`.

```bash
# In the pangolin/ directory
PANGOLIN_NO_AUTH=true cargo run --bin pangolin_api
```

### What Happens?

When the server starts in this mode, it will:
1.  **Create a Default Tenant**: A system tenant (ID: `00000000-0000-0000-0000-000000000000`) is created automatically.
2.  **Auto-Provision a Admin**: A Tenant Administrator user (`tenant_admin`) is created for you.
3.  **Generate Credentials**: A valid, long-lived JWT token is generated.
4.  **Print Configuration**: The server's startup logs will display a handy banner with everything you need.

## 2. Connecting with PyIceberg

Look at your terminal output for a block that looks like this:

```python
========================================================
 WARNING: NO_AUTH MODE ENABLED - FOR EVALUATION ONLY
========================================================
Initial Tenant Admin Auto-Provisioned:
 Username: tenant_admin
 Password: password123
 Tenant ID: 00000000-0000-0000-0000-000000000000
--------------------------------------------------------
PyIceberg Configuration Snippet:
--------------------------------------------------------
catalog = load_catalog(
    "local",
    **{
        "type": "rest",
        "uri": "http://127.0.0.1:8080/api/v1/catalogs/my_catalog/iceberg",
        "token": "eyJ0eXAiOiJKV1QiLCJ...",  <-- Your Auto-Generated Token
        "header.X-Pangolin-Tenant": "00000000-0000-0000-0000-000000000000"
    }
)
========================================================
```

Simply **copy and paste** that Python snippet into your script or notebook, and you are ready to interact with Pangolin as an Iceberg REST Catalog!

## 3. Using the CLI

You can also use the `pangolin-admin` CLI easily in this mode.

### Setup
Configure your CLI to point to your local instance. Since `NO_AUTH` mode auto-provisions a user, you can just login with the default credentials:

```bash
pangolin-admin login --username tenant_admin --password password123
```

### Try These Commands

Once logged in, verify your access and start exploring:

**1. Create a Warehouse**
Define where your data lives (e.g., an S3 bucket or local memory for testing).
```bash
pangolin-admin create-warehouse my_warehouse --type memory
```

**2. Create a Catalog**
Link a catalog name to your warehouse.
```bash
pangolin-admin create-catalog my_catalog --warehouse my_warehouse
```
*Note: This matches the URI in the Python snippet above (`.../catalogs/my_catalog/iceberg`).*

**3. Manage Users**
Create other users to test permissions.
```bash
pangolin-admin create-user data_scientist --role tenant-user --email ds@example.com
```

## Nuances of NO_AUTH Mode

While `NO_AUTH` mode makes evaluation simple, keep these important details in mind:

*   **It is NOT Secure**: Do not use this mode in production. Anyone with network access to the server can act as the root user.
*   **Root Access**: The server internally treats requests without specific auth headers as "Root" requests, but the auto-provisioned user is a **Tenant Admin**. This allows you to test the exact permissions strictness that a real user would face.
*   **Tenant Isolation**: Even in `NO_AUTH` mode, Pangolin enforces tenant isolation. You must provide the correct `X-Pangolin-Tenant` header (or have it in your valid JWT token) to access resources. The auto-provisioned token handles this for you.
