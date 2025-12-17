# Production: GCP + Mongo

This setup uses Google Cloud Storage (GCS) for object storage and MongoDB for metadata storage.

## Configuration

Set the following environment variables:

```bash
# GCP Credentials
# Path to your Service Account JSON key
export GCP_CREDS_PATH=/path/to/your/service-account.json

# Security
export PANGOLIN_ROOT_PASSWORD=strong_password
export PANGOLIN_JWT_SECRET=random_secret_string
```

## Running

```bash
docker compose up -d
```
