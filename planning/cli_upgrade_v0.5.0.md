# CLI Upgrade Implementation Plan (v0.5.0)

**Date**: 2025-12-29
**Status**: **Complete** (Implemented and Verified).

## Goal Description
Enable and verify the `pangolin-admin` CLI commands for managing the Service User lifecycle (`create`, `list`, `get`, `update`, `delete`, `rotate-key`).

## Current State Analysis
*   **Commands Enum (`commands.rs`)**: Contains `CreateServiceUser`, `ListServiceUsers`, `GetServiceUser`, `UpdateServiceUser`, `DeleteServiceUser`, `RotateServiceUserKey`.
*   **Handlers (`handlers/service_users.rs`)**: Contains full implementation of these commands.
*   **Wiring**: Need to confirm `main.rs` dispatches these commands to the handlers (likely yes, but strictly needs checking).

## Proposed Changes (Verification Focus)

### 1. Wiring Verification
#### [CHECK] [pangolin/pangolin_cli_admin/src/main.rs](file:///home/alexmerced/development/personal/Personal/2026/pangolin/pangolin/pangolin_cli_admin/src/main.rs)
*   Ensure the `match command` block includes the Service User variants and calls the appropriate `handlers::service_users::*` functions.

### 2. Manual Verification (Live)
Run the following sequence against the local server (using `cargo run -p pangolin_cli_admin -- ...`).

#### Scenario 1: Lifecycle
1.  **List (Empty)**: `list-service-users` -> Expect empty list.
2.  **Create**: `create-service-user --name cli_bot --role tenant-user` -> Expect API Key output.
    *   *Capture ID*.
3.  **List (Populated)**: `list-service-users` -> Expect 1 row.
4.  **Get**: `get-service-user --id <ID>` -> Expect details.
5.  **Rotate**: `rotate-service-user-key --id <ID>` -> Expect NEW API Key output.
6.  **Delete**: `delete-service-user --id <ID>` -> Expect success.
7.  **List (Empty)**: `list-service-users` -> Expect empty list.

#### Scenario 2: RBAC Interaction
1.  **Create Role**: `create-role --name CliBotRole` -> Capture ID.
2.  **Create User**: `create-service-user --name rbac_bot` -> Capture ID.
3.  **Assign Role (Audit Check)**: `assign-role --user-id <BOT_ID> --role-id <ROLE_ID>` (Is this command `assign-role` or something else? `commands.rs` didn't show `AssignRole`? Wait, checking... `commands.rs` has `GrantPermission` but role assignment might be `UpdateUser`? No, let's re-read `commands.rs` carefully. It has `GrantPermission` and `RevokePermission`. Does it have `AssignRole`?
    *   *Correction*: `commands.rs` seems to lack `AssignRole` command explicitly for users?
    *   *Re-reading commands.rs (lines 55-86)*: It has `CreateUser`, `UpdateUser`.
    *   *Re-reading commands.rs (lines 170+)*: `ListPermissions`, `GrantPermission`, `RevokePermission`.
    *   *Missing Command?* `assign-role` might be missing from CLI entirely?
    *   *Action*: If `AssignRole` is missing, **add it**. The API has `/api/v1/users/{id}/roles`. The CLI should expose it.

## Implementation Tasks

### 1. Fix/Add `AssignRole` Command
If `AssignRole` is missing from the CLI (as suggested by a quick scan of `commands.rs`), add it to `commands.rs` and `handlers/users.rs` (or `handlers/governance.rs`).
*   **Command**: `AssignRole { user_id: String, role_id: String }`
*   **Handler**: `handle_assign_role` calls `POST /api/v1/users/{user_id}/roles`.

### 2. Live Verification
*   Execute the scenarios above.
