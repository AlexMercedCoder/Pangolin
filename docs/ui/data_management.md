# Data Management

This guide covers how to browse, version, and maintain your data assets through the Pangolin UI.

## 📁 Data Explorer

The Explorer is the heart of the Pangolin user experience, providing a hierarchical "Finder-like" view of your lakehouse.

### Hierarchy Navigation
- **Catalogs**: Expand to see namespaces.
- **Namespaces**: Recursive folders containing tables, views, or further sub-namespaces.
- **Assets**: Tables and Views are represented with distinct icons.

### Table Detail View
Clicking an asset opens a comprehensive detail pane:
- **Schema**: Columns, types, and nullability.
- **Snapshots**: A chronological list of table commits.
- **Properties**: Key-value pairs defining table behavior.
- **Maintenance**: One-click actions for table health (e.g., Compaction, Snapshot Expiration).

### 🗑️ Bulk Operations
- **Asset List**: View all tables and namespaces within a catalog in a searchable list.
- **Bulk Delete**: Select multiple assets to delete them in a single batch operation with progress tracking.

### ✅ Smart Validation
- **Name Validation**: Creation forms (Catalogs, Warehouses) automatically check for duplicate names in real-time, preventing errors before submission.

---

## 🌿 Versioning (Git-for-Data)

Pangolin brings version control to your data via a dedicated UI for branches and tags.

### Branch Management
- **List Branches**: See all branches for a specific catalog.
- **Create Branch**: Spin up a new isolated environment (e.g., `feature/cleanup`) from any existing commit or branch.
- **Delete Branch**: Remove experimental environments once they are no longer needed.

### Tag Management
Create permanent, named aliases for specific commits (e.g., `v1_analytics_gold`) to ensure reproducibility in reporting.

---

## 🔀 Merge Operations

Manage complex data integration workflows with the Merge UI.

### Initiate Merge
Trigger a merge from one branch into another (e.g., `ingest` -> `main`).

### Conflict Resolution
If Pangolin detects changes to the same metadata in both branches, the UI enters **Resolution Mode**:
- **Conflict List**: See exactly which assets or properties are in conflict.
- **Visual Diff**: Compare values from the Source, Target, and Base (3-way merge).
- **Resolver**: Select a winner (Source or Target) or apply a manual fix.

---

## 🛠️ Table Maintenance

Ensure your lakehouse stays performant without manual scripting.

- **Compaction**: Merge small files to improve query performance.
- **Manifest Rewriting**: Optimize metadata structures.
- **Snapshot Expiration**: Remove old data versions to save storage costs.

*All maintenance tasks can be triggered manually from the Table Detail header or scheduled (future feature).*
