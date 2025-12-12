# Docker Deployment

Pangolin can be deployed as a Docker container.

## Prerequisites
- Docker
- Docker Compose

## Configuration
The application is configured via environment variables. See `docs/configuration.md` for details.

Key variables for Docker:
- `PANGOLIN_STORAGE_TYPE`: Set to `s3` for persistence.
- `AWS_*`: S3 credentials and endpoint.

## Running with Docker Compose

A `docker-compose.yml` is provided to run Pangolin alongside MinIO, Postgres, and MongoDB.

```bash
docker-compose up --build
```

This will:
1. Start MinIO on ports 9000/9001.
2. Build the Pangolin image.
3. Start Pangolin on port 8080.

## Building the Image Manually

```bash
docker build -t pangolin .
```

## Running the Container

```bash
docker run -p 8080:8080 -e PANGOLIN_STORAGE_TYPE=memory pangolin
```
