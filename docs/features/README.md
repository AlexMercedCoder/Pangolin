# Features

Advanced features and capabilities of Pangolin catalog.

## Git-Like Features

### [Branch Management](branch_management.md)
- Create and manage branches
- Merge branches
- Branch-specific table versions
- Isolated development environments

### [Tag Management](tag_management.md)
- Create immutable tags
- Reference specific points in time
- Release management

### [Time Travel](time_travel.md)
- Query historical data
- Access previous table versions
- Snapshot-based queries

## Data Management

### [Warehouse Management](warehouse_management.md)
- Create and configure warehouses
- Multi-cloud storage support
- Credential management

### [Asset Management](asset_management.md)
- Manage tables and views
- Asset metadata
- Asset lifecycle

### [Maintenance](maintenance.md)
- Table optimization
- Snapshot expiration
- Orphan file removal

## Security & Access

### [Security & Credential Vending](security_vending.md)
- Automatic credential vending
- Warehouse-based credentials
- Multi-cloud support

### [Audit Logs](audit_logs.md)
- Track all operations
- Compliance and monitoring
- Event history

## Integration

### [PyIceberg Testing](pyiceberg_testing.md)
- PyIceberg integration guide
- Testing scenarios
- Example workflows
- Credential vending with PyIceberg

## System

### [Entities](entities.md)
- Core data model
- Entity relationships
- Schema definitions

### [RBAC UI](rbac_ui.md)
- Role-based access control
- UI management
- Permission system

## Feature Status

| Feature | Status | Description |
|---------|--------|-------------|
| Branch Management | ✅ Working | Git-like branching |
| Tag Management | ✅ Working | Immutable tags |
| Time Travel | ✅ Working | Snapshot queries |
| Credential Vending | ✅ Working | S3, Azure, GCS |
| PyIceberg Integration | ✅ Working | Full compatibility |
| Audit Logs | ✅ Working | Event tracking |
| Warehouse Management | ✅ Working | Multi-cloud |
| Maintenance | 📝 Documented | Optimization tools |
| RBAC | 📝 Documented | Access control |

## Contents

| Document | Description |
|----------|-------------|
| [branch_management.md](branch_management.md) | Git-like branch operations |
| [tag_management.md](tag_management.md) | Immutable tag management |
| [time_travel.md](time_travel.md) | Historical queries |
| [warehouse_management.md](warehouse_management.md) | Warehouse configuration |
| [asset_management.md](asset_management.md) | Table and view management |
| [maintenance.md](maintenance.md) | Table maintenance operations |
| [security_vending.md](security_vending.md) | Credential vending |
| [audit_logs.md](audit_logs.md) | Audit logging |
| [pyiceberg_testing.md](pyiceberg_testing.md) | PyIceberg integration |
| [entities.md](entities.md) | Data model entities |
| [rbac_ui.md](rbac_ui.md) | RBAC and UI |

## See Also

- [Getting Started](../getting-started/) - Setup guide
- [Storage](../warehouse/) - Storage configuration
- [API](../api/) - REST API reference
