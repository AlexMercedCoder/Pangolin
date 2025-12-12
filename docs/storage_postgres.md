# Postgres Storage Backend

Pangolin supports PostgreSQL as a shared metadata backend, allowing for robust, concurrent access to metadata across multiple Pangolin instances.

## Configuration

To use Postgres, set the following environment variables:

- `PANGOLIN_STORAGE_TYPE`: Set to `postgres`.
- `DATABASE_URL`: The connection string for your Postgres database (e.g., `postgres://user:password@localhost:5432/pangolin`).

## Setup

1.  **Ensure Postgres is running**: You can use the provided `docker-compose.yml` to start a local Postgres instance:
    ```bash
    docker-compose up -d postgres
    ```
2.  **Schema Migration**: Pangolin automatically handles schema creation on startup if the tables do not exist.

## Features Supported

The Postgres backend implements the full `CatalogStore` trait, supporting:
-   **Tenants, Catalogs, Namespaces**: Full CRUD.
-   **Assets**: Iceberg tables and other asset types.
-   **Branching & Tagging**: Full support for Git-like semantics.
-   **Audit Logs**: Persistent storage of audit events.
-   **Concurrency**: Uses Postgres transactions and locking for safe concurrent updates.

## Data Model

The data is stored in the following tables:
-   `tenants`
-   `warehouses`
-   `catalogs`
-   `namespaces`
-   `assets`
-   `branches`
-   `tags`
-   `commits`
-   `audit_logs`

Each table uses UUIDs for primary keys and foreign keys to maintain referential integrity.
