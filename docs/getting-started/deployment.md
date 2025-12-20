# Deployment Guide

This guide covers how to deploy Pangolin in various environments, from local development to production-ready Docker containers.

## 🛠️ Local Development

To run Pangolin locally for development or testing, ensure you have the [Rust toolchain](./dependencies.md) installed.

```bash
# Clone the repository
git clone https://github.com/your-org/pangolin.git
cd pangolin

# Run the API server
cargo run --bin pangolin_api
```

The server listens on `0.0.0.0:8080` by default. You can change this using the `PORT` environment variable.

---

## 🐳 Docker Deployment

Pangolin is designed to be cloud-native and easily containerized.

### 1. Using Docker Compose (Recommended for Evaluation)

Pangolin provides a `docker-compose.yml` that sets up the API server along with common dependencies like MinIO (S3), PostgreSQL, and MongoDB.

```bash
# Start all services
docker-compose up -d

# Check logs
docker-compose logs -f pangolin-api
```

**Services Exposed:**
- **Pangolin API**: `http://localhost:8080`
- **MinIO Console**: `http://localhost:9001` (Credentials: `minioadmin`/`minioadmin`)
- **PostgreSQL**: `localhost:5432`

### 2. Building the Image Manually

```bash
# From the root of the repository
docker build -t pangolin:latest .
```

### 3. Running the Container

```bash
docker run -p 8080:8080 \
  -e PANGOLIN_STORAGE_TYPE=memory \
  -e PANGOLIN_NO_AUTH=true \
  pangolin:latest
```

---

## 🚀 Production Best Practices

When moving to production, consider the following:

### 1. Persistent Metadata Storage
Do not use `memory` storage in production. Configure a robust backend like **PostgreSQL** or **MongoDB**.

```bash
export DATABASE_URL="postgresql://user:pass@db-host:5432/pangolin"
```

### 2. Enable Authentication
Ensure `PANGOLIN_NO_AUTH` is NOT set to `true`. Set a strong `PANGOLIN_JWT_SECRET`.

### 3. Use Cloud IAM Roles
For storage access, prefer `AwsSts` or `AzureSas` vending strategies over static keys. See [Credential Vending](../features/iam_roles.md).

### 4. Monitoring & Logging
Set `RUST_LOG=info` or `debug` and integrate the container logs with your logging provider.

## Related Documentation
- [Environment Variables](./env_vars.md)
- [Client Configuration](./client_configuration.md)
- [Multi-Tenancy](../features/multi_tenancy.md)
