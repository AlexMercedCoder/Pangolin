# UI Security Audit Report: Tenant Isolation & RBAC

This report summarizes findings from a comprehensive audit of the Pangolin UI logic, focusing on multi-tenancy isolation and security risks.

## 🚨 Critical Security Risks

### 1. Dual Store Inconsistency (Conflicting Auth Systems)
The codebase contains two separate and conflicting authentication/store implementations:
- **Legacy System**: `src/lib/auth.ts` (Uses `pangolin_user`, `pangolin_token`, `pangolin_selected_tenant`)
- **Modern System**: `src/lib/stores/auth.ts` (Uses `auth_user`, `auth_token`, `pangolin_selected_tenant`)

**Risk**: The `TenantSelector` component relies on the Legacy System, while most management pages rely on the Modern System. If a user has stale data in `localStorage` from a previous session, the UI might show the Tenant Selector to a non-Root user, or vice versa. This can lead to unauthorized context switching if the backend doesn't strictly validate the JWT against the `X-Pangolin-Tenant` header.

### 2. Lack of Proactive Route Protection
While the sidebar correctly filters navigation links based on roles (`$isRoot`), the `+layout.svelte` and individual routes do not proactively block manual navigation to restricted paths (e.g., `/tenants`, `/warehouses` for non-admins).
- **Current Behavior**: The page loads, the API call fails (backend security), and the user sees an empty state or error toast.
- **Risk**: Information leakage of route existence and potential UI exposure before API failure.

---

## 🔍 Specific Audit Findings

### Tenant Admin Isolation
- **User Creation**: `CreateUserForm.svelte` correctly hides the Tenant Selection for non-Root users. No vulnerability found here.
- **User Editing**: The user list correctly prevents Tenant Admins from editing Root users.
- **Tenant Hub**: Tenant Admins can manually navigate to `/tenants`, which should be restricted in the UI to prevent confusion.

### Root Context Switching
- The `TenantSelector` is visible only to users identified as `Root`.
- **Finding**: In `src/lib/components/TenantSelector.svelte`, the `fetchTenants` call (Line 17) hardcodes a fetch to `/api/v1/tenants`. If a Tenant Admin managed to trigger this, they could potentially list all tenants if the backend isn't properly scoped.

---

## 📈 Suggestions for UI Documentation & Help

To improve the user experience and reduce configuration errors, I suggest the following UI-based documentation features:

### 1. Inline "Info" Tooltips
Add subtle info icons next to complex fields like "Credential Vending Strategy", "Storage Prefix", and "Federated Auth Type".
- **Benefit**: Immediate context without leaving the form.

### 2. Role-Based Dashboard Hints
When a Tenant Admin first logs in, show a "Getting Started" widget with links to:
- "How to create your first Warehouse"
- "Adding your team (User Management)"
- "Setting up RBAC"

### 3. Contextual Documentation Panel
Implement a slide-over "Help" panel (triggered by a floating action button or sidebar link) that fetches Markdown content from the `docs/` folder based on the current route.
- **Example**: If the user is on `/warehouses`, the panel shows content from `docs/features/warehouse_management.md`.

### 4. Admin Guided Tour
Use a library like `driver.js` to provide a guided walkthrough for the "Root" and "Tenant Admin" dashboards.

---

## ✅ Recommended Actions (Non-Disruptive)

1. **Consolidate Stores**: Move all logic to `src/lib/stores/auth.ts` and remove/deprecate `src/lib/auth.ts`. Ensure `TenantSelector` uses the same store.
2. **Implement Guard Logic**: Add a check in `+layout.svelte` to redirect users if they lack the required role for a specific route.
3. **Audit Backend Parity**: Ensure every UI route that sends `X-Pangolin-Tenant` is accompanied by a backend check that verifies the user has access to that specific tenant.
