# Outstanding CLI Work - Post Backend Enhancements

## Overview
After implementing backend enhancements (#13, #14, #15, #6, #5), the CLI tools need updates to support new features.

## Priority: HIGH - Tenant-Scoped Login

### Issue
CLI login commands don't support the new `tenant-id` parameter for tenant-scoped authentication.

### Required Changes

#### `pangolin-admin` Login
**File**: `pangolin_cli/src/admin/auth.rs` (or similar)

**Add `--tenant-id` flag**:
```rust
#[derive(Parser)]
pub struct LoginArgs {
    #[arg(short, long)]
    username: String,
    
    #[arg(short, long)]
    password: String,
    
    #[arg(short, long)]
    tenant_id: Option<String>,  // New field
}
```

**Update login request**:
```rust
let login_request = json!({
    "username": args.username,
    "password": args.password,
    "tenant-id": args.tenant_id  // Note: kebab-case!
});
```

**Examples**:
```bash
# Root login
pangolin-admin login --username admin --password password

# Tenant-scoped login
pangolin-admin login --username user --password pass123 --tenant-id <uuid>
```

---

## Priority: MEDIUM - Dashboard Stats

### Issue
Dashboard command may not display `tenants_count` field.

### Required Changes

**File**: `pangolin_cli/src/admin/dashboard.rs` (or similar)

**Update stats display**:
```rust
println!("Dashboard Statistics (Scope: {})", stats.scope);
println!("=====================================");
if let Some(count) = stats.tenants_count {
    println!("Tenants:     {}", count);
}
println!("Catalogs:    {}", stats.catalogs_count);
println!("Warehouses:  {}", stats.warehouses_count);
// ... other fields
```

---

## Priority: MEDIUM - Audit Log Cross-Tenant Queries

### Issue
Audit log commands may not properly handle cross-tenant visibility for Root users.

### Required Changes

**File**: `pangolin_cli/src/admin/audit.rs` (or similar)

**Add tenant filter for Root users**:
```rust
#[derive(Parser)]
pub struct ListAuditArgs {
    // ... existing fields
    
    #[arg(long)]
    tenant_id: Option<String>,  // Filter by tenant (Root only)
}
```

**Update output to show tenant context**:
```rust
for event in events {
    println!("Tenant: {}", event.tenant_id);
    println!("Action: {}", event.action);
    // ... other fields
}
```

---

## Testing Requirements

### Manual Testing
1. Test Root login without `--tenant-id`
2. Test tenant-scoped login with `--tenant-id`
3. Verify dashboard shows `tenants_count` for Root users
4. Test audit log cross-tenant queries

### Regression Tests
- Add unit tests for login with/without tenant-id
- Test dashboard stats parsing
- Test audit log tenant filtering

---

## Documentation Updates

### CLI Documentation
**File**: `docs/cli/admin-authentication.md` (or create if missing)

Add examples:
```markdown
## Tenant-Scoped Login

### Root Login
```bash
pangolin-admin login --username admin --password password
```

### Tenant-Scoped Login
```bash
# Get tenant ID first
TENANT_ID=$(pangolin-admin tenants list --format json | jq -r '.[0].id')

# Login with tenant context
pangolin-admin login --username user --password pass123 --tenant-id $TENANT_ID
```
```

---

## Implementation Checklist

- [ ] Add `--tenant-id` flag to `pangolin-admin login`
- [ ] Update login request to use `tenant-id` (kebab-case)
- [ ] Add `--tenant-id` flag to `pangolin-user login` (if exists)
- [ ] Update dashboard stats display to show `tenants_count`
- [ ] Add `--tenant-id` filter to audit log commands
- [ ] Update audit log output to show tenant context
- [ ] Write unit tests for new flags
- [ ] Update CLI documentation
- [ ] Test with live API server

---

## Notes

### JSON Field Naming
**CRITICAL**: Use `tenant-id` (kebab-case) in JSON requests, NOT `tenant_id`:
```json
{
  "username": "user",
  "password": "pass",
  "tenant-id": "uuid-here"
}
```

### Backward Compatibility
Ensure existing scripts work - `--tenant-id` should be optional, defaulting to Root login when omitted.
