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
Create a new user account. You will be prompted for a password securely.

**Syntax**:
```bash
pangolin-admin create-user <username> [--role <role>]
```

**Options**:
- `--role`: Assign an initial role (default: `tenant-user`).

**Example**:
```bash
pangolin-admin create-user alice --role tenant-user
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
