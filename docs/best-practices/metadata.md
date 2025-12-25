# Business Metadata Management Best Practices

## Overview

This guide covers best practices for managing business metadata in Pangolin to enhance data discovery and governance.

## Metadata Strategy

### Types of Metadata

**Technical Metadata** (Auto-Generated)
- Schema information
- File formats
- Partition specs
- Statistics

**Business Metadata** (User-Defined)
- Business definitions
- Data ownership
- Data quality rules
- Usage guidelines

**Operational Metadata**
- Access patterns
- Query performance
- Data lineage
- Refresh schedules

## Business Metadata Schema

### Standard Fields

```json
{
  "owner": "data-team@company.com",
  "steward": "alice@company.com",
  "domain": "sales",
  "classification": "internal",
  "pii": false,
  "retention_days": 2555,  // 7 years
  "refresh_schedule": "daily",
  "description": "Customer transaction data for sales analytics",
  "tags": ["sales", "customer", "transactions"],
  "quality_score": 95,
  "last_validated": "2025-12-24"
}
```

### Custom Fields by Domain

**Sales Domain**
```json
{
  "sales_region": "north_america",
  "product_category": "software",
  "revenue_impact": "high"
}
```

**Finance Domain**
```json
{
  "gl_account": "4000-revenue",
  "cost_center": "engineering",
  "fiscal_year": "2025"
}
```

## Metadata Management

### Setting Metadata

```bash
# Via CLI
pangolin-admin set-metadata \
  --catalog analytics \
  --namespace sales \
  --asset customers \
  --key owner \
  --value "sales-team@company.com"

# Bulk metadata
pangolin-admin set-metadata \
  --catalog analytics \
  --namespace sales \
  --asset customers \
  --metadata '{
    "owner": "sales-team@company.com",
    "steward": "alice@company.com",
    "domain": "sales",
    "pii": true,
    "description": "Customer master data"
  }'
```

```python
# Via Python
from pypangolin import PangolinClient

client = PangolinClient("http://localhost:8080")
client.login("admin", "password")

# Set metadata
client.set_metadata(
    catalog="analytics",
    namespace="sales",
    asset="customers",
    metadata={
        "owner": "sales-team@company.com",
        "domain": "sales",
        "pii": True
    }
)
```

### Metadata Templates

**Create Reusable Templates**
```yaml
# templates/sales_table.yaml
metadata:
  domain: sales
  owner: sales-team@company.com
  classification: internal
  retention_days: 2555
  tags:
    - sales
    - customer_data
```

**Apply Templates**
```bash
# Apply template to new tables
for table in customers orders products; do
  pangolin-admin set-metadata \
    --catalog analytics \
    --namespace sales \
    --asset $table \
    --template templates/sales_table.yaml
done
```

## Data Governance

### Data Ownership

**Clear Ownership Model**
```
Data Owner (Business)
  ├── Accountable for data quality
  ├── Approves access requests
  └── Defines retention policies

Data Steward (Technical)
  ├── Implements data quality rules
  ├── Manages technical metadata
  └── Monitors data health

Data Consumer (User)
  ├── Uses data responsibly
  ├── Reports quality issues
  └── Follows usage guidelines
```

**Document Ownership**
```json
{
  "table": "sales.customers",
  "owner": {
    "name": "Sales VP",
    "email": "vp-sales@company.com",
    "role": "business_owner"
  },
  "steward": {
    "name": "Alice Johnson",
    "email": "alice@company.com",
    "role": "data_steward"
  },
  "contact": "sales-data-team@company.com"
}
```

### Data Classification

**Classification Levels**
```yaml
classifications:
  public:
    description: "Publicly available data"
    access: "all_users"
    encryption: "optional"
    
  internal:
    description: "Internal business data"
    access: "employees_only"
    encryption: "required"
    
  confidential:
    description: "Sensitive business data"
    access: "need_to_know"
    encryption: "required"
    audit_logging: "mandatory"
    
  restricted:
    description: "Highly sensitive data (PII, financial)"
    access: "explicit_approval"
    encryption: "required"
    audit_logging: "mandatory"
    data_masking: "required"
```

**Apply Classifications**
```bash
# Tag PII data
pangolin-admin set-metadata \
  --catalog production \
  --namespace customers \
  --asset personal_info \
  --metadata '{
    "classification": "restricted",
    "pii": true,
    "pii_fields": ["email", "phone", "ssn"],
    "data_masking_required": true
  }'
```

### Data Quality

**Quality Metadata**
```json
{
  "quality_rules": [
    {
      "rule": "not_null",
      "columns": ["customer_id", "email"],
      "severity": "critical"
    },
    {
      "rule": "unique",
      "columns": ["customer_id"],
      "severity": "critical"
    },
    {
      "rule": "valid_email",
      "columns": ["email"],
      "severity": "high"
    }
  ],
  "quality_score": 98.5,
  "last_quality_check": "2025-12-24T10:00:00Z",
  "quality_issues": 0
}
```

## Discovery & Search

### Searchable Metadata

**Optimize for Discovery**
```json
{
  "title": "Customer Master Data",
  "description": "Comprehensive customer information including demographics, contact details, and account status. Updated daily from CRM system.",
  "keywords": ["customer", "crm", "demographics", "contact"],
  "use_cases": [
    "Customer segmentation",
    "Marketing campaigns",
    "Sales analytics"
  ],
  "sample_queries": [
    "SELECT * FROM customers WHERE region = 'NA' LIMIT 10",
    "SELECT COUNT(*) FROM customers GROUP BY customer_segment"
  ]
}
```

**Rich Descriptions**
```markdown
# Customer Transactions Table

## Overview
Daily transaction data from point-of-sale systems across all retail locations.

## Schema
- `transaction_id`: Unique identifier for each transaction
- `customer_id`: Links to customers table
- `amount`: Transaction amount in USD
- `timestamp`: Transaction timestamp (UTC)

## Usage Guidelines
- Use for sales analysis and customer behavior studies
- Join with customers table for demographic analysis
- Partition by date for performance

## Known Issues
- Transactions before 2024-01-01 may have missing customer_id
- Amount field includes tax (use amount_pretax for revenue calculations)

## Contact
For questions, contact: sales-analytics@company.com
```

### Tagging Strategy

**Hierarchical Tags**
```yaml
tags:
  domain:
    - sales
    - marketing
    - finance
  
  data_type:
    - transactional
    - dimensional
    - aggregated
  
  sensitivity:
    - public
    - internal
    - confidential
  
  geography:
    - north_america
    - europe
    - asia_pacific
```

**Apply Tags**
```bash
# Tag by domain
pangolin-admin set-metadata \
  --catalog analytics \
  --namespace sales \
  --asset transactions \
  --tags "domain:sales,data_type:transactional,geography:north_america"
```

## Metadata Lifecycle

### New Asset Onboarding

```bash
#!/bin/bash
# Metadata onboarding checklist

CATALOG=$1
NAMESPACE=$2
ASSET=$3

# 1. Set basic metadata
pangolin-admin set-metadata \
  --catalog "$CATALOG" \
  --namespace "$NAMESPACE" \
  --asset "$ASSET" \
  --metadata '{
    "created_date": "'$(date -I)'",
    "status": "draft"
  }'

# 2. Assign owner
echo "Who is the data owner? (email)"
read OWNER
pangolin-admin set-metadata --key owner --value "$OWNER"

# 3. Set classification
echo "Classification level? (public/internal/confidential/restricted)"
read CLASSIFICATION
pangolin-admin set-metadata --key classification --value "$CLASSIFICATION"

# 4. Add description
echo "Please provide a description:"
read DESCRIPTION
pangolin-admin set-metadata --key description --value "$DESCRIPTION"

# 5. Mark as active
pangolin-admin set-metadata --key status --value "active"
```

### Metadata Updates

**Version Control**
```json
{
  "metadata_version": "2.0",
  "last_updated": "2025-12-24T15:00:00Z",
  "updated_by": "alice@company.com",
  "change_log": [
    {
      "date": "2025-12-24",
      "field": "owner",
      "old_value": "bob@company.com",
      "new_value": "alice@company.com",
      "reason": "Team reorganization"
    }
  ]
}
```

### Metadata Retirement

```bash
# Mark as deprecated
pangolin-admin set-metadata \
  --catalog analytics \
  --namespace legacy \
  --asset old_table \
  --metadata '{
    "status": "deprecated",
    "deprecated_date": "2025-12-24",
    "replacement": "analytics.new_namespace.new_table",
    "deletion_date": "2026-03-24"
  }'
```

## Automation

### Metadata Extraction

**Auto-Generate from Schema**
```python
def generate_metadata_from_schema(table):
    """Extract metadata from table schema"""
    return {
        "column_count": len(table.schema().fields),
        "columns": [
            {
                "name": field.name,
                "type": str(field.field_type),
                "nullable": field.optional
            }
            for field in table.schema().fields
        ],
        "partition_spec": str(table.spec()),
        "sort_order": str(table.sort_order())
    }
```

**Infer from Usage**
```python
def infer_metadata_from_usage(table_name):
    """Infer metadata from query logs"""
    queries = get_query_logs(table_name)
    
    return {
        "popular_columns": get_most_queried_columns(queries),
        "common_filters": get_common_where_clauses(queries),
        "typical_joins": get_join_patterns(queries),
        "query_frequency": len(queries),
        "unique_users": count_unique_users(queries)
    }
```

### Metadata Validation

**Validation Rules**
```python
def validate_metadata(metadata):
    """Validate required metadata fields"""
    required_fields = ["owner", "description", "classification"]
    
    for field in required_fields:
        if field not in metadata:
            raise ValueError(f"Missing required field: {field}")
    
    if not is_valid_email(metadata["owner"]):
        raise ValueError("Invalid owner email")
    
    if len(metadata["description"]) < 50:
        raise ValueError("Description too short (minimum 50 characters)")
    
    valid_classifications = ["public", "internal", "confidential", "restricted"]
    if metadata["classification"] not in valid_classifications:
        raise ValueError(f"Invalid classification: {metadata['classification']}")
```

## Reporting

### Metadata Coverage

```sql
-- Check metadata completeness
SELECT 
    catalog,
    namespace,
    COUNT(*) as total_assets,
    SUM(CASE WHEN owner IS NOT NULL THEN 1 ELSE 0 END) as with_owner,
    SUM(CASE WHEN description IS NOT NULL THEN 1 ELSE 0 END) as with_description,
    SUM(CASE WHEN classification IS NOT NULL THEN 1 ELSE 0 END) as with_classification,
    ROUND(100.0 * SUM(CASE WHEN owner IS NOT NULL THEN 1 ELSE 0 END) / COUNT(*), 2) as owner_coverage_pct
FROM assets
GROUP BY catalog, namespace
ORDER BY owner_coverage_pct ASC;
```

### Metadata Quality Dashboard

```python
def generate_metadata_dashboard():
    """Generate metadata quality metrics"""
    return {
        "total_assets": count_all_assets(),
        "metadata_coverage": {
            "owner": calculate_coverage("owner"),
            "description": calculate_coverage("description"),
            "classification": calculate_coverage("classification"),
            "tags": calculate_coverage("tags")
        },
        "data_quality": {
            "avg_quality_score": get_avg_quality_score(),
            "assets_with_quality_rules": count_assets_with_quality_rules()
        },
        "governance": {
            "pii_assets": count_pii_assets(),
            "classified_assets": count_classified_assets(),
            "orphaned_assets": count_orphaned_assets()  # No owner
        }
    }
```

## Best Practices Checklist

### For Data Owners
- [ ] Assign clear ownership to all datasets
- [ ] Provide comprehensive descriptions
- [ ] Classify data appropriately
- [ ] Define data quality rules
- [ ] Document retention policies
- [ ] Review metadata quarterly

### For Data Stewards
- [ ] Validate metadata completeness
- [ ] Enforce metadata standards
- [ ] Monitor data quality
- [ ] Update technical metadata
- [ ] Maintain data lineage
- [ ] Automate where possible

### For Data Consumers
- [ ] Search metadata before requesting access
- [ ] Report metadata inaccuracies
- [ ] Follow usage guidelines
- [ ] Provide feedback on data quality
- [ ] Document new use cases

## Additional Resources

- [Business Catalog Guide](../features/business_catalog.md)
- [Tag Management](../features/tag_management.md)
- [Data Discovery](../ui/discovery_governance.md)
