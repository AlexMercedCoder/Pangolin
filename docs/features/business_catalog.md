# Business Catalog & Discovery

Pangolin goes beyond technical metadata by providing a rich **Business Catalog** to help users find, understand, and access data.

## features

### 1. Metadata Management
Annotate your data assets with meaningful context:
- **Descriptions**: Rich text documentation for tables and columns.
- **Tags**: Categorize assets (e.g., `PII`, `Financial`, `Gold`).
- **Properties**: Custom key-value pairs for flexible metadata.

### 2. Discovery Portal
The **Discovery Portal** (`/discovery`) is a search-centric interface designed for data consumers.
- **Full Text Search**: Find tables by name, description, or tags.
- **Filtering**: Narrow results by Catalog, Namespace, or Tags.
- **Discoverability**: Assets can be marked as "Discoverable" to appear in global searches even if the user doesn't have explicit read access.

### 3. Access Request Workflow
Streamline data access governance:
- **Request Access**: Users can request access to discoverable tables directly from the search results.
- **Approval Workflow**: Tenant Admins receive access requests in the Admin Console (`/admin/requests`).
- **Audit Trail**: All requests and approvals are logged for compliance.

## RBAC Integration
- **MANAGE_DISCOVERY**: A specific permission action required to edit business metadata or manage discoverability.
- **Search Filtering**: Search results are automatically filtered based on the user's permissions, ensuring they only see what they are allowed to see (or what is explicitly discoverable).
