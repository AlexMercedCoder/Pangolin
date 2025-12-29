
# Known Issue: SQL Backend Token Listing for Root/Ephemeral Users

**Affected Versions**: v0.4.0 and prior  
**Affected Backends**: SQLite, PostgreSQL  
**Unaffected Backends**: Memory, MongoDB  

## Summary
When listing active tokens via `/api/v1/users/me/tokens` or `/api/v1/users/{id}/tokens`, SQL-based backends (SQLite, PostgreSQL) fail to return results for users who do not exist in the `users` database table (e.g., Ephemeral Root Users) or if there is a data mismatch in the `JOIN` operation. This results in empty token lists even when valid tokens exist.

## Technical Detail
The `list_active_tokens` implementation in SQL backends relies on a `JOIN` between the `active_tokens` table and the `users` table to filter tokens by `tenant_id`:

```sql
SELECT t.token_id, t.user_id, t.token, t.expires_at 
FROM active_tokens t 
JOIN users u ON t.user_id = u.id 
WHERE u.tenant_id = ? ...
```

### The Problem
1.  **Ephemeral Root Users**: The default "System Root" (`admin`/`password` or env vars) is often ephemeral and does not have a persisted row in the `users` table.
    *   **Result**: The `JOIN` fails (Inner Join finds no matching `user_id`), so the query returns 0 rows.
2.  **Schema Sensitivity**: Any mismatch between the token's stored `user_id` and the `users` table record causes specific tokens to disappear from lists.

### Comparison with Other Backends
*   **Memory Store & MongoDB**: These backends store the `tenant_id` directly on the token record/document (`TokenInfo`). The list operation performs a direct filter on the tokens collection without needing to look up the user. This makes them robust for all user types.

## Workaround
*   **Use Memory Store** for robust local development involving Root user token management.
*   **Persist the User**: If using SQLite/Postgres, ensure the user (even Root) is explicitly created in the `users` table with the correct Tenant ID.
    *   *Note*: Root users are typically tenant-less, which makes the `WHERE u.tenant_id = ?` clause problematic even if the user exists, unless the query logic handles `NULL` tenants (which it currently doesn't for tenant-scoped inputs).

## Future Fix
The intended fix for v0.5.x is to denormalize the `tenant_id` column into the `active_tokens` table for SQL backends, mirroring the NoSQL/Memory architecture.

**Proposed Schema Change**:
```sql
ALTER TABLE active_tokens ADD COLUMN tenant_id TEXT;
CREATE INDEX idx_tokens_tenant ON active_tokens(tenant_id);
-- Migration logic to populate from users or defaults
```

**Proposed Query Change**:
```sql
SELECT * FROM active_tokens WHERE tenant_id = ? AND expires_at > ?
-- No JOIN required
```
