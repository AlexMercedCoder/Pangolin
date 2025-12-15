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
Initialize a new tenant environment.

**Syntax**:
```bash
pangolin-admin create-tenant --name <name>
```

**Example**:
```bash
pangolin-admin create-tenant --name acme_corp
```

### Delete Tenant
Permanently remove a tenant and all associated data. **Irreversible**.

**Syntax**:
```bash
pangolin-admin delete-tenant <id>
```

**Example**:
```bash
pangolin-admin delete-tenant 123e4567-e89b-12d3
```
