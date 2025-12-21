# Catalog Management

Pangolin supports managing multiple Iceberg catalogs within a single tenant. This document outlines the behavior and features of catalog management.

## Overview

Catalogs in Pangolin serve as the top-level container for namespaces and tables. Each catalog is associated with a Warehouse which defines the underlying object storage configuration.

## Catalog Types

*   **Local**: Managed directly by Pangolin. Metadata is stored in Pangolin's backend.
*   **Federated**: Proxies requests to an external catalog (e.g., Snowflake, Glue, another REST catalog).

## Naming and Collisions

Catalog names must be unique within a Tenant.

### Behavior by Storage Backend

*   **Postgres / SQLite (Production)**:
    *   **Constraint**: The database enforces a `UNIQUE(tenant_id, name)` constraint.
    *   **Collision Behavior**: Attempting to create a catalog with a name that already exists in the same tenant will result in an error. The API will currently return a `500 Internal Server Error` (to be improved to `409 Conflict`).
    *   **Cross-Tenant**: Different tenants can have catalogs with the same name without conflict.

*   **Memory Store (Development/Demo)**:
    *   **Constraint**: Based on a generic Key-Value map.
    *   **Collision Behavior**: **Silent Overwrite**. Creating a catalog with an existing name will replace the old catalog configuration. No error is returned.
    *   **Note**: This behavior is specific to the in-memory development store and does not reflect production safety.

## Creation

Catalogs can be created via:
1.  **UI**: *Catalogs -> Create Catalog*
2.  **API**: `POST /api/v1/catalogs`
