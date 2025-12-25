# Deployment Best Practices

## Overview

This guide covers best practices for deploying Pangolin in production environments, from initial setup to ongoing operations.

## Pre-Deployment Planning

### Infrastructure Assessment

**Compute Requirements**
- **API Server**: 2-4 CPU cores, 4-8GB RAM minimum for production
- **Metadata Store**: Size based on catalog scale (see Backend Storage section)
- **Network**: Low-latency connection to object storage (S3/Azure/GCS)

**Storage Requirements**
- **Metadata Backend**: PostgreSQL (recommended) or MongoDB for production
- **Object Storage**: S3, Azure Blob, or GCS for table data
- **Backup Storage**: Separate location for metadata backups

### Deployment Architecture

**Single-Region Deployment**
```
┌─────────────────────────────────────┐
│   Load Balancer (Optional)          │
└──────────────┬──────────────────────┘
               │
    ┌──────────┴──────────┐
    │                     │
┌───▼────┐          ┌────▼───┐
│ API    │          │ API    │
│ Server │          │ Server │
│   #1   │          │   #2   │
└───┬────┘          └────┬───┘
    │                    │
    └──────────┬─────────┘
               │
        ┌──────▼──────┐
        │  PostgreSQL │
        │   Cluster   │
        └─────────────┘
```

**Multi-Region Deployment**
- Deploy API servers in each region
- Use read replicas for metadata store
- Configure region-specific object storage
- Implement cross-region replication for critical catalogs

## Environment Configuration

### Production Environment Variables

```bash
# Server Configuration
PANGOLIN_STORE_TYPE=postgresql
PANGOLIN_AUTH_MODE=auth
PANGOLIN_JWT_SECRET=<strong-random-secret>  # Use secrets manager
PANGOLIN_JWT_EXPIRATION_HOURS=24

# Database Connection
DATABASE_URL=postgresql://user:pass@host:5432/pangolin
DATABASE_MAX_CONNECTIONS=20
DATABASE_IDLE_TIMEOUT=300

# Storage Configuration
AWS_REGION=us-east-1
AZURE_STORAGE_ACCOUNT=<account>
GCS_PROJECT_ID=<project>

# Logging
RUST_LOG=info,pangolin_api=debug
LOG_FORMAT=json  # For structured logging

# Performance
WORKER_THREADS=4
MAX_REQUEST_SIZE=10485760  # 10MB
```

### Secrets Management

**DO NOT** store secrets in environment files or version control.

**Recommended Approaches:**
1. **AWS Secrets Manager** - For AWS deployments
2. **Azure Key Vault** - For Azure deployments
3. **HashiCorp Vault** - Platform-agnostic
4. **Kubernetes Secrets** - For K8s deployments

```bash
# Example: Fetch from AWS Secrets Manager
export PANGOLIN_JWT_SECRET=$(aws secretsmanager get-secret-value \
  --secret-id pangolin/jwt-secret \
  --query SecretString --output text)
```

## Container Deployment

### Docker Best Practices

**Production Dockerfile**
```dockerfile
FROM rust:1.92 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin pangolin_api

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/pangolin_api /usr/local/bin/
EXPOSE 8080
CMD ["pangolin_api"]
```

**Docker Compose for Production**
```yaml
version: '3.8'

services:
  api:
    image: your-registry/pangolin-api:latest
    restart: unless-stopped
    ports:
      - "8080:8080"
    environment:
      - PANGOLIN_STORE_TYPE=postgresql
      - DATABASE_URL=postgresql://postgres:5432/pangolin
    depends_on:
      - postgres
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  postgres:
    image: postgres:16-alpine
    restart: unless-stopped
    volumes:
      - pgdata:/var/lib/postgresql/data
    environment:
      - POSTGRES_PASSWORD=${DB_PASSWORD}
      - POSTGRES_DB=pangolin

volumes:
  pgdata:
```

### Kubernetes Deployment

**Deployment Manifest**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: pangolin-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: pangolin-api
  template:
    metadata:
      labels:
        app: pangolin-api
    spec:
      containers:
      - name: api
        image: your-registry/pangolin-api:v0.2.0
        ports:
        - containerPort: 8080
        env:
        - name: PANGOLIN_JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: pangolin-secrets
              key: jwt-secret
        resources:
          requests:
            memory: "2Gi"
            cpu: "1000m"
          limits:
            memory: "4Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
```

## High Availability

### API Server HA

**Load Balancing**
- Use multiple API server instances (minimum 2)
- Configure health checks on `/health` endpoint
- Implement session affinity if needed (stateless preferred)

**Graceful Shutdown**
```bash
# Handle SIGTERM for graceful shutdown
# Drain existing connections before terminating
```

### Database HA

**PostgreSQL**
- Use managed service (RDS, Cloud SQL, Azure Database)
- Configure automatic failover
- Enable point-in-time recovery
- Set up read replicas for read-heavy workloads

**MongoDB**
- Deploy as replica set (minimum 3 nodes)
- Use managed service (Atlas, DocumentDB)
- Configure automatic failover

## Monitoring & Observability

### Health Checks

```bash
# API Health
curl http://localhost:8080/health

# Database Connection
curl http://localhost:8080/health/db
```

### Metrics to Monitor

**Application Metrics**
- Request rate and latency (p50, p95, p99)
- Error rate (4xx, 5xx responses)
- Active connections
- JWT token generation/validation rate

**Database Metrics**
- Connection pool utilization
- Query latency
- Deadlocks and slow queries
- Replication lag (if using replicas)

**System Metrics**
- CPU and memory utilization
- Disk I/O
- Network throughput

### Logging

**Structured Logging**
```json
{
  "timestamp": "2025-12-24T20:00:00Z",
  "level": "INFO",
  "message": "Table created",
  "tenant_id": "uuid",
  "user_id": "uuid",
  "catalog": "production",
  "table": "sales.orders"
}
```

**Log Aggregation**
- Use centralized logging (ELK, Splunk, CloudWatch)
- Retain logs for compliance (30-90 days minimum)
- Set up alerts for error patterns

## Backup & Disaster Recovery

### Metadata Backup

**PostgreSQL**
```bash
# Daily automated backups
pg_dump -h localhost -U postgres pangolin > backup_$(date +%Y%m%d).sql

# Point-in-time recovery enabled
# Retain backups for 30 days minimum
```

**MongoDB**
```bash
# Snapshot backups
mongodump --uri="mongodb://localhost:27017/pangolin" --out=/backup/$(date +%Y%m%d)
```

### Recovery Testing

- Test restore procedures quarterly
- Document recovery time objective (RTO) and recovery point objective (RPO)
- Maintain runbooks for common failure scenarios

## Security Hardening

### Network Security

- Deploy API behind firewall/security groups
- Use TLS/HTTPS for all connections
- Restrict database access to API servers only
- Implement rate limiting

### Authentication

- Use strong JWT secrets (256-bit minimum)
- Rotate secrets regularly (quarterly)
- Implement token expiration (24 hours recommended)
- Enable multi-factor authentication for admin users

## Performance Optimization

### Database Optimization

**Indexing**
```sql
-- Ensure proper indexes exist
CREATE INDEX idx_assets_catalog ON assets(catalog_id);
CREATE INDEX idx_assets_namespace ON assets(namespace);
CREATE INDEX idx_audit_timestamp ON audit_logs(timestamp);
```

**Connection Pooling**
- Configure appropriate pool size (10-20 connections per API instance)
- Set idle timeout (5 minutes)
- Monitor pool exhaustion

### Caching

- Implement caching for frequently accessed metadata
- Use Redis for distributed caching
- Set appropriate TTLs based on data volatility

## Upgrade Strategy

### Rolling Updates

1. **Test in Staging** - Always test upgrades in non-production first
2. **Backup** - Take full metadata backup before upgrade
3. **Rolling Deployment** - Update one instance at a time
4. **Verify** - Check health and metrics after each instance
5. **Rollback Plan** - Keep previous version ready for quick rollback

### Database Migrations

```bash
# Run migrations before deploying new API version
./pangolin-migrate up

# Verify migration success
./pangolin-migrate status
```

## Compliance & Audit

### Audit Logging

- Enable comprehensive audit logging
- Store audit logs separately from application logs
- Retain for compliance period (typically 1-7 years)
- Implement log integrity verification

### Access Control

- Follow principle of least privilege
- Regular access reviews (quarterly)
- Automated user deprovisioning
- Monitor privileged operations

## Cost Optimization

### Resource Right-Sizing

- Monitor actual resource usage
- Scale down during off-peak hours
- Use spot/preemptible instances for non-critical workloads

### Storage Optimization

- Implement lifecycle policies for object storage
- Archive old metadata to cheaper storage tiers
- Clean up orphaned files regularly

## Troubleshooting

### Common Issues

**High Latency**
- Check database query performance
- Review connection pool settings
- Verify network latency to object storage

**Connection Errors**
- Verify database connectivity
- Check connection pool exhaustion
- Review firewall/security group rules

**Authentication Failures**
- Verify JWT secret configuration
- Check token expiration settings
- Review user permissions

## Checklist

### Pre-Deployment
- [ ] Infrastructure provisioned and tested
- [ ] Secrets configured in secrets manager
- [ ] Database initialized and migrated
- [ ] Monitoring and alerting configured
- [ ] Backup procedures tested
- [ ] Load testing completed

### Post-Deployment
- [ ] Health checks passing
- [ ] Metrics being collected
- [ ] Logs aggregating correctly
- [ ] Backup jobs running
- [ ] Documentation updated
- [ ] Team trained on operations

## Additional Resources

- [Scalability Best Practices](./scalability.md)
- [Security Best Practices](./security.md)
- [Monitoring Guide](../features/audit_logs.md)
