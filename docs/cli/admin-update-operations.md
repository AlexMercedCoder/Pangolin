# Admin CLI - Update Operations

This guide covers the update commands for modifying existing resources in Pangolin.

---

## Update Tenant

Update properties of an existing tenant.

### Syntax
```bash
pangolin-admin update-tenant --id <TENANT_ID> --name <NEW_NAME>
```

### Parameters
- `--id` - UUID of the tenant to update (required)
- `--name` - New name for the tenant (optional)

### Example
```bash
# Update tenant name
pangolin-admin update-tenant \
  --id "8582f8f2-63af-4ecf-9aaf-82133bc42a16" \
  --name "production-tenant"
```

### Output
```
✅ Tenant updated successfully!
```

### Notes
- Requires root privileges
- Tenant ID must be a valid UUID
- Only specified fields will be updated

---

## Update User

Update properties of an existing user.

### Syntax
```bash
pangolin-admin update-user --id <USER_ID> [OPTIONS]
```

### Parameters
- `--id` - UUID of the user to update (required)
- `--username` - New username (optional)
- `--email` - New email address (optional)
- `--active` - Set active status (true/false) (optional)

### Examples
```bash
# Update username
pangolin-admin update-user \
  --id "user-uuid" \
  --username "new-username"

# Update email
pangolin-admin update-user \
  --id "user-uuid" \
  --email "newemail@example.com"

# Deactivate user
pangolin-admin update-user \
  --id "user-uuid" \
  --active false

# Update multiple fields
pangolin-admin update-user \
  --id "user-uuid" \
  --username "updated-user" \
  --email "updated@example.com" \
  --active true
```

### Output
```
✅ User updated successfully!
```

### Notes
- Requires tenant admin or root privileges
- User ID must be a valid UUID
- At least one optional parameter must be provided

---

## Update Warehouse

Update properties of an existing warehouse.

### Syntax
```bash
pangolin-admin update-warehouse --id <WAREHOUSE_ID> --name <NEW_NAME>
```

### Parameters
- `--id` - UUID of the warehouse to update (required)
- `--name` - New name for the warehouse (optional)

### Example
```bash
# Update warehouse name
pangolin-admin update-warehouse \
  --id "warehouse-uuid" \
  --name "production-warehouse"
```

### Output
```
✅ Warehouse updated successfully!
```

### Notes
- Requires tenant admin privileges
- Warehouse ID must be a valid UUID
- Name must be unique within the tenant

---

## Update Catalog

Update properties of an existing catalog.

### Syntax
```bash
pangolin-admin update-catalog --id <CATALOG_ID> --name <NEW_NAME>
```

### Parameters
- `--id` - UUID of the catalog to update (required)
- `--name` - New name for the catalog (optional)

### Example
```bash
# Update catalog name
pangolin-admin update-catalog \
  --id "catalog-uuid" \
  --name "production-catalog"
```

### Output
```
✅ Catalog updated successfully!
```

### Notes
- Requires tenant admin privileges
- Catalog ID must be a valid UUID
- Name must be unique within the tenant

---

## Common Patterns

### Getting Resource IDs

To update a resource, you first need its ID. Use list commands:

```bash
# Get tenant ID
pangolin-admin list-tenants

# Get user ID
pangolin-admin list-users

# Get warehouse ID
pangolin-admin list-warehouses

# Get catalog ID
pangolin-admin list-catalogs
```

### Batch Updates

For multiple updates, use a script:

```bash
#!/bin/bash
# Update multiple users
for user_id in "${USER_IDS[@]}"; do
  pangolin-admin update-user --id "$user_id" --active true
done
```

---

## Error Handling

### Common Errors

**404 Not Found**:
```
Error: API Request Failed: Failed to update tenant (404 Not Found): Tenant not found
```
- Solution: Verify the ID is correct

**403 Forbidden**:
```
Error: API Request Failed: Failed to update user (403 Forbidden): Insufficient permissions
```
- Solution: Ensure you have appropriate privileges

**400 Bad Request**:
```
Error: API Request Failed: Failed to update catalog (400 Bad Request): Name already exists
```
- Solution: Choose a unique name

---

## Best Practices

1. **Verify Before Update**: List resources first to confirm IDs
2. **Use Descriptive Names**: Choose meaningful names for resources
3. **Test in Development**: Test updates in a dev environment first
4. **Document Changes**: Keep a log of resource updates
5. **Check Permissions**: Ensure you have the required privileges

---

## Related Commands

- `list-tenants` - List all tenants
- `list-users` - List all users
- `list-warehouses` - List all warehouses
- `list-catalogs` - List all catalogs
- `delete-tenant` - Delete a tenant
- `delete-user` - Delete a user
- `delete-warehouse` - Delete a warehouse
- `delete-catalog` - Delete a catalog
