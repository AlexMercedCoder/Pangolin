# Project Organization

The Pangolin repository follows a monorepo structure separating the core Rust implementation, Management UI, Python SDK, and documentation.

## Directory Structure

```
pangolin-monorepo/
├── docs/                        # Comprehensive Documentation
│   ├── api/                     # API Reference and Swagger info
│   ├── cli/                     # CLI Command Reference
│   ├── features/                # Feature Guides (RBAC, Federation, etc.)
│   ├── getting-started/         # Installation and Architecture guides
│   ├── known-issues/            # Registry of quirks and temporary traps
│   └── ui/                      # UI User Guide
│
├── pangolin/                    # Core Rust Implementation (Workspace)
│   ├── pangolin_api/           # REST API Server (Axum)
│   ├── pangolin_core/          # Domain Models, Traits, and Logic
│   ├── pangolin_store/         # Storage Backends (Memory, SQLite, Postgres, Mongo)
│   ├── pangolin_cli_admin/     # Admin CLI Tool
│   ├── pangolin_cli_user/      # User CLI Tool
│   └── pangolin_cli_common/    # Shared CLI Logic
│
├── pangolin_ui/                 # Management UI (SvelteKit)
│   ├── src/routes/             # Application Routes
│   └── src/lib/                # Shared Components and Stores
│
├── pypangolin/                  # Python SDK
│   ├── pypangolin/             # Source Code
│   └── docs/                   # SDK-specific Documentation
│
├── scripts/                     # Automation & Verification
│   ├── verify_pypangolin_*.py  # SDK Verification Scripts
│   ├── test_release_*.py       # End-to-End Release Tests
│   └── docker-build.sh         # Build helpers
│
├── tests/                       # Integration Test Suites
│   └── pyiceberg/              # PyIceberg compatibility tests
│
├── planning/                    # Project Planning & Release Notes
│
├── website/                     # Landing Page (pangolincatalog.com)
└── deployment_assets/           # Kubernetes manifests & Helm charts
```

## Core Components

### 1. Backend (`pangolin/`)
The heart of the system, written in Rust. It utilizes a workspace architecture to separate the core logic (`pangolin_core`) from the API server (`pangolin_api`) and multiple storage backends (`pangolin_store`).

### 2. User Interface (`pangolin_ui/`)
A generic, multi-tenant SvelteKit application providing a visual interface for managing catalogs, namespaces, tables, and access control. It usually runs on port `3000`.

### 3. Python SDK (`pypangolin/`)
A Pydantic-based client library for automating Pangolin operations, including Service User management request automation and credential handling.

### 4. Admin & User CLIs (`pangolin/pangolin_cli_*`)
Rust-based command-line interfaces for both system administrators (Catalog/User management) and end-users (Branching/Tagging).

## Documentation Strategy

Documentation is centralized in `docs/` but specific component implementation details may reside closer to the code (`pypangolin/README.md`).

- **User Guides**: `docs/getting-started/`
- **Architecture**: `docs/architecture/`
- **API Specs**: `docs/api/`

## Key Configuration Files
- `Cargo.toml` (Root workspace config)
- `.env` (Environment variables for local dev)
- `docker-compose.yml` (Full stack orchestration)
