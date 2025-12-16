# Comprehensive Test Plan

## 1. Backend Testing (`pangolin_api`, `pangolin_core`, `pangolin_store`)

### Unit Tests
- **Goal**: Verify logic of individual components.
- **Tools**: `cargo test`
- **Scope**:
    - `pangolin_core`: Serialization/Deserialization of models.
    - `pangolin_store`:
        - `create/read/update/delete` for all entities (Tenant, User, Asset, AccessRequest).
        - Use `MemoryStore` for fast logic verification.
        - Use `SqliteStore`, `PostgresStore`, `MongoStore` for backend-specific implementation verification.

### Integration Tests
- **Goal**: Verify API endpoints and handler logic.
- **Tools**: `cargo test -p pangolin_api`
- **Scope**:
    - `business_metadata_flow`: Verifies metadata addition, search, and access request workflow.
    - `auth_middleware`: Verifies JWT parsing and RBAC enforcement.
    - `iceberg_endpoints_tests`: Verifies compliance with Iceberg REST spec.

### Manual Verification
- **Goal**: Verify interactions via HTTP client.
- **Tools**: `curl`, Postman, or `scripts/integration_test.py`.
- **Scenarios**:
    - Create Tenant -> Create User -> Create Catalog -> Branching -> Merging.

## 2. CLI Testing (`pangolin_cli`)

### Automated Verification (Scripted)
- **Goal**: Ensure CLI commands return expected exit codes and output.
- **Plan**: Create `tests/cli_e2e.sh` script.
- **Scenarios**:
    - `pangolin-admin tenants list`
    - `pangolin-admin users create`
    - `pangolin-user access-requests list`

### Manual Verification
- **Goal**: Verify UX and REPL behavior.
- **Plan**: Walkthrough of REPL mode usage.

## 3. UI Testing (`pangolin_ui`)

### Unit/Component Tests
- **Goal**: Verify individual Svelte components.
- **Tools**: `vitest` (currently using `playwright` for everything, but `vitest` recommended for logic).

### End-to-End (E2E) Tests
- **Goal**: Verify full user journeys in the browser.
- **Tools**: Playwright.
- **Scenarios**:
    - **Auth Flow**: Login/Logout.
    - **Admin Flow**: Create Tenant, Create Warehouse, Create Catalog.
    - **Discovery Flow**:
        1. Login as User.
        2. Go to `/discovery`.
        3. Search for "sales_data".
        4. Click "Request Access".
        5. Submit request.
        6. Logout.
        7. Login as Admin.
        8. Go to `/admin/requests`.
        9. Approve request.
        10. Logout.
        11. Login as User.
        12. Verify access (if applicable).

## 4. Execution Strategy

1. **Continuous Integration**: Run Backend Unit/Integration tests on every commit.
2. **Nightly**: Run Full E2E (UI + CLI) against a fresh environment.
3. **Pre-Release**: Manual Smoke Test using the `walkthrough.md` checklist.
