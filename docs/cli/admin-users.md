# Admin User Management

Manage users who can authenticate to the Pangolin system for the current tenant.

## Commands

### List Users
View all registered users and their primary roles.

**Syntax**:
```bash
pangolin-admin list-users
```

### Create User
Create a new user account.

> [!WARNING]
> The **Root User** can ONLY create a `tenant-admin` for a new tenant. They cannot create regular `tenant-user` accounts. Regular user management is the responsibility of the Tenant Admin.

**Syntax**:
```bash
pangolin-admin create-user <username> --email <email> --role <role> [--password <password>] [--tenant-id <uuid>]
```

**Options**:
- `--role`: Assign a role (`root`, `tenant-admin`, `tenant-user`).
- `--password`: Provide password directly (omitting will trigger a secure prompt).
- `--tenant-id`: Target a specific tenant (Root only).

**Example**:
```bash
pangolin-admin create-user alice --email alice@acme.com --role tenant-user
```

### Update User
Modify an existing user's profile or status.

**Syntax**:
```bash
pangolin-admin update-user --id <uuid> [--username <name>] [--email <email>] [--active <true|false>]
```

**Example**:
```bash
pangolin-admin update-user --id "user-uuid" --active false
```

### Delete User
Remove a user account.

**Syntax**:
```bash
pangolin-admin delete-user <username>
```

**Example**:
```bash
pangolin-admin delete-user bob
```
