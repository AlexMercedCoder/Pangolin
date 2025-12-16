# Data Explorer

The **Data Explorer** provides a hierarchical view of your data assets, allowing you to browse, inspect, and manage resources directly from the Pangolin UI.

## Features

### 1. Unified Hierarchy
Browse the full structure of your lakehouse:
- **Catalogs**: Top-level containers (e.g., `prod`, `dev`, `analytics`).
- **Namespaces**: Logical groupings (e.g., `finance.reports`).
- **Tables**: Iceberg tables with full details.

### 2. Table Details
Inspect table metadata without writing queries:
- **Schema**: View columns, types, and documentation.
- **snapshots**: Track history of table operations (append, overwrite).
- **Metadata**: View table properties and configuration.

### 3. Creation Workflows
Create resources via intuitive dialogs:
- **Create Namespace**: Support for nested namespaces (e.g., `a.b.c`).
- **Create Table**: Visual **Schema Builder** to define columns, types, and required fields.

### 4. Real-time Refresh
The Explorer Tree automatically refreshes when you create new resources, ensuring your view is always up-to-date. `Catalog` and `Content` updates are intelligently separated to minimize reloading.

### 5. Access Control
The Explorer respects RBAC:
- **Tenant Admins**: Full creation and viewing rights.
- **Tenant Users**: Read-only access to allowed resources (Creation buttons are hidden or disabled based on backend permissions).
