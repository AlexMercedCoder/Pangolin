# Pangolin Helm Chart

This directory contains the Helm chart for deploying Pangolin to Kubernetes.

## Chart Structure
- `templates/`: Kubernetes manifests (Deployment, Service, etc.)
- `values.yaml`: Default configuration (runs with defaults, but expects DB details).
- `values-*.yaml`: Example value overrides for specific production scenarios.

## Quick Start

```bash
# Install with default values (requires DB secret setup or editing values.yaml)
helm install pangolin ./pangolin

# Install with AWS + Postgres values
helm install pangolin ./pangolin -f ./pangolin/values-aws-postgres.yaml
# NOTE: You must edit values-aws-postgres.yaml to put REAL secrets/endpoints first!
```

## Configuration
See `values.yaml` for all available options including:
- Image tag/repository
- Resource limits
- Environment variables (`PANGOLIN_STORAGE_TYPE`, `AWS_REGION`, etc.)
- Secrets
