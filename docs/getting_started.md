# Getting Started with Pangolin

This guide will walk you through setting up Pangolin and performing an end-to-end workflow: creating a tenant, setting up a warehouse, managing data, and using advanced features like branching and merging.

## Prerequisites

- **Rust**: Version 1.92 or later ([Install Rust](https://www.rust-lang.org/tools/install))
- **Docker** (Optional): For running dependencies like MinIO (S3 compatible storage) if you want to test S3 integration.
- **curl**: For making API requests.

## Installation & Setup

1.  **Clone the Repository**
    ```bash
    git clone https://github.com/your-repo/pangolin.git
    cd pangolin
    ```

2.  **Run the Server**
    ```bash
    # Run from the workspace root
    cargo run --bin pangolin_api
    ```
    The server will start at `http://localhost:8080`.

## End-to-End Walkthrough

We will simulate a real-world scenario: A "Data Team" setting up their environment, creating a table, experimenting on a branch, and merging changes back.

### 1. Create a Tenant
Pangolin is multi-tenant by default. First, create a tenant for your organization.

```bash
curl -X POST http://localhost:8080/api/v1/tenants \
  -u admin:password \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Acme Corp",
    "id": "00000000-0000-0000-0000-000000000001"
  }'
```
*Note: We are using a fixed UUID for simplicity. The `-u admin:password` corresponds to the default Root User credentials in `example.env`.*

### 2. Create a Warehouse
A warehouse maps to a physical storage location (like an S3 bucket).

```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "X-Pangolin-Tenant: 00000000-0000-0000-0000-000000000001" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "main_warehouse",
    "storage_config": {
      "type": "memory",
      "bucket": "demo-bucket",
      "prefix": "data"
    }
  }'
```
*Note: We use `type: "memory"` for this quick start. For S3, see [Configuration](configuration.md).*

### 3. Create a Namespace
Namespaces organize your tables (like schemas in a database).

```bash
curl -X POST http://localhost:8080/v1/main_warehouse/namespaces \
  -H "X-Pangolin-Tenant: 00000000-0000-0000-0000-000000000001" \
  -H "Content-Type: application/json" \
  -d '{"namespace": ["data_team"]}'
```

### 4. Create a Table on `main`
Create an Iceberg table in the `data_team` namespace. By default, operations target the `main` branch.

```bash
curl -X POST http://localhost:8080/v1/main_warehouse/namespaces/data_team/tables \
  -H "X-Pangolin-Tenant: 00000000-0000-0000-0000-000000000001" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "users",
    "location": "s3://demo-bucket/data/data_team/users",
    "schema": {
      "type": "struct",
      "fields": [
        {"id": 1, "name": "id", "type": "int", "required": true},
        {"id": 2, "name": "name", "type": "string", "required": false}
      ]
    }
  }'
```

### 5. Create a Feature Branch (`dev`)
Create a new branch named `dev` to experiment safely. We will use **Partial Branching** to only track the `users` table on this branch.

```bash
curl -X POST http://localhost:8080/api/v1/branches \
  -H "X-Pangolin-Tenant: 00000000-0000-0000-0000-000000000001" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "dev",
    "from_branch": "main",
    "branch_type": "experimental",
    "assets": ["data_team.users"]
  }'
```

### 6. Update Table on `dev`
Modify the table schema on the `dev` branch. Note the `@dev` suffix in the URL.

```bash
curl -X POST http://localhost:8080/v1/main_warehouse/namespaces/data_team/tables/users?branch=dev \
  -H "X-Pangolin-Tenant: 00000000-0000-0000-0000-000000000001" \
  -H "Content-Type: application/json" \
  -d '{
    "updates": [
      {"action": "add-column", "name": "email", "type": "string"}
    ]
  }'
```

### 7. Verify Isolation
Check that the `main` branch is unaffected.

```bash
# Check main (should NOT have email column)
curl http://localhost:8080/v1/main_warehouse/namespaces/data_team/tables/users?branch=main \
  -H "X-Pangolin-Tenant: 00000000-0000-0000-0000-000000000001"

# Check dev (SHOULD have email column)
curl http://localhost:8080/v1/main_warehouse/namespaces/data_team/tables/users?branch=dev \
  -H "X-Pangolin-Tenant: 00000000-0000-0000-0000-000000000001"
```

### 8. Merge `dev` into `main`
Once satisfied, merge the changes back to `main`.

```bash
curl -X POST http://localhost:8080/api/v1/branches/merge \
  -H "X-Pangolin-Tenant: 00000000-0000-0000-0000-000000000001" \
  -H "Content-Type: application/json" \
  -d '{
    "source_branch": "dev",
    "target_branch": "main"
  }'
```

Now, `main` will contain the schema updates made in `dev`.

## Next Steps

- Explore [Branch Management](branch_management.md) for advanced strategies.
- Configure [S3 Storage](storage_s3.md) for production data.
- Learn about [Warehouse Management](warehouse_management.md).
