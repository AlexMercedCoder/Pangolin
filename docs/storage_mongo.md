# MongoDB Storage Backend

Pangolin supports MongoDB as a shared metadata backend, providing a flexible, document-oriented storage option for metadata.

## Configuration

To use MongoDB, set the following environment variables:

- `PANGOLIN_STORAGE_TYPE`: Set to `mongo`.
- `DATABASE_URL`: The connection string for your MongoDB instance (e.g., `mongodb://localhost:27017`).

## Setup

1.  **Ensure MongoDB is running**: You can use the provided `docker-compose.yml` to start a local MongoDB instance:
    ```bash
    docker-compose up -d mongo
    ```
2.  **Database Creation**: Pangolin will automatically create the `pangolin` database and necessary collections upon the first write operation.

## Features Supported

The MongoDB backend implements the full `CatalogStore` trait, supporting:
-   **Tenants, Catalogs, Namespaces**: Full CRUD.
-   **Assets**: Iceberg tables and other asset types.
-   **Branching & Tagging**: Full support for Git-like semantics.
-   **Audit Logs**: Persistent storage of audit events.

## Data Model

The data is stored in the following collections within the `pangolin` database:
-   `tenants`
-   `warehouses`
-   `catalogs`
-   `namespaces`
-   `assets`
-   `branches`
-   `tags`
-   `commits`
-   `audit_logs`

### Schema Details
-   **UUIDs**: Stored as BSON Binary (Subtype 4 or Generic) for efficiency.
-   **Timestamps**: Stored as BSON DateTime.
-   **Relationships**: References are stored as embedded UUIDs (e.g., `tenant_id` in `catalogs`).
