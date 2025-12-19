# UI Last Stretch Plan

## Goal
Address final outstanding UI/UX requests and close security gaps in data visibility.

## 1. Dashboard Enhancement (PyIceberg Snippet)
**Current State**: Empty statistic cards.
**Goal**: Display a useful "Getting Started" code snippet for PyIceberg configuration.
**Implementation**:
- Create `src/lib/components/ui/CodeSnippet.svelte` (if not exists) or use existing.
- Update `src/routes/+page.svelte`:
    - Fetch current User/Tenant details.
    - Render a pre-filled Python script:
    ```python
    from pyiceberg.catalog import load_catalog

    catalog = load_catalog("pangolin", **{
        "uri": "http://<HOST>:8080/api/v1/catalogs/<CATALOG_NAME>",
        "s3.endpoint": "http://<S3_HOST>:9000",
        "py-iceberg.catalog-impl": "pyiceberg.catalog.rest.RestCatalog",
        "header.X-Pangolin-Tenant": "<TENANT_ID>",
        "token": "<ACCESS_TOKEN>" 
    })
    ```
    - Use placeholders for dynamic values (Host, Tenant ID). Display a "Copy" button.

## 2. Secure Catalog Listing (Explorer Fix)
**Status**: ✅ **Completed**
**Implementation**: `search_assets` now strictly enforces `READ` permissions or `discoverable` flag. FQN display and "Request Access" logic are verified. `list_catalogs` backend logic was updated to filter by permission.

## 3. Metadata Tagging in UI
**Status**: ✅ **Completed**
**Implementation**: Verified tag search support (`#tag`) and UI display. "Edit" form functionality verified.

## 4. Discovery Scope
**Status**: ✅ **Completed**
**Implementation**:
- `search_assets` logic finalized and verified.
- UI correctly handles:
    - **Has Access**: Shows "View Asset".
    - **No Access + Discoverable**: Shows "Request Access" + FQN.
    - **No Access + Not Discoverable**: Hidden.

## 5. Outstanding Work (The Last Stretch)
1.  **Dashboard Enhancement**:
    - [ ] Implement "Getting Started" PyIceberg snippet on the main dashboard (`src/routes/+page.svelte`).
    - [ ] Dynamic values for Token/TenantID.
2.  **Full Token Management (UI)**:
    - [ ] **List Tokens**: UI to show active tokens for the user (`/profile/tokens`?).
    - [ ] **Revoke Token**: Button to revoke specific tokens.
    - [ ] **Rotate Token**: Feature to cycle credentials.
3.  **Access Request Management (Admin UI)**:
    - [ ] Admin view to list pending requests (`/admin/requests`).
    - [ ] Approve/Reject actions in UI.

## Execution Order
1.  **Dashboard**: Implement PyIceberg snippet (Quick win).
2.  **Token Management**: Build UI for listing/revoking tokens.
3.  **Access Requests**: Build Admin UI for handling requests.
