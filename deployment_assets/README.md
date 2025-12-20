# Deployment Assets

This directory contains resources for deploying Pangolin to various environments.

## Structure

### `demo/`
Contains quick-start configurations for evaluating Pangolin.
- **`evaluate_single_tenant/`**: No-auth mode with memory storage for rapid testing.
- **`evaluate_multi_tenant/`**: Full authentication with SQLite for persistent evaluation.

Both demo scenarios include:
- **Jupyter Notebook** (port 8888) with PyIceberg pre-installed for interactive testing
- **MinIO** for S3-compatible local storage
- **Pangolin UI** for visual management
- **Pangolin API** for REST catalog operations

### `helm/`
Helm charts for Kubernetes deployments.
- Production-ready templates with configurable values
- Support for various cloud providers (AWS, Azure, GCP)
- Includes example value files for different scenarios

### `production/`
Production-grade Docker Compose configurations for different cloud + database combinations:
- `s3_postgres/`, `s3_mongo/` - AWS S3 with PostgreSQL or MongoDB
- `azure_postgres/`, `azure_mongo/` - Azure Blob Storage with PostgreSQL or MongoDB
- `gcp_postgres/`, `gcp_mongo/` - Google Cloud Storage with PostgreSQL or MongoDB

### `terraform/`
Terraform modules for provisioning cloud infrastructure (AWS, Azure, GCP).
- Provisions object storage (S3, Blob, GCS)
- Provisions managed databases (RDS, Azure Database, Cloud SQL)
- Outputs connection details for Pangolin configuration

## Quick Start

For evaluation, use the demo scenarios:
```bash
cd demo/evaluate_single_tenant
docker compose up -d
```

Access:
- **Pangolin UI**: http://localhost:3000
- **Jupyter Notebook**: http://localhost:8888
- **MinIO Console**: http://localhost:9001

## Usage
Refer to the [Getting Started Guide](../docs/getting-started/getting_started.md) for detailed deployment instructions.
