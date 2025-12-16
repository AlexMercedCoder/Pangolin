# General Overview

The **Pangolin Management UI** is the visual control center for your lakehouse. It provides a modern interface for managing tenants, users, access controls, and data assets.

## Core Concepts

### Authentication & Access
- **Login**: Users authenticate via JWT (Username/Password) or OAuth.
- **Tenant Context**: Upon login, users are scoped to their home tenant.
- **Role-Based Access**: The UI dynamically adjusts based on your role:
    - **Root Users**: Access to global system management (Tenants).
    - **Tenant Admins**: Full management of a specific tenant (Users, Warehouses, Catalogs).
    - **Tenant Users**: Focused on data discovery and usage (Explorer).

## Navigation Layout

### 1. Root Dashboard
*Visible to specific Root users only.*
- **Tenants**: Manage multi-tenant isolation.
- **System Settings**: Global configuration.

### 2. Tenant Dashboard
*The primary view for most users.*
- **Catalogs**: Manage Iceberg catalogs.
- **Warehouses**: Configure storage backends (S3, GCS, Azure).
- **Users & Roles**: Manage team access.
- **Data Explorer**: Browse and query data.

## Getting Started
1. **Log In**: Use your credentials provided by the admin.
2. **Select Context**: If you belong to multiple tenants (future feature), select your working tenant.
3. **Explore Data**: Navigate to the **Explorer** tab to browse catalogs.
4. **Manage Resources**: Use the sidebar to access administrative functions.
