# Permissions System

Pangolin implements a flexible Role-Based Access Control (RBAC) system enhanced with Tag-Based Access Control (TBAC) and specific logic for Branching strategies.

## 0. User Roles & The Root User

Pangolin strictly separates **Platform Administration** from **Data Governance**.

### Root User
The `Root` user is a pure **Platform Administrator**. Their capabilities are strictly limited to multi-tenancy management.

-   **Allowed**: Create Tenant, Create Initial Tenant Admin.
-   **Forbidden**: Create Warehouse, Create Catalog, Manage Data, Grant Granular Permissions, Create regular Tenant Users.
-   **Context**: The Root user cannot "switch context" to a tenant to view or modify data.

### Tenant Admin
The `TenantAdmin` is the **Data Governor** for their specific tenant.

-   **Allowed**: Create Warehouses, Catalogs, Namespaces, Tenant Users.
-   **Responsibility**: Manage all data assets and permissions within their tenant.


## 1. Role-Based Access Control (RBAC)

Permissions are assigned to **Users** directly or via **Roles** (which group permissions for easier management).

### Scope Hierarchy
Permissions can be granted at different levels of the hierarchy:
1.  **Catalog**: Applies to the entire catalog.
2.  **Namespace**: Applies to a namespace and all its tables.
3.  **Asset (Table/View)**: Applies to a specific table or view.

### Standard Actions
- `read`: Read data and metadata.
- `write`: specific modifications.
- `create`/`delete`: Lifecycle management.
- `all`: Full control.

## 2. Tag-Based Access Control (TBAC)

Tag-based permissions allow you to enforce security policies across disparate assets based on their classification (e.g., "PII", "Sensitive", "Finance").

### How it Works
1.  **Tagging**: An admin adds a tag (e.g., `PII`) to an asset's business metadata.
2.  **Granting**: A Role is granted permission on the **Tag Scope** (e.g., `PermissionScope::Tag { tag_name: "PII" }`).
3.  **Enforcement**: Any user with that Role can access *any* asset that has the `PII` tag, regardless of their direct table permissions.

*Note: Tag permissions are additive. If a user has `read` access to a Namespace but an asset in it is tagged `Confidential`, they might ALSo need the `Confidential` tag permission depending on your policy configuration (typically Pangolin permissions are additive: if you have ANY valid path to access, you get access).*

## 3. Branching Permissions

Pangolin treats branches as first-class citizens with their own permission requirements, specifically distinguishing between "Experimental" and "Ingestion" workflows.

### Branch Types & Permissions
-   **Experimental Branches**: Used for "what-if" analysis.
    -   **Action Required**: `experimental_branching`
    -   **Scope**: `Catalog` (User needs this permission on the Catalog to create experimental branches).
-   **Ingest Branches**: Used for data ingestion pipelines (mergable).
    -   **Action Required**: `ingest_branching`
    -   **Scope**: `Catalog`.

### Why the distinction?
-   **Data Engineers** conducting experiments should not accidentally create merge-ready branches that could corrupt the main data lake.
-   **ETL Service Accounts** need rights to create Ingest branches but may not need to run ad-hoc experiments.

## 4. Best Practices
-   **Least Privilege**: Grant permissions at the lowest necessary scope (Asset > Namespace > Catalog).
-   **Use Roles**: Avoid assigning direct permissions to users. Create functional roles (e.g., `FinanceAnalyst`, `DataEngineer`) and assign those.
-   **Leverage Tags**: Use tags for cross-functional governance (e.g., GDPR compliance) rather than managing thousands of individual table grants.
