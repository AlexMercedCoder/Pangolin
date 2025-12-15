# Admin Permissions Management

The permissions system in Pangolin allows for fine-grained access control over resources. Permissions are granted to **Roles**, and Roles are assigned to Users.

## Concepts

### Roles
Arbitrary strings representing a job function.
- `admin`
- `data_engineer`
- `analyst`
- `viewer`

### Actions
- `read`: View metadata and read data.
- `write`: Modify metadata and write data (insert/update/delete).
- `delete`: Delete resources.
- `create_branch`: Create new branches.
- `merge`: Merge branches.
- `manage_users`: Create/Delete users (System level).

### Resources
Resources are hierarchical strings.
1.  **System**: `system` (Global control)
2.  **Catalog**: `catalog:{catalog_name}`
3.  **Namespace**: `namespace:{catalog_name}:{namespace_name}`
4.  **Table**: `table:{catalog_name}:{namespace_name}:{table_name}`
5.  **Tag/Attribute**: `tag:{name}` or `attribute:{key}`

## Commands

### List Permissions
List all active permissions, optionally filtered.

**Syntax**:
```bash
pangolin-admin list-permissions [--role <role>] [--user <user>]
```

**Examples**:
```bash
# List all permissions in the system
pangolin-admin list-permissions

# See what the 'analyst' role can do
pangolin-admin list-permissions --role analyst
```

### Grant Permission
Grant a specific action on a resource to a **User**.

**Syntax**:
```bash
pangolin-admin grant-permission <username> <action> <resource>
```

**Examples**:
```bash
# Grant 'admin_user' read access to the 'sales' catalog
pangolin-admin grant-permission admin_user read catalog:sales

# Grant 'data_engineer' write access to a specific namespace
pangolin-admin grant-permission data_engineer write namespace:sales:region_us

# Allow 'audit_bot' to read tables tagged with 'compliance'
pangolin-admin grant-permission audit_bot read tag:compliance
```

### Revoke Permission
Remove a previously granted permission. The arguments must match exactly.

**Syntax**:
```bash
pangolin-admin revoke-permission <username> <action> <resource>
```

**Examples**:
```bash
# Remove write access for 'analyst_user' on the sales catalog
pangolin-admin revoke-permission analyst_user write catalog:sales
```
