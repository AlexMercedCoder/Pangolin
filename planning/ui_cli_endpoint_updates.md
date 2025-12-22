# UI and CLI Updates for New Backend Endpoints

## Overview

The following UI and CLI updates are needed to integrate the newly implemented backend endpoint optimizations.

---

## UI Updates

### 1. Dashboard Page

**New Component**: `src/routes/dashboard/+page.svelte`

**API Integration**:
```typescript
import { dashboardApi } from '$lib/api/dashboard';

const stats = await dashboardApi.getStats();
```

**Display**:
- Catalogs count with link to catalogs page
- Tables/namespaces counts
- Warehouses count
- User count (for admins)
- Scope indicator (system/tenant/user)

---

### 2. Catalog Overview Page

**Update**: `src/routes/catalogs/[name]/+page.svelte`

**API Integration**:
```typescript
const summary = await catalogsApi.getSummary(catalogName);
```

**Display**:
- Namespace count
- Table count  
- Branch count
- Storage location
- Quick stats cards

---

### 3. Global Search Component

**New Component**: `src/lib/components/search/GlobalSearch.svelte`

**API Integration**:
```typescript
import { searchApi } from '$lib/api/search';

const results = await searchApi.searchAssets({
  q: searchQuery,
  catalog: selectedCatalog,
  limit: 50
});
```

**Features**:
- Search input in navbar
- Dropdown results with highlighting
- Pagination for large result sets
- Filter by catalog

---

### 4. Bulk Delete Dialog

**New Component**: `src/lib/components/dialogs/BulkDeleteDialog.svelte`

**API Integration**:
```typescript
const result = await optimizationApi.bulkDeleteAssets(selectedAssetIds);
```

**Features**:
- Multi-select in asset lists
- Confirmation dialog with count
- Progress indicator
- Error reporting for failed deletions

---

### 5. Name Validation in Forms

**Update**: `src/lib/components/forms/CreateCatalogForm.svelte`

**API Integration**:
```typescript
const validation = await optimizationApi.validateNames({
  resource_type: 'catalog',
  names: [catalogName]
});
```

**Features**:
- Real-time validation as user types
- Visual feedback (green checkmark / red X)
- Error message display

---

## CLI Updates

### 1. Stats Command

**Admin CLI**: `pangolin-admin stats`

**Implementation**:
```rust
async fn handle_stats(client: &PangolinClient) -> Result<()> {
    let resp = client.get("/api/v1/dashboard/stats").await?;
    let stats: DashboardStats = resp.json().await?;
    
    println!("Dashboard Statistics ({})", stats.scope);
    println!("  Catalogs: {}", stats.catalogs_count);
    println!("  Warehouses: {}", stats.warehouses_count);
    println!("  Namespaces: {}", stats.namespaces_count);
    println!("  Tables: {}", stats.tables_count);
    
    Ok(())
}
```

---

### 2. Catalog Summary Command

**Admin CLI**: `pangolin-admin catalog summary <name>`

**Implementation**:
```rust
async fn handle_catalog_summary(
    client: &PangolinClient,
    name: String
) -> Result<()> {
    let resp = client.get(&format!("/api/v1/catalogs/{}/summary", name)).await?;
    let summary: CatalogSummary = resp.json().await?;
    
    println!("Catalog: {}", summary.name);
    println!("  Namespaces: {}", summary.namespace_count);
    println!("  Tables: {}", summary.table_count);
    println!("  Branches: {}", summary.branch_count);
    println!("  Storage: {}", summary.storage_location.unwrap_or_default());
    
    Ok(())
}
```

---

### 3. Search Command

**Admin CLI**: `pangolin-admin search <query> [--catalog <name>] [--limit <n>]`

**Implementation**:
```rust
async fn handle_search(
    client: &PangolinClient,
    query: String,
    catalog: Option<String>,
    limit: Option<usize>
) -> Result<()> {
    let mut url = format!("/api/v1/search/assets?q={}", query);
    if let Some(cat) = catalog {
        url.push_str(&format!("&catalog={}", cat));
    }
    if let Some(lim) = limit {
        url.push_str(&format!("&limit={}", lim));
    }
    
    let resp = client.get(&url).await?;
    let results: SearchResponse = resp.json().await?;
    
    println!("Found {} results:", results.total);
    for result in results.results {
        println!("  {}.{}.{}", 
            result.catalog, 
            result.namespace.join("."),
            result.name
        );
    }
    
    Ok(())
}
```

---

### 4. Bulk Delete Command

**Admin CLI**: `pangolin-admin bulk-delete --ids <uuid1,uuid2,...>`

**Implementation**:
```rust
async fn handle_bulk_delete(
    client: &PangolinClient,
    ids: Vec<String>
) -> Result<()> {
    let payload = json!({ "asset_ids": ids });
    let resp = client.post("/api/v1/bulk/assets/delete", &payload).await?;
    let result: BulkOperationResponse = resp.json().await?;
    
    println!("Bulk delete completed:");
    println!("  Succeeded: {}", result.succeeded);
    println!("  Failed: {}", result.failed);
    
    if !result.errors.is_empty() {
        println!("Errors:");
        for error in result.errors {
            println!("  - {}", error);
        }
    }
    
    Ok(())
}
```

---

### 5. Validate Names Command

**Admin CLI**: `pangolin-admin validate <type> <name1> [name2...]`

**Implementation**:
```rust
async fn handle_validate_names(
    client: &PangolinClient,
    resource_type: String,
    names: Vec<String>
) -> Result<()> {
    let payload = json!({
        "resource_type": resource_type,
        "names": names
    });
    
    let resp = client.post("/api/v1/validate/names", &payload).await?;
    let result: ValidateNamesResponse = resp.json().await?;
    
    println!("Name validation results:");
    for validation in result.results {
        let status = if validation.available { "✓ Available" } else { "✗ Taken" };
        println!("  {}: {}", validation.name, status);
        if let Some(reason) = validation.reason {
            println!("    Reason: {}", reason);
        }
    }
    
    Ok(())
}
```

---

## API Client Updates

### New TypeScript API Modules

**File**: `src/lib/api/dashboard.ts`
```typescript
export const dashboardApi = {
  async getStats(): Promise<DashboardStats> {
    const response = await apiClient.get<DashboardStats>('/api/v1/dashboard/stats');
    if (response.error) throw new Error(response.error.message);
    return response.data!;
  }
};
```

**File**: `src/lib/api/search.ts`
```typescript
export const searchApi = {
  async searchAssets(params: SearchParams): Promise<SearchResponse> {
    const query = new URLSearchParams(params as any).toString();
    const response = await apiClient.get<SearchResponse>(`/api/v1/search/assets?${query}`);
    if (response.error) throw new Error(response.error.message);
    return response.data!;
  }
};
```

**File**: `src/lib/api/optimization.ts`
```typescript
export const optimizationApi = {
  async bulkDeleteAssets(assetIds: string[]): Promise<BulkOperationResponse> {
    const response = await apiClient.post<BulkOperationResponse>(
      '/api/v1/bulk/assets/delete',
      { asset_ids: assetIds }
    );
    if (response.error) throw new Error(response.error.message);
    return response.data!;
  },
  
  async validateNames(request: ValidateNamesRequest): Promise<ValidateNamesResponse> {
    const response = await apiClient.post<ValidateNamesResponse>(
      '/api/v1/validate/names',
      request
    );
    if (response.error) throw new Error(response.error.message);
    return response.data!;
  }
};
```

---

## Priority

### High Priority (Immediate Value)
1. Dashboard stats integration
2. Global search component
3. Name validation in forms

### Medium Priority
4. Catalog summary page
5. Bulk delete dialog

### Low Priority (Nice to Have)
6. CLI commands (can be added incrementally)

---

## Testing Checklist

### UI Testing
- [ ] Dashboard displays correct stats for different user roles
- [ ] Search returns relevant results
- [ ] Bulk delete handles errors gracefully
- [ ] Name validation provides real-time feedback
- [ ] Catalog summary shows accurate counts

### CLI Testing
- [ ] Stats command works in interactive and non-interactive modes
- [ ] Search command supports all parameters
- [ ] Bulk delete requires confirmation
- [ ] Validate command handles multiple names

---

## Notes

- All endpoints support NO_AUTH mode for testing
- Error responses are consistent JSON format
- Pagination is supported where applicable
- All endpoints respect tenant isolation
