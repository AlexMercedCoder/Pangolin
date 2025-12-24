# UI Requirements: Tenant-Scoped Login

## Overview
The backend now supports tenant-scoped authentication via an optional `tenant_id` parameter in the login request. This resolves username collision issues across tenants.

## Backend Changes
- `LoginRequest` now accepts optional `tenant_id: Option<Uuid>`
- Login logic validates user belongs to specified tenant
- Root users must login with `tenant_id: null`
- Tenant users must login with their `tenant_id`

## Frontend Implementation Required

### 1. Update Login API Client

**File**: `pangolin_ui/src/lib/api/auth.ts`

Update `LoginRequest` interface:
```typescript
export interface LoginRequest {
  username: string;
  password: string;
  tenant_id?: string;  // Optional UUID
}
```

### 2. Create Tenant-Specific Login Routes

**New Files**:
- `pangolin_ui/src/routes/login/[tenant]/+page.svelte`

**Route Structure**:
- `/login/root` - Root user login (tenant_id = null)
- `/login/{tenant-uuid}` - Tenant-specific login

**Implementation**:
```svelte
<script lang="ts">
  import { page } from '$app/stores';
  
  $: tenantParam = $page.params.tenant || '';
  $: isRootLogin = tenantParam === 'root';
  $: tenantId = isRootLogin ? null : tenantParam;
  
  async function handleLogin() {
    const result = await authStore.login(username, password, tenantId);
    // ... handle result
  }
</script>
```

### 3. Update Auth Store

**File**: `pangolin_ui/src/lib/stores/auth.ts`

Update `login` method signature:
```typescript
async login(username: string, password: string, tenantId?: string | null) {
  const response = await authApi.login({ 
    username, 
    password,
    tenant_id: tenantId 
  });
  // ... rest of logic
}
```

### 4. Update Main Login Page

**File**: `pangolin_ui/src/routes/login/+page.svelte`

Add tenant selector or redirect to tenant-specific login:
- Option A: Show tenant dropdown for user to select
- Option B: Redirect to `/login/root` by default with link to tenant login

## Error Messages

The backend returns specific error messages:
- `"Invalid credentials for this tenant"` - User exists but in different tenant
- `"Root users must login without tenant_id"` - Root user tried tenant-scoped login
- `"Tenant users must login with tenant_id"` - Tenant user tried root login
- `"User not found in this tenant"` - Username doesn't exist in specified tenant

## Testing Checklist

- [ ] Root login via `/login/root` works
- [ ] Tenant admin login via `/login/{tenant-id}` works
- [ ] Username collision resolved (same username in different tenants)
- [ ] Appropriate error messages displayed
- [ ] Session persists correctly after tenant-scoped login
