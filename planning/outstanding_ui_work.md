# Outstanding UI Work - Post Backend Enhancements

## Overview
After implementing backend enhancements (#13, #14, #15, #6, #5), the UI needs updates to support new features.

---

## 🔴 Priority: HIGH - Tenant-Scoped Login

### Issue
UI login doesn't support tenant-scoped authentication, preventing users with duplicate usernames across tenants from logging in.

### Required Changes

#### 1. Update Login API Interface
**File**: `pangolin_ui/src/lib/api/auth.ts`

```typescript
export interface LoginRequest {
  username: string;
  password: string;
  "tenant-id": string | null;  // CRITICAL: kebab-case!
}

export async function login(username: string, password: string, tenantId: string | null = null) {
  const response = await fetch(`${API_URL}/api/v1/users/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      username,
      password,
      "tenant-id": tenantId  // kebab-case!
    })
  });
  // ... handle response
}
```

#### 2. Add Tenant Selector to Login Page
**File**: `pangolin_ui/src/routes/login/+page.svelte`

```svelte
<script lang="ts">
  import { goto } from '$app/navigation';
  import { login } from '$lib/api/auth';
  
  let username = '';
  let password = '';
  let selectedTenant: string | null = null;
  let tenants: Tenant[] = [];
  let showTenantSelector = false;
  
  // Fetch tenants for selector (optional - can be hidden for Root login)
  async function loadTenants() {
    // Fetch from /api/v1/tenants (public endpoint)
  }
  
  async function handleLogin() {
    try {
      await login(username, password, selectedTenant);
      goto('/dashboard');
    } catch (error) {
      // Handle error
    }
  }
</script>

<form on:submit|preventDefault={handleLogin}>
  <input bind:value={username} placeholder="Username" />
  <input bind:value={password} type="password" placeholder="Password" />
  
  <label>
    <input type="checkbox" bind:checked={showTenantSelector} />
    Tenant-specific login
  </label>
  
  {#if showTenantSelector}
    <select bind:value={selectedTenant}>
      <option value={null}>Root User</option>
      {#each tenants as tenant}
        <option value={tenant.id}>{tenant.name}</option>
      {/each}
    </select>
  {/if}
  
  <button type="submit">Login</button>
</form>
```

#### 3. Create Tenant-Specific Login Routes
**File**: `pangolin_ui/src/routes/login/[tenant]/+page.svelte`

```svelte
<script lang="ts">
  import { page } from '$app/stores';
  import { login } from '$lib/api/auth';
  
  const tenantId = $page.params.tenant;
  
  // Pre-fill tenant ID for this route
  async function handleLogin() {
    await login(username, password, tenantId);
  }
</script>
```

**Route**: `/login/{tenant-uuid}` - Direct link for tenant-specific login

---

## 🟡 Priority: MEDIUM - Dashboard Tenants Count

### Issue
Dashboard doesn't display `tenants_count` field for Root users.

### Required Changes

#### Update Dashboard Stats Interface
**File**: `pangolin_ui/src/lib/api/dashboard.ts`

```typescript
export interface DashboardStats {
  catalogs_count: number;
  warehouses_count: number;
  tenants_count?: number;  // New field
  tables_count: number;
  users_count: number;
  branches_count: number;
  namespaces_count: number;
  scope: string;  // "system" or "tenant"
}
```

#### Update Dashboard Display
**File**: `pangolin_ui/src/routes/dashboard/+page.svelte`

```svelte
<script lang="ts">
  import { dashboardStats } from '$lib/api/dashboard';
  
  let stats: DashboardStats;
</script>

<div class="stats-grid">
  {#if stats.scope === 'system'}
    <div class="stat-card">
      <h3>Tenants</h3>
      <p class="stat-value">{stats.tenants_count ?? 0}</p>
    </div>
  {/if}
  
  <div class="stat-card">
    <h3>Catalogs</h3>
    <p class="stat-value">{stats.catalogs_count}</p>
  </div>
  <!-- ... other stats -->
</div>

<p class="scope-indicator">Scope: {stats.scope}</p>
```

---

## 🟡 Priority: MEDIUM - Asset ID Integration

### Issue
Table explorer doesn't use asset ID from table response to fetch business metadata.

### Required Changes

#### 1. Update TableResponse Interface
**File**: `pangolin_ui/src/lib/api/iceberg.ts`

```typescript
export interface TableResponse {
  id?: string;  // New field - asset UUID
  metadata: TableMetadata;
  "metadata-location"?: string;
  config?: Record<string, string>;
}
```

#### 2. Use Asset ID in Table Explorer
**File**: `pangolin_ui/src/routes/explorer/[catalog]/[namespace]/[table]/+page.svelte`

```svelte
<script lang="ts">
  import { loadTable } from '$lib/api/iceberg';
  import { getAsset } from '$lib/api/assets';
  
  let tableResponse: TableResponse;
  let assetMetadata: Asset | null = null;
  
  async function loadTableData() {
    tableResponse = await loadTable(catalog, namespace, table);
    
    // Fetch business metadata using asset ID
    if (tableResponse.id) {
      assetMetadata = await getAsset(tableResponse.id);
    }
  }
</script>

{#if assetMetadata}
  <div class="asset-metadata">
    <h3>Business Metadata</h3>
    <p>Owner: {assetMetadata.owner}</p>
    <p>Description: {assetMetadata.description}</p>
    <!-- ... other metadata -->
  </div>
{/if}
```

---

## 🟢 Priority: LOW - Audit Log Cross-Tenant Filtering

### Issue
Audit log UI doesn't support cross-tenant queries or filtering for Root users.

### Required Changes

**File**: `pangolin_ui/src/routes/audit/+page.svelte`

```svelte
<script lang="ts">
  import { listAuditEvents } from '$lib/api/audit';
  import { session } from '$lib/stores/auth';
  
  let selectedTenant: string | null = null;
  let events: AuditLogEntry[] = [];
  
  async function loadEvents() {
    events = await listAuditEvents({
      tenant_id: selectedTenant,  // Filter by tenant
      limit: 50
    });
  }
</script>

{#if $session.role === 'root'}
  <select bind:value={selectedTenant} on:change={loadEvents}>
    <option value={null}>All Tenants</option>
    {#each tenants as tenant}
      <option value={tenant.id}>{tenant.name}</option>
    {/each}
  </select>
{/if}

<table>
  <thead>
    <tr>
      {#if $session.role === 'root'}
        <th>Tenant</th>
      {/if}
      <th>Action</th>
      <th>User</th>
      <th>Timestamp</th>
    </tr>
  </thead>
  <tbody>
    {#each events as event}
      <tr>
        {#if $session.role === 'root'}
          <td>{event.tenant_id}</td>
        {/if}
        <td>{event.action}</td>
        <td>{event.user_id}</td>
        <td>{event.timestamp}</td>
      </tr>
    {/each}
  </tbody>
</table>
```

---

## Implementation Checklist

### Tenant-Scoped Login
- [ ] Update `LoginRequest` interface with `tenant-id` field (kebab-case!)
- [ ] Add tenant selector to login page
- [ ] Create `/login/[tenant]` route for direct tenant login
- [ ] Update `login()` function to accept tenant ID
- [ ] Test Root login (tenant-id: null)
- [ ] Test tenant-scoped login with duplicate usernames
- [ ] Add error handling for invalid tenant ID

### Dashboard
- [ ] Add `tenants_count` to `DashboardStats` interface
- [ ] Display tenant count for Root users only
- [ ] Show scope indicator (system vs tenant)
- [ ] Test with Root and non-Root users

### Asset ID
- [ ] Add `id` field to `TableResponse` interface
- [ ] Fetch asset metadata using table response ID
- [ ] Display business metadata in table explorer
- [ ] Add link to asset detail page

### Audit Log
- [ ] Add tenant filter dropdown for Root users
- [ ] Update event list to show tenant column
- [ ] Test cross-tenant queries
- [ ] Add tenant name resolution

---

## Testing Requirements

### Manual Testing
1. Test Root login without tenant selection
2. Test tenant-scoped login with tenant selector
3. Create users with duplicate usernames in different tenants
4. Verify login works for both users
5. Test dashboard displays tenant count for Root
6. Test asset ID linking in table explorer
7. Test audit log tenant filtering

### E2E Tests
- Login flow with tenant selection
- Dashboard stats display for Root vs non-Root
- Asset metadata fetching
- Audit log filtering

---

## Design Considerations

### Tenant Selector UX
- **Option 1**: Checkbox to show/hide tenant selector (simpler for Root users)
- **Option 2**: Always show tenant dropdown with "Root User" as default
- **Option 3**: Separate login pages: `/login` (Root) and `/login/tenant` (Tenant)

**Recommendation**: Option 1 - keeps Root login simple while supporting tenant-scoped login.

### Error Messages
- "Invalid tenant ID" - when tenant doesn't exist
- "User not found in this tenant" - when username exists but in different tenant
- "Invalid credentials" - when password is wrong

---

## Notes

### Critical: JSON Field Naming
**LoginRequest uses kebab-case**: `tenant-id` NOT `tenant_id`

```typescript
// CORRECT
{
  "username": "user",
  "password": "pass",
  "tenant-id": "uuid"
}

// WRONG - will be ignored!
{
  "username": "user",
  "password": "pass",
  "tenant_id": "uuid"
}
```

### Existing UI Audit Documents
- [`tenant_scoped_login_ui.md`](file:///home/alexmerced/development/personal/Personal/2026/pangolin/planning/tenant_scoped_login_ui.md) - Detailed UI requirements
- [`asset_id_ui.md`](file:///home/alexmerced/development/personal/Personal/2026/pangolin/planning/asset_id_ui.md) - Asset ID integration guide
