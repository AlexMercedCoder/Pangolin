# Pangolin on Azure (Terraform Template)

> [!WARNING]
> This is a **template**. It opens the PostgreSQL firewall to all IPs (`0.0.0.0/0`) for demonstration purposes. **Configure proper VNET integration for production.**

## Resources Created
- **Resource Group**: Contains all resources.
- **Storage Account**: Blob storage for Iceberg.
- **Azure Database for PostgreSQL**: Metadata store.

## Usage
1. `terraform init`
2. `terraform apply`
3. Retrieve outputs to configure Pangolin.
