# Admin Tenant Management

Tenants are the highest level of isolation in Pangolin. Data, users, and configuration do not leak between tenants.

## Commands

### List Tenants
View all existing tenants. (Root Admin only)

**Syntax**:
```bash
pangolin-admin list-tenants
```

### Create Tenant
Initialize a new tenant environment. (Root Admin only)

**Syntax (Interactive)**:
```bash
create-tenant --name <name> --admin-username <username> --admin-password <password>
```

**Syntax (Non-Interactive)**:
```bash
pangolin-admin create-tenant --name <name> --admin-username <username> --admin-password <password>
```

**Example**:
```bash
pangolin-admin create-tenant --name acme_corp --admin-username admin --admin-password "p@ssword123"
```

### Update Tenant
Modify properties of an existing tenant. (Root Admin only)

**Syntax**:
```bash
pangolin-admin update-tenant --id <uuid> [--name <new_name>]
```

**Example**:
```bash
pangolin-admin update-tenant --id 123e4567-e89b-12d3 --name acme_holdings
```

### Delete Tenant
Permanently remove a tenant and all associated data. **Irreversible**. (Root Admin only)

**Syntax**:
```bash
pangolin-admin delete-tenant <id>
```

**Example**:
```bash
pangolin-admin delete-tenant 123e4567-e89b-12d3
```
