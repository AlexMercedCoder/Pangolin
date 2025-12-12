# Security & Credential Vending

Pangolin provides mechanisms to securely vend AWS credentials and presigned URLs to clients, enabling direct data access while maintaining security.

## Overview

Instead of sharing long-term AWS credentials with clients (e.g., Spark jobs, Dremio, Trino), Pangolin acts as a trusted intermediary. It authenticates the client and then issues temporary, scoped credentials or presigned URLs for specific S3 resources.

## Features

### 1. Table Credential Vending
Get temporary AWS credentials (Access Key, Secret Key, Session Token) scoped to a specific table's location.

**Endpoint:** `POST /v1/{prefix}/namespaces/{namespace}/tables/{table}/credentials`

**Response:**
```json
{
  "access_key_id": "ASIA...",
  "secret_access_key": "...",
  "session_token": "...",
  "expiration": "2023-10-27T10:00:00Z"
}
```

### 2. Presigned URLs
Get a presigned URL to download a specific file (e.g., a metadata file or data file) without needing AWS credentials.

**Endpoint:** `GET /v1/{prefix}/namespaces/{namespace}/tables/{table}/presign?location=s3://bucket/key`

**Response:**
```json
{
  "url": "https://bucket.s3.amazonaws.com/key?X-Amz-Algorithm=..."
}
```

## Configuration

To enable vending, Pangolin must be configured with AWS credentials that have `sts:AssumeRole` permissions (for role assumption) or direct S3 access (for static credential vending).

See `docs/env_vars.md` for AWS configuration details.
