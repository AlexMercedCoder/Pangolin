# Iceberg Handlers Modularization Plan

## Goal
Refactor the monolithic `pangolin_api/src/iceberg_handlers.rs` (~1,850 lines) into a modular directory structure to improve maintainability, reduce compilation times, and align with the Iceberg REST API specification.

## Proposed Structure
The logic will be moved to a new directory `pangolin_api/src/iceberg/`:

```
iceberg/
├── mod.rs                   # Route definitions + re-exports
├── namespaces.rs            # Namespace CRUD (List, Create, Load, Delete, Update)
├── tables.rs                # Table CRUD (List, Create, Load, Drop, Rename)
├── table_metadata.rs        # Metadata operations (Commit/Update, Register)
├── snapshots.rs             # Snapshot management and cleanup
├── config.rs                # Catalog configuration endpoint
├── views.rs                 # View operations (if implemented)
└── types.rs                 # Iceberg-specific request/response types
```

## Implementation Steps
1. **Create Directory Structure**: Create `src/iceberg/` and empty module files.
2. **Extract Common Types**: Move all Iceberg-specific structs (e.g., `IcebergNamespace`, `IcebergTable`) to `types.rs`.
3. **Namespace Operations**: Extract all handlers related to `/v1/{prefix}/namespaces`.
4. **Table Operations**: Extract all handlers related to `/v1/{prefix}/namespaces/{ns}/tables`.
5. **Metadata & Commits**: Extract the complex `update_table` (commit) and `register_table` logic to `table_metadata.rs`.
6. **Configuration**: Move the `get_config` handler to `config.rs`.
7. **Update Routes**: Modify `src/iceberg/mod.rs` to wire up the new handler modules and export a consolidated `register_routes` function.
8. **Update lib.rs**: Replace the `iceberg_handlers` module with the new `iceberg` module.

## Verification Plan
- **Compilation**: Ensure `pangolin_api` compiles without errors.
- **Integration Tests**: Run the full suite of PyIceberg integration tests to ensure no regressions in REST API compliance.
- **REST Compliance**: Verify against the Iceberg OpenAPI specification.
