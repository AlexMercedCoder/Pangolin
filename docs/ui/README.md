# Pangolin Management UI

Welcome to the documentation for the Pangolin Management UI. This web interface is built with SvelteKit and provides a comprehensive suite of tools for managing your data lakehouse.

## 🗺️ Navigation Map

### [General Overview](./overview.md)
Start here to understand the interface layout, theme switching, and the core dashboard experience.

### [Data Management](./data_management.md)
Learn how to browse your data, manage branches (Git-for-Data), and handle merge operations.
- Data Explorer
- Branching & Tagging
- Merge Operations & Conflict Resolution
- Table Maintenance

### [Discovery & Governance](./discovery_governance.md)
Tools for finding data and auditing system activity.
- Data Discovery Portal
- Access Request Workflow
- Audit Log Viewer

### [Administration](./administration.md)
Configure the foundational resources of your Pangolin instance.
- Multi-Tenant Management
- Warehouse & Catalog Configuration
- Service Users & API Keys

### [Security](./security.md)
Manage who can access what.
- Users & Roles
- Granular Permissions UI
- Token Management (JWT)

## ⚙️ Configuration & Development

### Local Development Setup
When running the UI locally against a backend services, ensure your `vite.config.ts` proxy matches your backend port (default: 8080).

**Common Pitfall:** The backend defaults to port `8080`. If `vite.config.ts` proxies to `8085`, you may encounter 500 errors or connection refusals. Ensure the proxy target matches:
```typescript
server: {
    proxy: {
        '/api': 'http://localhost:8080', // Match backend port
        '/v1': 'http://localhost:8080',
        '/oauth': 'http://localhost:8080'
    }
}
```
