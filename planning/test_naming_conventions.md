# Test File Naming Conventions

To ensure that temporary test files and databases are not accidentally committed to the repository, follow these naming conventions.

## Databases
All SQLite and other local database files should be automatically ignored by Git. However, to be explicit:
- Use the suffix `.db`, `.sqlite`, or `.sqlite3`.
- Preferred prefix: `temp_` or `test_` (e.g., `temp_user_data.db`).

## Scripts
While many test scripts are valuable for regression testing, ephemeral or feature-specific debug scripts should follow these conventions:
- Use the prefix `tmp_` for scripts that are not intended to be part of the long-term test suite.
- Example: `tmp_debug_auth.py`.

## Temporary Output
Any files containing diagnostic output should be prefixed with `tmp_` or `debug_`.
- Example: `debug_api_response.json`.

## Gitignored Patterns
The following patterns are currently ignored in the root `.gitignore`:
- `*.db`
- `*.sqlite`
- `*.sqlite3`
- `temp_*`
- `tmp_*`
- `debug_*`
- `test_output*.txt`
- `*_output*.txt`
