# UI Parity Implementation Plan (Merge Operations)

## Goal
Achieve 100% UI Parity by implementing the missing **Merge Operations** workflow. This will allow users to merge branches, view conflicts, and resolve them directly from the Pangolin UI.

## Current Gap
- **Backend:** Fully supports `create_merge_operation`, `list_conflicts`, `resolve_conflict`, `complete_merge`.
- **UI:** No visual interface for initiating or managing merges.

## Implementation Steps

### 1. Merge Request Initiation
**Location:** `pangolin_ui/src/routes/catalogs/[name]/branches/+page.svelte` (or a specific Branch Detail page)

- **UI Element:** Add a "Merge" button to each Branch card (or the top header if viewing a specific branch).
- **Interaction:**
    - Button click opens a Validated Form / Modal.
    - **Inputs:**
        - `Target Branch` (Dropdown, exclude current branch).
    - **Action:** Calls `POST /api/v1/branches/merge` with `dry_run=true` (or creates the operation which essentially is the dry run until completed).

### 2. Merge Operation Detail / Conflict Resolution
**Location:** New Route `pangolin_ui/src/routes/catalogs/[name]/merges/[operation_id]/+page.svelte`

- **Purpose:** View the status of a specific merge operation.
- **Components:**
    - **Header:** Source -> Target, Status (Open, Conflicted, Ready, Completed).
    - **Conflict List:** If `status == Conflicted`:
        - List all `MergeConflict` items.
        - Show `ConflictType` (Schema, Metadata, etc.).
        - Show "Resolve" action.
    - **Resolution Modal:**
        - Display conflict details (Diff if possible).
        - Options: "Use Source", "Use Target".
        - Calls `POST /api/v1/conflicts/{id}/resolve`.

### 3. Completion
- **Action:** Once all conflicts are resolved (or if none existed), enable "Complete Merge" button.
- **API:** Calls `POST /api/v1/merge-operations/{id}/complete`.
- **Feedback:** Redirect to Branch list with success toast.

## Technical Details

### API Client Updates
- Ensure `pangolin_ui/src/lib/api.ts` has methods for:
    - `createMergeOperation`
    - `getMergeOperation`
    - `listMergeConflicts`
    - `resolveConflict`
    - `completeMerge`

### Components
- `MergeStatusBadge.svelte`
- `ConflictResolver.svelte`

## Verification Plan
1.  **Manual Test:** Create a branch `dev`, modify a table.
2.  **Initiate Merge:** Use UI to merge `dev` -> `main`.
3.  **Conflict Test:** (Optional) Create a conflict via CLI/Script, then use UI to resolve it.
4.  **Completion:** Verify `main` is updated after UI completion.
