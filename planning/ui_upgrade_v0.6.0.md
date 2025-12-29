# UI Upgrade Plan (v0.6.0)

**Constraint**: The Backend API is **LOCKED** (v0.5.0). All UI features must work with existing endpoints.

## 1. Service User Management (New Feature)

**Goal**: Expose the hidden v0.5.0 Service User API features in the UI.

### A. Settings -> Service Users Page
*(New Route: `/settings/service-users`)*

*   **Access Control**: Only visible/accessible to `TenantAdmin`.
*   **Data Table**:
    *   Columns: Name, Role (Badge), Active? (Boolean Icon), Created At, Actions.
    *   Pagination: Use standard `limit`/`offset` query params.
    *   **Action Menu**:
        *   Edit (Update Description/Active Status).
        *   Rotate Key.
        *   Delete.

### B. Create Service User Modal
`POST /api/v1/service-users`

*   **Fields**:
    *   Name (Required)
    *   Description (Optional)
    *   Role (Dropdown: `tenant-user` [Default], `tenant-admin`) — *Note: Backend supports custom roles here? Or just rigid string? Checked: rigid string in DB, but logic might allow custom Role ID if we relax it. For now, stick to `tenant-user`.*
*   **Success State**:
    *   **CRITICAL**: The API returns the API Key **ONLY** in the response body of the Creation request.
    *   **UI Behavior**: Must show a "Copy this Key" dialog immediately. The key is never retrievable again.

### C. Rotate Key Modal
`POST /api/v1/service-users/{id}/rotate`

*   **Warning**: "This will invalidate the previous key immediately."
*   **Success State**: Same as Create—display new Key for copying.

## 2. RBAC Visibility (Polish)

*   **Role Badges**: Ensure user lists display generic roles properly.
*   **Permission Error Handling**:
    *   If a standard user tries to access `/settings/warehouses`, catch the `403` and show a friendly checking "Access Denied" or hide the link entirely based on `session.role` state.

## 3. Implementation Steps

1.  **Add `ServiceUser` Store**: Svelte store to manage list/state.
2.  **Add `ServiceUserClient`**: TypeScript client wrapper for API endpoints.
3.  **Implement List Page**: `/settings/service-users/+page.svelte`.
4.  **Implement Create Modal**: Form & Copy-Key logic.
5.  **Implement Rotate Modal**.
6.  **Verify**: Log in as Tenant Admin, create bot, verify key works via `curl` or PyPangolin.
