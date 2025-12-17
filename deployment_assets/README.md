# Deployment Assets

This directory contains resources for deploying Pangolin to various environments.

## Structure

### `demo/`
Contains quick-start configurations for evaluating Pangolin.
- **`evaluate_single_tenant/`**: Simple SQLite-based setup.
- **`evaluate_multi_tenant/`**: Full setup with MinIO and SQLite/Postgres options.

### `helm/`
Helm charts for Kubernetes deployments.

### `production/`
Production-grade configurations (e.g., proper logging, metrics, security settings).

### `terraform/`
Terraform modules for provisioning cloud infrastructure (AWS, Azure, GCP).

## Usage
Refer to the [Getting Started Guide](../docs/getting-started/getting_started.md) for detailed deployment instructions.
