# Release v0.2.0 Audit Findings

**Date**: 2025-12-24
**Version**: v0.2.0 (Docker Images)

## Summary
Tracking issues found during end-to-end testing of Docker images with MinIO.

## Issues

| ID | Severity | Description | Status |
|----|----------|-------------|--------|
| FIND-01 | Medium | `VendingStrategy` enum uses external tagging (default) instead of internal tagging (`type` field) as documented. API expects `"vending_strategy": "None"` or `{ "AwsSts": ... }` instead of `{ "type": "AwsSts", ... }`. | Open - Code needs `#[serde(tag="type")]` |

## Test Log

### No Auth Mode
- [ ] Environment Setup (Docker + MinIO)
- [ ] Warehouse Creation (MinIO)
- [ ] Catalog/Namespace Creation
- [ ] PyIceberg: Create Table
- [ ] PyIceberg: Insert/Read/Update
- [ ] "Easy Auth" Role Masquerading

### Auth Mode
- [ ] Environment Setup (Docker + MinIO)
- [ ] Root Login
- [ ] Tenant Creation
- [ ] User Creation (Tenant Admin & Analyst)
- [ ] RBAC Enforcement (Analyst vs Admin)
- [ ] PyIceberg: Credential Vending (STS/MinIO)

### CLI Validation
- [ ] Admin CLI: Login
- [ ] Admin CLI: Create Resources
- [ ] User CLI: List Resources

## Deployment Assets Audit
- **Status**: Verified & Updated.
- **Findings**:
    - Validated `docs/getting-started` and `docs/pyiceberg` content.
    - Audited `deployment_assets` structure (`production` templates, `demo` environments).
    - **Fix**: Updated `deployment_assets/demo/evaluate_single_tenant/demo_client.py` to include `header.X-Pangolin-Tenant` for best practice compliance.
    - Verified `evaluate_multi_tenant` instructions are accurate for `Auth` mode.
    - Confirmed `latest` tag usage is appropriate for generic assets.
