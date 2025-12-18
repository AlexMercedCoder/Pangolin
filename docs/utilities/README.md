# Utilities

This directory contains utility documentation for maintaining and working with the Pangolin project.

## Available Guides

### [Regenerating OpenAPI Documentation](./regenerating-openapi.md)
Complete guide for regenerating the OpenAPI specification files (JSON and YAML) after making changes to API handlers or models.

**Quick Commands**:
```bash
# Generate JSON
cd pangolin && cargo run -p pangolin_api --bin export_openapi json 2>/dev/null > ../docs/api/openapi.json

# Generate YAML
cd pangolin && cargo run -p pangolin_api --bin export_openapi yaml 2>/dev/null > ../docs/api/openapi.yaml
```

## Future Utilities

Additional utility guides will be added here as needed:
- Database migrations
- Test data generation
- Performance benchmarking
- Deployment scripts
- Backup and restore procedures
