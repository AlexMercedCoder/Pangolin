# Production: S3 + Mongo

This setup uses AWS S3 for object storage and MongoDB for metadata storage.

## Configuration

Set the following environment variables:

```bash
# AWS Credentials
export AWS_ACCESS_KEY_ID=your_access_key
export AWS_SECRET_ACCESS_KEY=your_secret_key
export AWS_REGION=us-east-1

# Security
export PANGOLIN_ROOT_PASSWORD=strong_password
export PANGOLIN_JWT_SECRET=random_secret_string
```

## Running

```bash
docker compose up -d
```
