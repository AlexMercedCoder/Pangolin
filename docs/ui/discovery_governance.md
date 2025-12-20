# Discovery & Governance

Pangolin provides a transparent and searchable view of your data estate, combined with rigorous auditing and access governance.

## 🔍 Discovery Portal

The Discovery view is designed for data consumers to find and understand assets without needing deep technical knowledge of the catalog hierarchy.

### Search & Filtering
- **Fuzzy Search**: Search by asset name, description, or owner.
- **Tag Filtering**: Filter assets by business tags (e.g., `PII`, `Financial`, `Certified`).
- **Full Qualified Names**: Search results display the full `catalog.namespace.table` for clarity.

### Asset Cards
Each search result presents a card with:
- **Status Indicators**: Indicates if the data is currently accessible to you.
- **Business Metadata**: Descriptions and ownership info.
- **Schema Preview**: Quick glimpse at the top columns.

---

## 🙋 Access Request Workflow

For assets you discover but do not have permissions to read, Pangolin provides a self-service workflow.

1. **Request Access**: From the Discovery card, click "Request Access".
2. **Justification**: Provide a reason for the request.
3. **Admin Review**: Tenant Admins see a notification and a dedicated "Requests" dashboard to Approve or Deny.
4. **Approval**: Upon approval, Pangolin automatically grants the necessary RBAC permissions.

---

## 📜 Audit Log Viewer

The Audit Log provides a complete, immutable history of all actions performed in both the UI and via the API.

### Features
- **Global View**: Admins can see events across the entire tenant.
- **Details Modal**: Click an event to see the raw JSON payload, Actor ID, Client IP, and Timestamp.
- **Filtering**: (Upcoming) Filter by Action Type (e.g., `Asset_Create`) or User ID.

### Logged Actions
Pangolin logs everything from login attempts to table deletions and permission grants, ensuring compliance and security.
