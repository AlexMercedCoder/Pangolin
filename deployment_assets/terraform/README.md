# Pangolin Terraform Templates

This directory contains **terraform templates** to provision the cloud infrastructure required by Pangolin (Object Storage + Database).

> [!IMPORTANT]
> These templates are intended as **starting points**. They purposefully omit complex networking (VPC peering, private endpoints) to remain portable and easy to demonstrate. **Review all security groups and access controls before deploying to production.**

## Supported Clouds
- [Amazon Web Services (AWS)](./aws/)
  - S3 + RDS Postgres
- [Microsoft Azure](./azure/)
  - Blob Storage + Azure Database for PostgreSQL
- [Google Cloud Platform (GCP)](./gcp/)
  - GCS + Cloud SQL Postgres

## Workflow
1. Direct into the desired cloud provider folder.
2. Initialize and Apply Terraform.
   ```bash
   cd aws
   terraform init
   terraform apply
   ```
3. Use the **output values** (Database Host, Bucket Name, Secret Keys) to configure:
   - Your local `docker-compose.yml`
   - OR your Helm Chart `values.yaml`
