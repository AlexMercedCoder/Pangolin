# Production: S3 + Postgres

This setup uses AWS S3 for object storage and PostgreSQL for metadata storage.

## Configuration

Set the following environment variables in a `.env` file or export them before running:

```bash
# AWS Credentials
export AWS_ACCESS_KEY_ID=your_access_key
export AWS_SECRET_ACCESS_KEY=your_secret_key
export AWS_REGION=us-east-1

# Security
export PANGOLIN_ROOT_PASSWORD=strong_password
export PANGOLIN_JWT_SECRET=random_secret_string

# Database (Optional, defaults provided)
export POSTGRES_PASSWORD=db_password
```

## Running

```bash
docker compose up -d
```
