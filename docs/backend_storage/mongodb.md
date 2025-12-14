# MongoDB Backend Storage

MongoDB is a popular NoSQL document database that provides excellent scalability and flexibility for Pangolin's metadata storage.

## Overview

MongoDB stores Pangolin catalog metadata as JSON documents with:
- Flexible schema evolution
- Horizontal scalability through sharding
- Built-in replication
- Rich query capabilities
- Cloud-native architecture

## Pros and Cons

### ✅ Advantages
- **Horizontal Scalability**: Shard across multiple servers
- **Flexible Schema**: Easy to add new fields without migrations
- **Cloud-Native**: Excellent managed options (MongoDB Atlas)
- **JSON-Native**: Natural fit for metadata storage
- **Aggregation Framework**: Powerful data processing
- **Change Streams**: Real-time data monitoring
- **Geographic Distribution**: Multi-region deployments

### ⚠️ Considerations
- **No Foreign Keys**: Application-level referential integrity
- **Memory Usage**: Keeps working set in RAM
- **Learning Curve**: Different from SQL databases
- **Cost**: Atlas pricing can be higher than RDS

## Use Cases

**Best For**:
- Cloud-native deployments
- Applications requiring horizontal scaling
- Flexible schema requirements
- Multi-region deployments
- When you're already using MongoDB

**Not Ideal For**:
- When you need strict foreign key constraints
- Traditional SQL-heavy applications
- Embedded deployments
- When you need complex joins

## Prerequisites

- MongoDB 5.0+ installed
- Database created for Pangolin
- User with readWrite permissions

## Installation

### Local Development (Docker)

```bash
# Start MongoDB
docker run -d \
  --name pangolin-mongo \
  -e MONGO_INITDB_ROOT_USERNAME=admin \
  -e MONGO_INITDB_ROOT_PASSWORD=your_password \
  -e MONGO_INITDB_DATABASE=pangolin \
  -p 27017:27017 \
  mongo:7

# Create Pangolin user
docker exec -it pangolin-mongo mongosh -u admin -p your_password --authenticationDatabase admin

use pangolin
db.createUser({
  user: "pangolin",
  pwd: "pangolin_password",
  roles: [{ role: "readWrite", db: "pangolin" }]
})
```

### Production (MongoDB Atlas)

1. Create cluster at [cloud.mongodb.com](https://cloud.mongodb.com)
2. Create database user
3. Whitelist IP addresses
4. Get connection string

## Configuration

### Environment Variables

```bash
# Connection string format:
# mongodb://username:password@host:port/database

# Local development
DATABASE_URL=mongodb://pangolin:pangolin_password@localhost:27017/pangolin

# MongoDB Atlas
DATABASE_URL=mongodb+srv://pangolin:password@cluster0.xxxxx.mongodb.net/pangolin?retryWrites=true&w=majority

# With replica set
DATABASE_URL=mongodb://pangolin:password@host1:27017,host2:27017,host3:27017/pangolin?replicaSet=rs0

# Additional settings
MONGO_DB_NAME=pangolin
MONGO_MAX_POOL_SIZE=10
MONGO_MIN_POOL_SIZE=2
```

### Collections

Pangolin creates the following collections:
- `tenants` - Tenant information
- `warehouses` - Storage configurations
- `catalogs` - Catalog definitions
- `namespaces` - Namespace hierarchies
- `assets` - Table and view metadata
- `branches` - Branch information
- `tags` - Tag information
- `commits` - Commit history
- `metadata_locations` - Metadata file locations
- `audit_logs` - Audit trail

### Indexes

Automatically created indexes:
```javascript
// Tenants
db.tenants.createIndex({ "name": 1 }, { unique: true })

// Warehouses
db.warehouses.createIndex({ "tenant_id": 1, "name": 1 }, { unique: true })

// Catalogs
db.catalogs.createIndex({ "tenant_id": 1, "name": 1 }, { unique: true })

// Namespaces
db.namespaces.createIndex({ "tenant_id": 1, "catalog_name": 1, "namespace_path": 1 }, { unique: true })

// Assets
db.assets.createIndex({ "tenant_id": 1, "catalog_name": 1 })
db.assets.createIndex({ "namespace_path": 1 })
```

## Performance Tuning

### Recommended Settings

```javascript
// Enable profiling for slow queries
db.setProfilingLevel(1, { slowms: 100 })

// Check current profile level
db.getProfilingStatus()

// View slow queries
db.system.profile.find().sort({ ts: -1 }).limit(10)
```

### Monitoring

```javascript
// Check database stats
db.stats()

// Check collection stats
db.catalogs.stats()

// Current operations
db.currentOp()

// Server status
db.serverStatus()
```

## Backup and Restore

### MongoDB Atlas

Atlas provides:
- Continuous backups
- Point-in-time recovery
- Cross-region backups

### Manual Backups

```bash
# Backup
mongodump --uri="mongodb://pangolin:password@localhost:27017/pangolin" --out=/backup/pangolin

# Restore
mongorestore --uri="mongodb://pangolin:password@localhost:27017/pangolin" /backup/pangolin/pangolin

# Backup specific collection
mongodump --uri="mongodb://..." --collection=catalogs --out=/backup

# Restore specific collection
mongorestore --uri="mongodb://..." --collection=catalogs /backup/pangolin/catalogs.bson
```

## High Availability

### Replica Sets

```bash
# Initialize replica set
mongosh --eval "rs.initiate({
  _id: 'rs0',
  members: [
    { _id: 0, host: 'mongo1:27017' },
    { _id: 1, host: 'mongo2:27017' },
    { _id: 2, host: 'mongo3:27017' }
  ]
})"

# Check replica set status
mongosh --eval "rs.status()"
```

### Sharding

For very large deployments:
```bash
# Enable sharding on database
sh.enableSharding("pangolin")

# Shard collection by tenant_id
sh.shardCollection("pangolin.catalogs", { "tenant_id": 1 })
```

## Security Best Practices

1. **Authentication**: Always enable authentication
2. **TLS/SSL**: Use encrypted connections
3. **Network Security**: Bind to specific IPs, use firewalls
4. **Least Privilege**: Grant minimal required permissions
5. **Audit Logging**: Enable MongoDB audit logs
6. **Encryption at Rest**: Enable for Atlas or self-managed

## Troubleshooting

### Connection Issues

```bash
# Test connection
mongosh "mongodb://pangolin:password@localhost:27017/pangolin"

# Check if MongoDB is running
sudo systemctl status mongod

# Check logs
sudo tail -f /var/log/mongodb/mongod.log
```

### Performance Issues

```javascript
// Explain query
db.catalogs.find({ tenant_id: "..." }).explain("executionStats")

// Check index usage
db.catalogs.aggregate([
  { $indexStats: {} }
])

// Compact collection
db.runCommand({ compact: 'catalogs' })
```

## Test Results

✅ **All 5 MongoDB tests passing**:
- `test_mongo_tenant_crud`
- `test_mongo_warehouse_crud`
- `test_mongo_catalog_crud`
- `test_mongo_namespace_operations`
- `test_mongo_multi_tenant_isolation`

## Additional Resources

- [MongoDB Official Documentation](https://docs.mongodb.com/)
- [MongoDB Atlas](https://www.mongodb.com/cloud/atlas)
- [MongoDB University](https://university.mongodb.com/)
- [Performance Best Practices](https://docs.mongodb.com/manual/administration/analyzing-mongodb-performance/)

## Next Steps

- [PostgreSQL Backend](postgresql.md)
- [SQLite Backend](sqlite.md)
- [Backend Comparison](comparison.md)
- [Warehouse Storage](../warehouse/README.md)
