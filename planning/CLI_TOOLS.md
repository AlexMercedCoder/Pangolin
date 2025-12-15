# CLI Tools Project Plan

**Date**: 2025-12-15
**Status**: Planning

## Overview

This document outlines the plan for creating two distinct CLI tools for the Pangolin ecosystem. These tools will be separate binaries from the main server, designed to provide interactive, shell-like experiences for different user personas.

- **`pangolin-admin`**: A tool for Root Admins and Tenant Admins to manage the system.
- **`pangolin-user`**: A tool for end-users (Data Engineers, Analysts) to discover data and generate configuration.

## Core Requirements

- **Language**: Rust
- **Interaction Model**: Interactive REPL/Shell (Read-Eval-Print Loop).
- **Authentication**: Prompt for Username/Password at startup. Authenticate against Pangolin API.
- **Output**: Rich text output (tables, JSON, syntax highlighted code).
- **Configuration**: Persist session/config (optional, but primarily session-based).

## 1. `pangolin-admin`

**Target Audience**: Root Admins, Tenant Admins

### Functionality
*   **Authentication**: Login flow (Root or Tenant Admin).
*   **Tenant Management** (Root only):
    *   `create-tenant`, `list-tenants`, `delete-tenant`
*   **User Management**:
    *   `create-user`, `list-users`, `reset-password`, `assign-role`
*   **Infrastructure Management**:
    *   `create-warehouse`, `list-warehouses`
    *   `create-catalog`, `list-catalogs`
*   **System Health**:
    *   `check-health`, `view-audit-logs`

### Proposed Commands (REPL)
```bash
> login
Username: admin
Password: ***
Logged in as Root Admin.

> list-tenants
| ID | Name | Status |
|----|------|--------|
| ...| ...  | ...    |

> switch-tenant <tenant-id>
Switched context to Tenant: acme-corp

> create-user --email bob@acme.com --role data-engineer
User created.
```

## 2. `pangolin-user`

**Target Audience**: Data Consumers, Data Engineers, Data Scientists

### Functionality
*   **Authentication**: Login flow (Standard User).
*   **Data Discovery**:
    *   `list-catalogs`, `list-namespaces`, `list-tables`
    *   `search <query>` (Metadata search)
    *   `describe <table>` (Show schema, partitioning)
*   **Access Management**:
    *   `request-access <asset>`
    *   `my-requests` (View status)
*   **Code Generation**:
    *   `get-token` (Generate API token for scripts)
    *   `sample-code pyiceberg --table <name>`
    *   `sample-code pyspark --table <name>`
    *   `sample-code dremio --table <name>`
*   **Branching (User-facing)**:
    *   `create-branch`, `list-branches`

### Proposed Commands (REPL)
```bash
> login
Username: alice
Password: ***
Logged in as Alice.

> search "sales_data"
Found 3 tables:
1. raw.sales_data
2. cur.sales_data_aggregated

> describe raw.sales_data
Schema:
 - order_id (long)
 - amount (double)
...

> sample-code pyiceberg --table raw.sales_data
# Generated PyIceberg Code
from pyiceberg.catalog import load_catalog
...
```

## Technical Architecture

### Workspace Structure
Add new crates to the Cargo workspace:
```toml
[workspace]
members = [
    "pangolin_core",
    "pangolin_store",
    "pangolin_api",
    "pangolin_cli_admin", // NEW
    "pangolin_cli_user",  // NEW
    # Shared CLI logic crate?
    "pangolin_cli_common" // NEW: Shared auth, http client, formatting
]
```

### Key Libraries (Rust)
*   **`rustyline`**: For the interactive shell/REPL experience (history, auto-completion).
*   **`clap` (derive)**: For parsing commands within the loops (or custom parser).
*   **`reqwest`**: HTTP Client to talk to Pangolin API.
*   **`serde`, `serde_json`**: Serialization.
*   **`comfy-table`** or **`cli-table`**: For pretty-printing tabular data.
*   **`console`** / **`dialoguer`**: For password input and prompts.
*   **`syntect`**: For syntax highlighting code snippets in `pangolin-user`.

## Implementation Phases

### Phase 1: Foundation (`pangolin_cli_common`)
*   Define `PangolinClient` struct wrapper around `reqwest`.
*   Implement Authentication flow (Login -> JWT).
*   Implement REPL loop skeleton (Read input, Parse, Execute).

### Phase 2: `pangolin-admin` Skeleton
*   Setup crate.
*   Implement Login prompt.
*   Implement `list-tenants` and `list-users`.

### Phase 3: `pangolin-user` Skeleton
*   Setup crate.
*   Implement Login prompt.
*   Implement `list-catalogs` and `sample-code`.

### Phase 4: Feature Parity
*   Flesh out all commands defined in functionality sections.
