
# Known Issues

This section documents verified issues, limitations, and architectural quirks present in the current release.

## Resolved

*   [SQL Backend Token Listing (SQLite/Postgres)](./token_listing_sqlite_join.md)
    *   **Description**: Active token lists were empty for Root or ephemeral accounts on SQL backends, because the query inner-joined `active_tokens` to `users` and an ephemeral root has no row there.
    *   **Status**: **Fixed.** Both backends now filter `tenant_id` directly off `active_tokens`, matching what the memory and MongoDB backends always did. Verified by the cross-backend parity suite, which asserts identical behaviour on all four.

## Current

No verified issues are open against 0.8.0 beyond the limitations recorded in
[STATUS.md](../../STATUS.md), which is the authoritative list.
