# Admin CLI - Business Metadata & Governance

This guide covers business metadata management and access control features.

---

## Delete Metadata

Delete business metadata from an asset.

### Syntax
```bash
pangolin-admin delete-metadata --asset-id <ASSET_ID>
```

### Parameters
- `--asset-id` - UUID of the asset (required)

### Example
```bash
pangolin-admin delete-metadata --asset-id "asset-uuid"
```

### Output
```
✅ Business metadata deleted successfully!
```

### Notes
- Removes all business metadata from the asset
- Does not delete the asset itself
- Cannot be undone
- Requires appropriate permissions

---

## Request Access

Request access to a restricted asset.

### Syntax
```bash
pangolin-admin request-access --asset-id <ASSET_ID> --reason <REASON>
```

### Parameters
- `--asset-id` - UUID of the asset (required)
- `--reason` - Justification for access request (required)

### Example
```bash
pangolin-admin request-access \
  --asset-id "asset-uuid" \
  --reason "Need access for Q4 analysis project"
```

### Output
```
✅ Access request submitted successfully!
Request ID: request-uuid
```

### Notes
- Creates a pending access request
- Admin must approve/deny the request
- Reason should be clear and specific
- Request ID used for tracking

---

## List Access Requests

List all access requests (admin view).

### Syntax
```bash
pangolin-admin list-access-requests
```

### Parameters
None

### Example
```bash
pangolin-admin list-access-requests
```

### Output
```
+--------------------------------------+----------+---------+---------+---------------------------+---------------------+
| ID                                   | Asset ID | User ID | Status  | Reason                    | Created At          |
+--------------------------------------+----------+---------+---------+---------------------------+---------------------+
| request-uuid-1                       | asset-1  | user-1  | pending | Q4 analysis project       | 2025-12-18T10:00:00 |
+--------------------------------------+----------+---------+---------+---------------------------+---------------------+
| request-uuid-2                       | asset-2  | user-2  | approved| Data quality validation   | 2025-12-18T09:30:00 |
+--------------------------------------+----------+---------+---------+---------------------------+---------------------+
```

### Notes
- Shows all access requests in the system
- Status: pending, approved, denied
- Admins can see all requests
- Users see only their own requests

---

## Update Access Request

Update the status of an access request (approve/deny).

### Syntax
```bash
pangolin-admin update-access-request --id <REQUEST_ID> --status <STATUS>
```

### Parameters
- `--id` - UUID of the access request (required)
- `--status` - New status: approved, denied (required)

### Examples
```bash
# Approve request
pangolin-admin update-access-request \
  --id "request-uuid" \
  --status "approved"

# Deny request
pangolin-admin update-access-request \
  --id "request-uuid" \
  --status "denied"
```

### Output
```
✅ Access request updated to: approved
```

### Notes
- Requires admin privileges
- User is notified of status change
- Approved requests grant access
- Denied requests can be re-requested

---

## Get Asset Details

Get detailed information about an asset.

### Syntax
```bash
pangolin-admin get-asset-details --id <ASSET_ID>
```

### Parameters
- `--id` - UUID of the asset (required)

### Example
```bash
pangolin-admin get-asset-details --id "asset-uuid"
```

### Output
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Asset Details
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ID: asset-uuid
Name: customer_data
Type: table
Catalog: production-catalog
Description: Customer master data table
Discoverable: true
Created At: 2025-12-18T08:00:00
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### Notes
- Shows comprehensive asset information
- Includes metadata and properties
- Discoverable flag indicates if asset is searchable
- Useful for governance and auditing

---

## Complete Governance Workflow

### 1. User Requests Access
```bash
# User discovers an asset
pangolin-admin search-assets --query "customer"

# User requests access
pangolin-admin request-access \
  --asset-id "customer-data-uuid" \
  --reason "Customer segmentation analysis for marketing campaign"
```

### 2. Admin Reviews Request
```bash
# Admin lists pending requests
pangolin-admin list-access-requests

# Admin reviews asset details
pangolin-admin get-asset-details --id "customer-data-uuid"

# Admin approves request
pangolin-admin update-access-request \
  --id "request-uuid" \
  --status "approved"
```

### 3. User Accesses Data
```bash
# User can now access the asset
# Access is granted through permissions system
```

---

## Access Request Workflow

```
User Discovers Asset
        ↓
Request Access (with reason)
        ↓
Admin Reviews Request
        ↓
    Approve/Deny
        ↓
User Granted/Denied Access
```

---

## Metadata Management

### Adding Metadata
```bash
# Use set-metadata command
pangolin-admin set-metadata \
  --entity-type "table" \
  --entity-id "table-uuid" \
  "owner" "data-team"
```

### Viewing Metadata
```bash
# Use get-metadata command
pangolin-admin get-metadata \
  --entity-type "table" \
  --entity-id "table-uuid"
```

### Deleting Metadata
```bash
# Remove all metadata
pangolin-admin delete-metadata --asset-id "table-uuid"
```

---

## Best Practices

### Access Requests

1. **Clear Reasons**: Provide specific, detailed justification
2. **Timely Review**: Admins should review requests promptly
3. **Documentation**: Keep audit trail of approvals/denials
4. **Regular Audits**: Review granted access periodically
5. **Revocation**: Remove access when no longer needed

### Metadata Management

1. **Consistent Schema**: Use standard metadata keys
2. **Regular Updates**: Keep metadata current
3. **Ownership**: Assign clear data owners
4. **Classification**: Tag sensitive data appropriately
5. **Discoverability**: Make relevant assets discoverable

---

## Access Request Templates

### Data Analysis
```bash
pangolin-admin request-access \
  --asset-id "asset-uuid" \
  --reason "Data analysis for [project name]. Required for [specific task]. Duration: [timeframe]."
```

### Reporting
```bash
pangolin-admin request-access \
  --asset-id "asset-uuid" \
  --reason "Monthly reporting requirements. Need access to generate [report name]."
```

### Development
```bash
pangolin-admin request-access \
  --asset-id "asset-uuid" \
  --reason "Development of [feature name]. Need sample data for testing."
```

---

## Admin Review Checklist

Before approving access requests:

- [ ] Verify user identity
- [ ] Check reason is legitimate
- [ ] Confirm data classification allows access
- [ ] Ensure compliance requirements met
- [ ] Verify user has appropriate training
- [ ] Set access expiration if needed
- [ ] Document approval decision

---

## Error Handling

### Common Errors

**Asset Not Found**:
```
Error: Failed to get asset details (404): Asset not found
```
- Solution: Verify asset ID is correct

**Duplicate Request**:
```
Error: Failed to request access (400): Active request already exists
```
- Solution: Wait for existing request to be processed

**Insufficient Permissions**:
```
Error: Failed to update access request (403): Admin privileges required
```
- Solution: Only admins can approve/deny requests

---

## Governance Features

### Data Discovery
- Search assets by metadata
- Filter by classification
- View asset lineage

### Access Control
- Request-based access
- Admin approval workflow
- Audit trail

### Compliance
- Track data access
- Document approvals
- Regular access reviews

---

## Related Commands

- `search-assets` - Search for assets by metadata
- `get-metadata` - View asset metadata
- `set-metadata` - Add/update metadata
- `grant-permission` - Grant direct permissions

---

## Metadata Standards

### Recommended Keys
- `owner` - Data owner/team
- `classification` - Data sensitivity level
- `retention` - Data retention policy
- `pii` - Contains PII (true/false)
- `description` - Asset description
- `tags` - Searchable tags

### Example
```bash
pangolin-admin set-metadata \
  --entity-type "table" \
  --entity-id "table-uuid" \
  "owner" "data-engineering"

pangolin-admin set-metadata \
  --entity-type "table" \
  --entity-id "table-uuid" \
  "classification" "confidential"

pangolin-admin set-metadata \
  --entity-type "table" \
  --entity-id "table-uuid" \
  "pii" "true"
```

---

## Compliance Reporting

### Access Audit
```bash
# List all access requests
pangolin-admin list-access-requests > access_audit.txt

# Filter by status
grep "approved" access_audit.txt

# Count pending requests
grep "pending" access_audit.txt | wc -l
```

### Asset Inventory
```bash
# Get details for all assets
for asset_id in "${ASSET_IDS[@]}"; do
  pangolin-admin get-asset-details --id "$asset_id"
done
```

---

## Tips

- **Automate Reviews**: Set up workflows for common request types
- **SLAs**: Define response times for access requests
- **Training**: Ensure users understand governance policies
- **Monitoring**: Track access patterns and anomalies
- **Documentation**: Maintain clear governance documentation
