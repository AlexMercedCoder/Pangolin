# Demo Evaluation & Recommendations

## 1. PyIceberg 404 "Catalog Not Found" Issue

### Investigation
We successfully reproduced the "Single Tenant / No Auth" demo environment and performed the following steps:
1.  Started the stack (`docker compose up -d`).
2.  Created `demo_warehouse` and `demo` catalog via both API (curl) and UI (simulated).
3.  Ran a Python script exactly matching the user's provided code.

**Result**: The code executed successfully against the local environment. `POST /v1/demo/v1/namespaces` returned 200 OK.

### Diagnosis
The error `RESTError 404: Catalog not found` indicates the API did not find a catalog named `demo`. Given that the stack uses `MemoryStore` (as per `README.md`), all metadata **is lost on container restart**.

**Likely Causes:**
1.  **Container Restart**: The `pangolin-api` container may have restarted between the time resources were created and the Python script was run.
2.  **Order of Operations**: The Python script was run before the catalog was successfully created.
3.  **Naming Mismatch**: A typo in the catalog name during creation (e.g., "demo " or "Demo").

### Recommendation
*   **Verify Persistence**: Run `docker ps` to ensure uptime of `pangolin-api`.
*   **Verify Existence**: Before running PyIceberg, verify the catalog exists:
    ```bash
    curl http://localhost:8080/api/v1/catalogs
    ```
*   **Retry Demo**: Restart the stack (`docker compose down && docker compose up -d`), wait for health check, create resources, and immediately run the script.

## 2. Catalog Name Collisions

### Question
"How is catalog name collisions handled what if two tenants have a catalog with the same name, and are you allowed to have two catalogs with the same name in the same tenant, does it give you a graceful error if not."

### Analysis
*   **Different Tenants**: Allowed. Catalogs are scoped by `tenant_id`.
*   **Same Tenant**:
    *   **MemoryStore**: **Silent Overwrite**. Creating a catalog with an existing name will update the existing entry without error.
    *   **PostgresStore**: **Database Error**. The unique constraint on `(tenant_id, name)` will trigger a violation. The current API handler catches this as a generic error and returns **500 Internal Server Error**.

### Recommendation
*   **Improve Error Handling**: Update `create_catalog` handler to catch exact collision errors (e.g., `sqlx::Error::Database` with constraint violation) or check existence first (in Postgres) and return **409 Conflict** with a graceful message ("Catalog with this name already exists").
    *   *Note*: `MemoryStore` needs to be updated to optionally return an error if key exists to match production behavior.

