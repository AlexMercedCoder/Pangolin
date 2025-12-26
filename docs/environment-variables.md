# Environment Variables Reference

This document provides a comprehensive reference for all environment variables used by Pangolin API.

## Table of Contents

- [Storage Backend Configuration](#storage-backend-configuration)
- [Authentication & Security](#authentication--security)
- [Object Storage (S3/MinIO)](#object-storage-s3minio)
- [MongoDB Configuration](#mongodb-configuration)
- [PostgreSQL Configuration](#postgresql-configuration)
- [SQLite Configuration](#sqlite-configuration)
- [Server Configuration](#server-configuration)
- [Logging](#logging)
- [Quick Start Examples](#quick-start-examples)

---

## Storage Backend Configuration

### `DATABASE_URL`

**Required:** No (defaults to in-memory storage)  
**Type:** String (Connection URL)  
**Description:** Primary configuration for selecting and connecting to the storage backend.

The format of this URL determines which backend is used:

| Backend | URL Format | Example |
|---------|------------|---------|
| PostgreSQL | `postgresql://` or `postgres://` | `postgresql://user:pass@localhost:5432/pangolin` |
| MongoDB | `mongodb://` or `mongodb+srv://` | `mongodb://user:pass@localhost:27017` |
| SQLite | `sqlite://` or ends with `.db` | `sqlite://pangolin.db` or `/path/to/pangolin.db` |
| Memory | Not set or invalid format | *(no value)* |

**Examples:**

```bash
# PostgreSQL
export DATABASE_URL="postgresql://pangolin:secret@localhost:5432/pangolin_db"

# MongoDB
export DATABASE_URL="mongodb://admin:password@localhost:27017"

# MongoDB Atlas
export DATABASE_URL="mongodb+srv://user:pass@cluster.mongodb.net"

# SQLite
export DATABASE_URL="sqlite://./pangolin.db"

# Memory (default - no persistence)
# Don't set DATABASE_URL or set to empty string
```

### `MONGO_DB_NAME`

**Required:** Only when using MongoDB  
**Type:** String  
**Default:** `pangolin`  
**Description:** Name of the MongoDB database to use.

```bash
export DATABASE_URL="mongodb://localhost:27017"
export MONGO_DB_NAME="pangolin_production"
```

### `PANGOLIN_STORAGE_TYPE`

**Status:** ⚠️ **DEPRECATED** - Not currently used  
**Description:** This variable is not read by the API. Use `DATABASE_URL` instead.

---

## Authentication & Security

### `PANGOLIN_NO_AUTH`

**Required:** No  
**Type:** Boolean (`true` or `false`)  
**Default:** `false`  
**Description:** **⚠️ EVALUATION/DEVELOPMENT ONLY** - Disables authentication entirely.

When enabled:
- Creates a default tenant with ID `00000000-0000-0000-0000-000000000000`
- Auto-provisions a `tenant_admin` user with password `password123`
- All API requests bypass authentication
- Tenant creation endpoint is disabled

**Example:**

```bash
export PANGOLIN_NO_AUTH=true
```

**⚠️ WARNING:** Never use `PANGOLIN_NO_AUTH=true` in production environments!

### `JWT_SECRET`

**Required:** Yes (for production with authentication)  
**Type:** String  
**Default:** Auto-generated (insecure for production)  
**Description:** Secret key used for signing JWT tokens.

```bash
# Generate a secure secret
export JWT_SECRET=$(openssl rand -base64 32)
```

---

## Object Storage (S3/MinIO)

These variables configure access to S3-compatible object storage for Iceberg table metadata and data files.

### `AWS_ACCESS_KEY_ID`

**Required:** Yes (for S3/MinIO storage)  
**Type:** String  
**Description:** AWS access key ID or MinIO access key.

```bash
export AWS_ACCESS_KEY_ID="AKIAIOSFODNN7EXAMPLE"
```

### `AWS_SECRET_ACCESS_KEY`

**Required:** Yes (for S3/MinIO storage)  
**Type:** String  
**Description:** AWS secret access key or MinIO secret key.

```bash
export AWS_SECRET_ACCESS_KEY="wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
```

### `AWS_REGION`

**Required:** No  
**Type:** String  
**Default:** `us-east-1`  
**Description:** AWS region for S3 buckets.

```bash
export AWS_REGION="us-west-2"
```

### `S3_ENDPOINT`

**Required:** Only for MinIO or S3-compatible storage  
**Type:** String (URL)  
**Description:** Custom S3 endpoint URL (for MinIO, LocalStack, etc.).

```bash
# MinIO
export S3_ENDPOINT="http://localhost:9000"

# LocalStack
export S3_ENDPOINT="http://localhost:4566"
```

### `AWS_ENDPOINT_URL`

**Required:** No  
**Type:** String (URL)  
**Description:** Alternative to `S3_ENDPOINT`. Both work the same way.

```bash
export AWS_ENDPOINT_URL="http://minio:9000"
```

---

## MongoDB Configuration

### `DATABASE_URL` (MongoDB)

See [Storage Backend Configuration](#storage-backend-configuration) above.

### `MONGO_DB_NAME`

See [Storage Backend Configuration](#storage-backend-configuration) above.

### `MONGODB_URI`

**Status:** ⚠️ **NOT USED**  
**Description:** This variable is not read. Use `DATABASE_URL` instead.

### `MONGODB_DATABASE`

**Status:** ⚠️ **NOT USED**  
**Description:** This variable is not read. Use `MONGO_DB_NAME` instead.

---

## PostgreSQL Configuration

### `DATABASE_URL` (PostgreSQL)

See [Storage Backend Configuration](#storage-backend-configuration) above.

**Connection String Format:**

```
postgresql://[user[:password]@][host][:port][/dbname][?param1=value1&...]
```

**Example with all parameters:**

```bash
export DATABASE_URL="postgresql://pangolin_user:secure_password@db.example.com:5432/pangolin_db?sslmode=require&connect_timeout=10"
```

---

## SQLite Configuration

### `DATABASE_URL` (SQLite)

See [Storage Backend Configuration](#storage-backend-configuration) above.

**Path Formats:**

```bash
# Relative path
export DATABASE_URL="sqlite://pangolin.db"

# Absolute path
export DATABASE_URL="sqlite:///var/lib/pangolin/data.db"

# Alternative format (file path only)
export DATABASE_URL="/var/lib/pangolin/pangolin.db"
```

---

## Server Configuration

### `PANGOLIN_HOST`

**Required:** No  
**Type:** String (IP address)  
**Default:** `0.0.0.0`  
**Description:** IP address to bind the server to.

```bash
# Listen on all interfaces
export PANGOLIN_HOST="0.0.0.0"

# Listen only on localhost
export PANGOLIN_HOST="127.0.0.1"
```

### `PANGOLIN_PORT` or `PORT`

**Required:** No  
**Type:** Integer  
**Default:** `8080`  
**Description:** Port number for the API server.

```bash
export PANGOLIN_PORT=3000
# or
export PORT=3000
```

---

## Logging

### `RUST_LOG`

**Required:** No  
**Type:** String (log level)  
**Default:** `info`  
**Description:** Controls logging verbosity.

**Valid levels:** `error`, `warn`, `info`, `debug`, `trace`

```bash
# Production
export RUST_LOG=info

# Development
export RUST_LOG=debug

# Troubleshooting
export RUST_LOG=trace

# Module-specific logging
export RUST_LOG=pangolin_api=debug,pangolin_store=info
```

---

## Quick Start Examples

### Development with Memory Storage

```bash
export PANGOLIN_NO_AUTH=true
export RUST_LOG=debug
export AWS_ACCESS_KEY_ID=minioadmin
export AWS_SECRET_ACCESS_KEY=minioadmin
export S3_ENDPOINT=http://localhost:9000
export AWS_REGION=us-east-1

./pangolin_api
```

### Production with MongoDB

```bash
export DATABASE_URL="mongodb://pangolin_user:secure_pass@mongo.example.com:27017"
export MONGO_DB_NAME="pangolin_production"
export JWT_SECRET="your-super-secret-jwt-key-here"
export AWS_ACCESS_KEY_ID="AKIAIOSFODNN7EXAMPLE"
export AWS_SECRET_ACCESS_KEY="wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
export AWS_REGION="us-east-1"
export RUST_LOG=info
export PANGOLIN_PORT=8080

./pangolin_api
```

### Production with PostgreSQL

```bash
export DATABASE_URL="postgresql://pangolin:password@postgres.example.com:5432/pangolin?sslmode=require"
export JWT_SECRET="your-super-secret-jwt-key-here"
export AWS_ACCESS_KEY_ID="AKIAIOSFODNN7EXAMPLE"
export AWS_SECRET_ACCESS_KEY="wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
export AWS_REGION="us-west-2"
export RUST_LOG=info

./pangolin_api
```

### Docker Compose with MinIO

```yaml
version: '3.8'

services:
  pangolin-api:
    image: pangolin-api:latest
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=mongodb://mongo:27017
      - MONGO_DB_NAME=pangolin
      - RUST_LOG=info
      - AWS_ACCESS_KEY_ID=minioadmin
      - AWS_SECRET_ACCESS_KEY=minioadmin
      - S3_ENDPOINT=http://minio:9000
      - AWS_REGION=us-east-1
    depends_on:
      - mongo
      - minio

  mongo:
    image: mongo:7.0
    ports:
      - "27017:27017"

  minio:
    image: minio/minio
    ports:
      - "9000:9000"
      - "9001:9001"
    environment:
      - MINIO_ROOT_USER=minioadmin
      - MINIO_ROOT_PASSWORD=minioadmin
    command: server /data --console-address ":9001"
```

---

## Common Pitfalls

### ❌ Using the wrong variable names

```bash
# WRONG - These are not used
export PANGOLIN_STORE_TYPE=mongo
export MONGODB_URI=mongodb://localhost:27017
export MONGODB_DATABASE=pangolin

# CORRECT
export DATABASE_URL=mongodb://localhost:27017
export MONGO_DB_NAME=pangolin
```

### ❌ Forgetting MongoDB database name

```bash
# WRONG - Will use default database name "pangolin"
export DATABASE_URL=mongodb://localhost:27017

# CORRECT - Explicitly set database name
export DATABASE_URL=mongodb://localhost:27017
export MONGO_DB_NAME=my_custom_db
```

### ❌ Missing S3 endpoint for MinIO

```bash
# WRONG - Will try to connect to AWS S3
export AWS_ACCESS_KEY_ID=minioadmin
export AWS_SECRET_ACCESS_KEY=minioadmin

# CORRECT - Specify MinIO endpoint
export AWS_ACCESS_KEY_ID=minioadmin
export AWS_SECRET_ACCESS_KEY=minioadmin
export S3_ENDPOINT=http://localhost:9000
```

### ❌ Using NO_AUTH in production

```bash
# WRONG - Security risk!
export PANGOLIN_NO_AUTH=true  # in production

# CORRECT - Use proper authentication
export JWT_SECRET=$(openssl rand -base64 32)
# Don't set PANGOLIN_NO_AUTH
```

---

## Environment Variable Priority

When multiple variables could configure the same thing:

1. **Storage Backend:** `DATABASE_URL` (only this is used)
2. **MongoDB Database:** `MONGO_DB_NAME` (only this is used)
3. **S3 Endpoint:** `S3_ENDPOINT` or `AWS_ENDPOINT_URL` (both work)
4. **Server Port:** `PANGOLIN_PORT` or `PORT` (PANGOLIN_PORT takes precedence)

---

## Troubleshooting

### API uses Memory storage instead of MongoDB

**Problem:** You set `MONGODB_URI` but API still uses memory storage.

**Solution:** Use `DATABASE_URL` instead:

```bash
export DATABASE_URL="mongodb://localhost:27017"
export MONGO_DB_NAME="pangolin"
```

### Cannot connect to MinIO

**Problem:** S3 operations fail with connection errors.

**Solution:** Make sure to set the endpoint:

```bash
export S3_ENDPOINT="http://localhost:9000"
# or
export AWS_ENDPOINT_URL="http://localhost:9000"
```

### Authentication errors in development

**Problem:** Getting 401 Unauthorized errors.

**Solution:** For development/testing only:

```bash
export PANGOLIN_NO_AUTH=true
```

**Remember:** Never use this in production!

---

## See Also

- [Deployment Guide](./deployment.md)
- [Docker Setup](./docker-setup.md)
- [Configuration Best Practices](./best-practices/configuration.md)
