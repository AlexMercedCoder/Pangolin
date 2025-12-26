# CLI Admin Modularization Plan

## Goal
Refactor the monolithic `pangolin_cli_admin/src/handlers.rs` (~1,800 lines) into a modular structure. This file contains the logic for over 75 CLI commands, making it difficult to navigate and maintain.

## Proposed Structure
The logic will be moved to a new directory `pangolin_cli_admin/src/handlers/`:

```
handlers/
├── mod.rs                   # Re-exports and dispatcher registration
├── tenants.rs               # Tenant CRUD commands
├── warehouses.rs            # Warehouse management commands
├── catalogs.rs              # Catalog management commands
├── users.rs                 # User and Service User commands
├── permissions.rs           # Role, Assignment, and Permission commands
├── security.rs              # Token and OAuth configuration commands
├── metadata.rs              # Business Metadata and Tag commands
├── collaboration.rs         # Branching and Merge operation commands
├── audit.rs                 # Audit log retrieval and search commands
├── federated.rs             # Federated catalog management commands
└── utils.rs                 # Shared CLI formatting and helper functions
```

## Implementation Steps
1. **Create Directory Structure**: Create `src/handlers/` and empty module files.
2. **Move Shared Helpers**: Extract utility functions (e.g., ID resolution, table formatting) to `utils.rs`.
3. **Categorical Extraction**:
    - Extract Tenant/Warehouse/Catalog logic.
    - Extract User/Role/Permission logic.
    - Extract Metadata/Tag logic.
    - Extract Branch/Merge/Audit logic.
4. **Update Dispatcher**: Modify `mod.rs` to re-export all handlers so they can be easily linked to the CLI command parser in `main.rs`.
5. **Update main.rs**: Update the command handling loop to call the modularized handlers.

## Verification Plan
- **Compilation**: Ensure `pangolin_cli_admin` compiles successfully.
- **Interactive Smoke Test**: Test 2-3 key commands from each category (e.g., `tenant list`, `user create`, `audit list`) to verify routing.
- **Scripted Verification**: Run `test_cli_live.sh` to ensure all high-level flows still function.
