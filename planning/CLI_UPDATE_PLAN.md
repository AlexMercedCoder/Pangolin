# CLI Update Implementation Plan

**Created**: December 19, 2025  
**Status**: 🔄 **READY FOR IMPLEMENTATION**

## Overview

All new backend endpoints have been implemented and verified across all three storage backends (Sqlite, Postgres, Mongo). This plan outlines the CLI updates needed to expose this functionality to users.

## Scope

Update both `pangolin-admin` and `pangolin-user` CLIs to support:
1. Token Management (list, rotate, delete)
2. System Configuration (get, update)
3. Federated Catalog Operations (sync, stats)
4. Enhanced Data Explorer (namespace tree)
5. Branching Operations (rebase)

---

## 1. Token Management Commands

### Admin CLI (`pangolin-admin`)

#### New Commands

**List User Tokens**
```bash
pangolin-admin list-user-tokens <user_id>
```
- **Endpoint**: `GET /api/v1/users/{user_id}/tokens`
- **Output**: Table with token ID, created_at, expires_at, is_valid
- **Permissions**: Root or TenantAdmin

**Delete Token**
```bash
pangolin-admin delete-token <token_id>
```
- **Endpoint**: `DELETE /api/v1/tokens/{token_id}`
- **Permissions**: Root or TenantAdmin
- **Confirmation**: Prompt before deletion

### User CLI (`pangolin-user`)

#### New Commands

**List My Tokens**
```bash
pangolin-user list-my-tokens
```
- **Endpoint**: `GET /api/v1/users/me/tokens`
- **Output**: Table with token ID, created_at, expires_at

**Rotate Token**
```bash
pangolin-user rotate-token
```
- **Endpoint**: `POST /api/v1/tokens/rotate`
- **Output**: New token (print to stdout for scripting)
- **Note**: Automatically updates stored token in config

---

## 2. System Configuration Commands

### Admin CLI Only

**Get System Settings**
```bash
pangolin-admin get-system-settings
```
- **Endpoint**: `GET /api/v1/config/settings`
- **Output**: JSON or formatted table
- **Permissions**: Root or TenantAdmin

**Update System Settings**
```bash
pangolin-admin update-system-settings \
  [--allow-public-signup <true|false>] \
  [--default-warehouse-bucket <bucket>] \
  [--default-retention-days <days>] \
  [--smtp-host <host>] \
  [--smtp-port <port>] \
  [--smtp-user <user>] \
  [--smtp-password <password>]
```
- **Endpoint**: `PUT /api/v1/config/settings`
- **Permissions**: Root or TenantAdmin
- **Note**: Only specified fields are updated (partial update)

---

## 3. Federated Catalog Operations

### Admin CLI

**Sync Federated Catalog**
```bash
pangolin-admin sync-federated-catalog <catalog_name>
```
- **Endpoint**: `POST /api/v1/federated-catalogs/{name}/sync`
- **Output**: Confirmation message
- **Permissions**: Root or TenantAdmin

**Get Sync Stats**
```bash
pangolin-admin get-federated-stats <catalog_name>
```
- **Endpoint**: `GET /api/v1/federated-catalogs/{name}/stats`
- **Output**: Formatted stats (last_synced_at, status, tables_synced, etc.)
- **Permissions**: Root or TenantAdmin

### User CLI

**Get Sync Stats** (Read-only)
```bash
pangolin-user get-federated-stats <catalog_name>
```
- **Endpoint**: `GET /api/v1/federated-catalogs/{name}/stats`
- **Output**: Same as admin, but read-only access

---

## 4. Enhanced Data Explorer

### Both CLIs

**List Namespace Tree**
```bash
pangolin-admin list-namespace-tree <catalog_name>
pangolin-user list-namespace-tree <catalog_name>
```
- **Endpoint**: `GET /api/v1/catalogs/{catalog}/namespaces/tree`
- **Output**: Tree structure (ASCII art or JSON with `--json` flag)
- **Example Output**:
  ```
  catalog_name
  ├── sales
  │   ├── north_america
  │   └── europe
  └── marketing
      └── campaigns
  ```

---

## 5. Branching Operations

### User CLI

**Rebase Branch**
```bash
pangolin-user rebase-branch <catalog_name> <target_branch> <source_branch>
```
- **Endpoint**: `POST /api/v1/branches/{name}/rebase`
- **Output**: Confirmation or conflict details
- **Note**: Similar to merge but uses rebase strategy

---

## Implementation Details

### File Structure

```
pangolin_cli_admin/src/
├── commands/
│   ├── tokens.rs          # NEW: Token management commands
│   ├── system_config.rs   # NEW: System configuration commands
│   ├── federated.rs       # ENHANCE: Add sync/stats commands
│   └── explorer.rs        # NEW: Namespace tree command

pangolin_cli_user/src/
├── commands/
│   ├── tokens.rs          # NEW: User token commands
│   ├── federated.rs       # NEW: Read-only federated stats
│   ├── explorer.rs        # NEW: Namespace tree command
│   └── branches.rs        # ENHANCE: Add rebase command
```

### Common Patterns

**Request/Response Structs**
- Reuse existing structs from `pangolin_core` where possible
- Add new structs in `pangolin_cli_common` for CLI-specific formatting

**Error Handling**
- Use consistent error messages
- Provide helpful hints for common failures (e.g., "Token not found. Use `list-my-tokens` to see available tokens.")

**Output Formatting**
- Default: Human-readable tables
- `--json` flag: JSON output for scripting
- `--quiet` flag: Minimal output (success/failure only)

**Authentication**
- All commands require valid token
- Token rotation automatically updates stored config
- Clear error messages for expired/invalid tokens

---

## Testing Plan

### Unit Tests
- Test each command's argument parsing
- Test request struct serialization
- Test response struct deserialization

### Integration Tests
- Test against live API (using test tenant)
- Verify token rotation updates config
- Verify system settings update correctly
- Verify federated sync triggers correctly

### E2E Test Script
Create `test_cli_new_features.sh`:
```bash
#!/bin/bash
# Test token management
pangolin-user list-my-tokens
pangolin-user rotate-token

# Test system config (admin)
pangolin-admin get-system-settings
pangolin-admin update-system-settings --allow-public-signup true

# Test federated operations
pangolin-admin sync-federated-catalog fed_cat
pangolin-admin get-federated-stats fed_cat

# Test namespace tree
pangolin-user list-namespace-tree my_catalog

# Test rebase
pangolin-user rebase-branch my_catalog main feature_branch
```

---

## Documentation Updates

### Files to Update
1. `docs/cli/admin-tokens.md` - NEW
2. `docs/cli/admin-system-config.md` - NEW
3. `docs/cli/admin-federated.md` - ENHANCE
4. `docs/cli/user-tokens.md` - NEW
5. `docs/cli/user-explorer.md` - NEW
6. `docs/cli/user-branches.md` - ENHANCE
7. `README.md` - Update CLI command list

### Help Text
- Each command needs comprehensive `--help` text
- Include examples in help text
- Document all flags and options

---

## Estimated Effort

| Task | Estimated Time | Priority |
|------|---------------|----------|
| Token Management (Admin) | 2 hours | High |
| Token Management (User) | 1 hour | High |
| System Config | 2 hours | Medium |
| Federated Operations | 2 hours | Medium |
| Namespace Tree | 1 hour | Low |
| Rebase Command | 1 hour | Low |
| Testing | 3 hours | High |
| Documentation | 2 hours | Medium |
| **Total** | **14 hours** | |

---

## Success Criteria

- [ ] All 11 new commands implemented
- [ ] All commands have `--help` text
- [ ] All commands support `--json` output
- [ ] Token rotation updates stored config
- [ ] E2E test script passes
- [ ] Documentation complete
- [ ] No regressions in existing commands

---

## Dependencies

- ✅ All API endpoints implemented
- ✅ All backends support new functionality
- ✅ UI role format fixed (kebab-case)
- ⚠️ CLI needs to use kebab-case for roles (same fix as UI)

---

## Notes

- **Role Format**: CLI must use kebab-case (`tenant-admin`, `root`, `tenant-user`) to match API
- **Backward Compatibility**: Existing commands should not be affected
- **Config Updates**: Token rotation should seamlessly update the stored token
- **Error Messages**: Should guide users to correct usage

---

**Next Steps**: Begin implementation with token management commands (highest priority)
