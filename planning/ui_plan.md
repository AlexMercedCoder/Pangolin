# UI Implementation Plan: Core CRUD Operations

## Goal
Implement full Create, Read, Update, Delete (CRUD) operations for the core entities (Tenants, Users, Warehouses, Catalogs) in the Pangolin Management UI, ensuring role-based access control (Root vs. Tenant Admin) is respected.

## Prerequisites
- [x] Backend API endpoints for CRUD (Completed)
- [x] Root User Restrictions (Completed - Root can only see Tenants)
- [x] Basic UI Layout & Authentication (Completed)

## 1. Tenant Management (Root Only)
**View**: `/tenants`
- [ ] **List**: Display table of tenants (ID, Name, Created At).
- [ ] **Create**: Modal/Page to create a new tenant (Name).
- [ ] **Delete**: Button to delete a tenant (with confirmation).
- [ ] **Switching**: "Use Tenant" button to switch active context (simulated or explicit).

## 2. User Management (Tenant Admin)
**View**: `/users`
- [ ] **List**: Display users for current tenant.
- [ ] **Create**: Form to add Tenant User (Username, Email, Role: Tenant User).
    - *Note: Tenant Admins created by Root should be visible?*
- [ ] **Delete**: Remove user access.

## 3. Warehouse Management (Tenant Admin)
**View**: `/warehouses`
- [ ] **List**: Card/Table view of registered warehouses.
- [ ] **Create**: Form for S3/Memory/FS warehouse details (Bucket, Keys, Region, Endpoint).
- [ ] **Delete**: Remove warehouse registration.

## 4. Catalog Management (Tenant Admin)
**View**: `/catalogs`
- [ ] **List**: Display catalogs.
- [ ] **Create**: Form to link a Catalog Name to a Warehouse.
- [ ] **Delete**: Remove catalog.

## 5. Data Explorer (Tenant Admin / User)
**View**: `/explorer`
- [x] **Hierarchy**: Browse Catalogs -> Namespaces -> Tables.
- [x] **Create Namespace**: Modal to create nested namespaces.
- [x] **Create Table**: Modal with Schema Builder (Columns, Types).
- [ ] **Table Details**: View Schema, Snapshots, Metadata.
- [ ] **Query**: Simple SQL query interface (Future).

## Technical Approach
- Use SvelteKit Loaders for fetching lists.
- Use Form Actions or API calls for Create/Delete operations.
- Implement shared `ConfirmModal.svelte` component.
- Ensure error handling (403 Forbidden, 500 Error) wraps UI elements or redirects appropriately.
