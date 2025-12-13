# S3 Storage Configuration

Pangolin supports storing catalog data and Iceberg metadata in S3-compatible object storage (AWS S3, MinIO, GCS, Azure Blob Storage via S3 compatibility).

## Configuration

To enable S3 storage, set the `PANGOLIN_STORAGE_TYPE` environment variable to `s3`.

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PANGOLIN_STORAGE_TYPE` | Storage backend (`memory`, `s3`, `postgres`, `mongo`) | `memory` |
| `AWS_REGION` | AWS Region | `us-east-1` |
| `PANGOLIN_S3_BUCKET` | S3 Bucket Name | `pangolin` |
| `PANGOLIN_S3_PREFIX` | S3 Prefix for data | `data` |
| `AWS_ACCESS_KEY_ID` | AWS Access Key | - |
| `AWS_SECRET_ACCESS_KEY` | AWS Secret Key | - |
| `AWS_ENDPOINT_URL` | Custom S3 Endpoint (for MinIO) | - |
| `AWS_ALLOW_HTTP` | Allow HTTP (for local MinIO) | `false` |

## Directory Structure

Pangolin organizes data in the S3 bucket using the following structure:

```
s3://<bucket>/
  tenants/
    <tenant_id>/
      tenant.json
      catalogs/
        <catalog_name>/
          catalog.json
          namespaces/
            <namespace_path>/
              namespace.json
              assets/
                <asset_name>.json
          branches/
            <branch_name>/
              branch.json
```

## Iceberg Metadata

Iceberg metadata files (`metadata.json`, `manifest_list`, `manifest_files`) are stored in the location specified by the table, typically:

```
s3://<warehouse>/<catalog>/<namespace>/<table>/metadata/
```

Pangolin manages the atomic updates of the `metadata.json` file during table commits.
