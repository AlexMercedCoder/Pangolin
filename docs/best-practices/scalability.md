# Scalability Best Practices

## Overview

This guide covers strategies for scaling Pangolin to handle growing data volumes, user bases, and query loads.

## Scaling Dimensions

### Vertical Scaling (Scale Up)

**When to Scale Up**
- Single-tenant deployments with predictable growth
- Cost-effective for small to medium workloads
- Simpler operational model

**Limits**
- Hardware constraints (CPU, RAM, disk)
- Single point of failure
- Downtime required for upgrades

### Horizontal Scaling (Scale Out)

**When to Scale Out**
- Multi-tenant deployments
- Unpredictable or bursty workloads
- High availability requirements
- Large-scale deployments (100+ users, 1000+ tables)

## API Server Scaling

### Stateless Design

Pangolin API servers are stateless, enabling easy horizontal scaling.

**Load Balancing Strategy**
```
┌──────────────┐
│ Load Balancer│
└──────┬───────┘
       │
   ┌───┴────┬────────┬────────┐
   │        │        │        │
┌──▼──┐  ┌──▼──┐  ┌──▼──┐  ┌──▼──┐
│ API │  │ API │  │ API │  │ API │
│  1  │  │  2  │  │  3  │  │  4  │
└──┬──┘  └──┬──┘  └──┬──┘  └──┬──┘
   │        │        │        │
   └────────┴────────┴────────┘
              │
       ┌──────▼──────┐
       │  PostgreSQL │
       └─────────────┘
```

**Scaling Triggers**
- CPU utilization > 70%
- Request latency p95 > 500ms
- Request queue depth > 100

**Auto-Scaling Configuration (Kubernetes)**
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: pangolin-api-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: pangolin-api
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

## Database Scaling

### PostgreSQL Scaling

**Read Replicas**
```
┌─────────────┐
│   Primary   │ ◄── Writes
└──────┬──────┘
       │ Replication
   ┌───┴────┬────────┐
   │        │        │
┌──▼──┐  ┌──▼──┐  ┌──▼──┐
│Read │  │Read │  │Read │ ◄── Reads
│Rep 1│  │Rep 2│  │Rep 3│
└─────┘  └─────┘  └─────┘
```

**Connection Pooling**
```bash
# PgBouncer configuration
[databases]
pangolin = host=localhost port=5432 dbname=pangolin

[pgbouncer]
pool_mode = transaction
max_client_conn = 1000
default_pool_size = 20
reserve_pool_size = 5
```

**Partitioning Strategy**
```sql
-- Partition audit logs by month
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY,
    timestamp TIMESTAMP NOT NULL,
    action VARCHAR(50),
    ...
) PARTITION BY RANGE (timestamp);

CREATE TABLE audit_logs_2025_01 PARTITION OF audit_logs
    FOR VALUES FROM ('2025-01-01') TO ('2025-02-01');
```

### MongoDB Scaling

**Sharding**
```javascript
// Shard by tenant_id for multi-tenancy
sh.shardCollection("pangolin.assets", { tenant_id: 1, catalog: 1 })

// Shard by catalog for single-tenant
sh.shardCollection("pangolin.assets", { catalog: 1, namespace: 1 })
```

## Multi-Tenancy Scaling

### Tenant Isolation Strategies

**Shared Infrastructure (Cost-Effective)**
- All tenants share same API servers and database
- Logical isolation via tenant_id
- Suitable for: 100s-1000s of small tenants

**Dedicated Infrastructure (High Isolation)**
- Each tenant gets dedicated resources
- Physical isolation
- Suitable for: Enterprise customers, compliance requirements

**Hybrid Approach (Recommended)**
- Shared API servers
- Tenant-specific database schemas or databases
- Balance of cost and isolation

### Tenant-Specific Scaling

```sql
-- Monitor per-tenant resource usage
SELECT 
    tenant_id,
    COUNT(*) as table_count,
    SUM(size_bytes) as total_size
FROM assets
GROUP BY tenant_id
ORDER BY total_size DESC;
```

## Catalog Scaling

### Namespace Organization

**Flat Structure (Small Scale)**
```
catalog
├── namespace1
├── namespace2
└── namespace3
```

**Hierarchical Structure (Large Scale)**
```
catalog
├── department
│   ├── team1
│   │   ├── project_a
│   │   └── project_b
│   └── team2
└── shared
```

**Best Practices**
- Limit namespace depth to 3-4 levels
- Use consistent naming conventions
- Implement namespace quotas

### Table Distribution

**Anti-Pattern: Single Mega-Catalog**
```
❌ production (10,000 tables)
```

**Best Practice: Multiple Catalogs**
```
✅ sales_analytics (500 tables)
✅ marketing_data (300 tables)
✅ finance_reports (200 tables)
✅ operations_metrics (400 tables)
```

## Metadata Scaling

### Metadata Size Management

**Audit Log Archival**
```sql
-- Archive logs older than 90 days
INSERT INTO audit_logs_archive
SELECT * FROM audit_logs
WHERE timestamp < NOW() - INTERVAL '90 days';

DELETE FROM audit_logs
WHERE timestamp < NOW() - INTERVAL '90 days';
```

**Snapshot Cleanup**
```bash
# Remove snapshots older than retention policy
# Implement via maintenance operations
pangolin-admin optimize-catalog --catalog production \
  --remove-old-snapshots --retention-days 30
```

### Indexing Strategy

**Essential Indexes**
```sql
-- Catalog operations
CREATE INDEX idx_assets_catalog_namespace ON assets(catalog_id, namespace);
CREATE INDEX idx_assets_tenant ON assets(tenant_id);

-- Search and discovery
CREATE INDEX idx_assets_name ON assets(name);
CREATE INDEX idx_assets_kind ON assets(kind);

-- Audit queries
CREATE INDEX idx_audit_tenant_time ON audit_logs(tenant_id, timestamp DESC);
CREATE INDEX idx_audit_action ON audit_logs(action);
```

## Caching Strategies

### Metadata Caching

**Redis Configuration**
```yaml
# Cache frequently accessed metadata
cache:
  type: redis
  host: redis-cluster
  ttl: 300  # 5 minutes
  
  # Cache keys
  patterns:
    - "catalog:*:metadata"
    - "namespace:*:list"
    - "permissions:user:*"
```

**Cache Invalidation**
- Invalidate on write operations
- Use pub/sub for multi-instance coordination
- Implement cache warming for critical data

### Query Result Caching

```python
# Cache expensive queries
@cache(ttl=600)  # 10 minutes
def get_catalog_summary(catalog_id):
    return {
        'table_count': count_tables(catalog_id),
        'total_size': calculate_size(catalog_id),
        'namespaces': list_namespaces(catalog_id)
    }
```

## Object Storage Scaling

### S3 Performance Optimization

**Prefix Distribution**
```
# Anti-pattern: Sequential prefixes
s3://bucket/table-00001/
s3://bucket/table-00002/
s3://bucket/table-00003/

# Best practice: Hash-based prefixes
s3://bucket/a7f3/table-00001/
s3://bucket/b2e9/table-00002/
s3://bucket/c4d1/table-00003/
```

**Request Rate Limits**
- S3: 3,500 PUT/COPY/POST/DELETE, 5,500 GET/HEAD per prefix per second
- Use multiple prefixes for high-throughput workloads

### Multi-Region Storage

```
Primary Region: us-east-1
├── Catalog metadata: RDS us-east-1
└── Table data: S3 us-east-1

Secondary Region: eu-west-1
├── Read replica: RDS eu-west-1
└── Replicated data: S3 eu-west-1
```

## Performance Benchmarks

### Target Metrics

**API Response Times**
- List catalogs: < 100ms (p95)
- Get table metadata: < 50ms (p95)
- Create table: < 200ms (p95)
- Search: < 500ms (p95)

**Throughput**
- 1,000 requests/second per API instance
- 10,000 concurrent users per deployment

**Scale Limits**
- 100,000 tables per catalog
- 1,000 catalogs per deployment
- 10,000 users per tenant

## Monitoring for Scale

### Key Performance Indicators

```sql
-- Track growth trends
SELECT 
    DATE_TRUNC('day', created_at) as date,
    COUNT(*) as new_tables,
    SUM(COUNT(*)) OVER (ORDER BY DATE_TRUNC('day', created_at)) as cumulative
FROM assets
GROUP BY DATE_TRUNC('day', created_at)
ORDER BY date DESC
LIMIT 30;
```

### Capacity Planning

**Forecast Resource Needs**
- Monitor growth rate (tables, users, queries)
- Project 6-12 months ahead
- Plan scaling actions proactively

**Alerting Thresholds**
- Database connections > 80% of pool
- Disk usage > 70%
- API latency p95 > 500ms
- Error rate > 1%

## Cost Optimization at Scale

### Resource Efficiency

**Right-Sizing**
- Start small, scale based on actual usage
- Use auto-scaling to handle peaks
- Scale down during off-hours

**Storage Tiering**
```bash
# Move old data to cheaper storage
aws s3 cp s3://hot-bucket/old-data/ s3://archive-bucket/old-data/ \
  --storage-class GLACIER_IR
```

### Multi-Tenancy Economics

**Shared Resources**
- API servers: Shared across all tenants
- Database: Shared with logical isolation
- Monitoring: Centralized

**Dedicated Resources (Premium Tier)**
- Dedicated API instances
- Isolated database
- Priority support

## Scaling Checklist

### Before Scaling
- [ ] Identify bottleneck (CPU, memory, database, network)
- [ ] Review current resource utilization
- [ ] Analyze query patterns and hot spots
- [ ] Test scaling strategy in staging

### During Scaling
- [ ] Monitor metrics continuously
- [ ] Validate health checks
- [ ] Test failover scenarios
- [ ] Verify data consistency

### After Scaling
- [ ] Measure performance improvement
- [ ] Update capacity planning
- [ ] Document changes
- [ ] Review cost impact

## Additional Resources

- [Deployment Best Practices](./deployment.md)
- [Performance Tuning Guide](../features/maintenance.md)
- [Database Optimization](../backend_storage/README.md)
