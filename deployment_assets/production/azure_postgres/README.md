# Production: Azure + Postgres

This setup uses Azure Data Lake Storage (ADLS/Blob) for object storage and PostgreSQL for metadata storage.

## Configuration

Set the following environment variables:

```bash
# Azure Credentials
export AZURE_STORAGE_ACCOUNT_NAME=your_storage_account
export AZURE_STORAGE_ACCOUNT_KEY=your_access_key

# Security
export PANGOLIN_ROOT_PASSWORD=strong_password
export PANGOLIN_JWT_SECRET=random_secret_string
```

## Running

```bash
docker compose up -d
```
