# Research Documentation

This directory contains research notes, planning documents, and specifications gathered during development.

## Contents

### Integration Documentation
- `PYICEBERG_INTEGRATION_SUMMARY.md` - Complete PyIceberg integration summary and findings

### Planning Documents
- `requirements.md` - Project requirements and specifications
- `AUDIT_AND_TEST_PLAN.md` - Audit logging and testing strategy
- `SQLITE_BACKEND_PLAN.md` - SQLite backend implementation plan
- `ui_requirements.md` - UI requirements and design specifications

### Specifications
- `rest-catalog-open-api.yaml` - Iceberg REST Catalog OpenAPI specification (official spec)

## Key Findings

### PyIceberg Integration
- PyIceberg 0.10.0 loads credentials from `TableResponse.config`, not separate `/credentials` endpoint
- Credentials must use `s3.access-key-id` format (with `s3.` prefix and hyphens)
- Both `create_table` and `load_table` must include credentials for full functionality
- Snapshot tracking requires storing full `Snapshot` objects, not just IDs

### Iceberg REST Specification
- Credentials endpoint should be GET, not POST
- Response format uses `storage-credentials` array with `prefix` and `config` fields
- Field names use kebab-case (hyphens) not snake_case

## Related Documentation

See also:
- `/docs` - Main documentation directory
- `/tests/pyiceberg` - Integration test suite
- `README.md` - Project overview
- `architecture.md` - System architecture
