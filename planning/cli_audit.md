# CLI Audit Report

**Date**: 2025-12-18 (Updated)
**Scope**: `pangolin_cli_admin`, `pangolin_cli_user`, `pangolin_cli_common`

## Overview
The CLI tools provide a solid interactive REPL experience for both admin and user personas. Recent work has added support for federated catalogs and token generation.

## Feature Status

| Feature | Implemented | Commands Found | Status |\n| :--- | :---: | :--- | :--- |
| **Token Generation** | ✅ | `get-token` (user CLI) | **COMPLETE** - Generates JWT tokens via `/api/v1/tokens`. |
| **Federated Catalogs** | ✅ | `create-federated-catalog`, `list-federated-catalogs`, `delete-federated-catalog`, `test-federated-catalog` (admin CLI) | **COMPLETE** - Full CRUD + testing support. |
| **Branching** | ✅ | `list-branches`, `create-branch`, `merge-branch` | Enhanced with `branch_type` and `assets` support. |
| **Business Metadata** | ✅ | `search`, `get-metadata`, `set-metadata` | Functional in both CLIs. |
| **CRUD (Core)** | ✅ | Various `create-*`, `list-*`, `delete-*` | Covers Tenants, Users, Warehouses, Catalogs. |
| **RBAC / Permissions** | ✅ | `grant-permission`, `revoke-permission`, `list-permissions` | **FIXED** - `list-permissions` now resolves usernames/roles to UUIDs. |

## Recent Fixes (2025-12-18)

### 1. Token Generation ✅
- **Added**: `get-token` command to `pangolin-user`
- **Endpoint**: `POST /api/v1/tokens`
- **Verified**: E2E test passing in `test_cli_live.sh`

### 2. Federated Catalog Management ✅
- **Added**: Complete federated catalog CRUD in `pangolin-admin`
- **Commands**: `create-federated-catalog`, `list-federated-catalogs`, `delete-federated-catalog`, `test-federated-catalog`
- **Verified**: E2E test passing, PyIceberg integration working

### 3. Permission Listing Fix ✅
- **Fixed**: `list-permissions` now resolves usernames and role names to UUIDs before API call
- **Impact**: Prevents 400 Bad Request errors
- **Verified**: E2E test passing

## Conclusion
CLI has reached feature parity with the API for core functionality. All major gaps identified in the original audit have been addressed.
