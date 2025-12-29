
# Known Issues

This section documents verified issues, limitations, and architectural quirks present in the current release.

## v0.4.0

*   [SQL Backend Token Listing (SQLite/Postgres)](./token_listing_sqlite_join.md)
    *   **Description**: Active token lists may be empty for Root users or ephemeral accounts when using SQL backends due to a strict `JOIN` dependency.
    *   **Status**: Identified. Fix proposed for v0.5.x.
