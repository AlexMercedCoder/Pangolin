# UI Requirements: Asset ID in Table Response

## Overview
The backend now includes the Pangolin asset UUID (`id`) in Iceberg table responses. This allows the UI to link table metadata to business metadata (tags, descriptions, etc.).

## Backend Changes
- `TableResponse` struct includes `id: Option<Uuid>` field
- `load_table` handler populates `id` from asset
- Field is optional and skipped if null (for compatibility)

## Frontend Implementation Required

### 1. Update Type Definitions

**File**: `pangolin_ui/src/lib/api/iceberg.ts`

Update `TableResponse` interface:
```typescript
export interface TableResponse {
  'metadata-location'?: string;
  metadata: TableMetadata;
  config?: Record<string, string>;
  id?: string;  // NEW: Pangolin asset UUID
}
```

### 2. Update Table Detail Component

**File**: `pangolin_ui/src/routes/explorer/[catalog]/[namespace]/[table]/+page.svelte`

Use the `id` field to fetch business metadata:
```typescript
async function loadTableDetails() {
  const tableResponse = await icebergApi.loadTable(catalog, namespace, table);
  
  // Use asset ID to fetch business metadata
  if (tableResponse.id) {
    const businessMetadata = await businessMetadataApi.get(tableResponse.id);
    // ... display tags, description, etc.
  }
}
```

### 3. Business Info Tab

The "Business Info" tab in the table explorer can now:
- Fetch business metadata using `tableResponse.id`
- Display tags, description, owner, etc.
- Allow editing business metadata (if user has permissions)

**Example**:
```svelte
{#if tableResponse?.id}
  <BusinessMetadataPanel assetId={tableResponse.id} />
{:else}
  <p>Business metadata not available for this table</p>
{/if}
```

## Benefits

- **Seamless Integration**: No need to re-query for asset ID
- **Performance**: Single API call returns both Iceberg and Pangolin metadata
- **Backward Compatible**: Optional field doesn't break existing clients

## Testing Checklist

- [ ] Table load returns `id` field
- [ ] Business metadata can be fetched using `id`
- [ ] Business Info tab displays correctly
- [ ] Editing business metadata works
- [ ] Works for both Iceberg tables and generic assets
