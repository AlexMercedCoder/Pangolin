# Pangolin on AWS (Terraform Template)

> [!WARNING]
> This is a **template** for demonstration. It creates resources in the default VPC and expores the RDS instance publicly for ease of access from local machines or K8s clusters without peering. **Do not use this "as is" for sensitive production data.**

## Resources Created
- **S3 Bucket**: Stores Iceberg data/metadata.
- **RDS Postgres**: Stores Pangolin catalog metadata.

## Usage
1. `terraform init`
2. `terraform apply`
3. Use the outputs to configure your `docker-compose.yml` or Helm Chart.
