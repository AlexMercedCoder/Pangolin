# Pangolin on GCP (Terraform Template)

> [!WARNING]
> This template creates a Cloud SQL instance with a **Public IP** open to `0.0.0.0/0`. This is for testing only. Use Cloud SQL Auth Proxy or Private IP in production.

## Resources Created
- **GCS Bucket**: Storage for Iceberg.
- **Cloud SQL**: Postgres 15 instance.

## Usage
1. `terraform init`
2. `terraform apply -var="project_id=YOUR_PROJECT_ID"`
3. Use outputs to configure Pangolin.
   - Note: Cloud SQL requires special connection handling mostly (Socket or Proxy), or just IP whitelisting. This template sets up IP whitelisting.
