# Deployment Guide

## Local Development
To run Pangolin locally, you can use `cargo`:

```bash
cd pangolin
cargo run --bin pangolin_api
```

This starts the server on `0.0.0.0:8080`.

## Docker Deployment
Pangolin includes a `Dockerfile` for containerized deployment.

### Build Image
```bash
docker build -t pangolin:latest .
```

### Run with Docker Compose
A `docker-compose.yml` is provided to run Pangolin with MinIO, Postgres, and MongoDB.

```bash
docker-compose up -d
```

This will expose:
- Pangolin API: `http://localhost:8080`
- MinIO Console: `http://localhost:9001`
