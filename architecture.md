# Pangolin Architecture

## Overview
Pangolin is a Rust-based, multi-tenant, branch-aware lakehouse catalog. It is designed to be compatible with the Apache Iceberg REST API while providing extended capabilities for branching, tagging, and managing non-Iceberg assets.

## Core Components

### 1. API Layer (`pangolin_api`)
- **Framework**: Axum
- **Responsibility**: Handles HTTP requests, routing, and serialization.
- **Modules**:
    - `iceberg_handlers`: Implements standard Iceberg REST endpoints.
    - `pangolin_handlers`: Implements extended Pangolin endpoints (Branching, Tenant Mgmt).
    - `middleware`: Handles authentication and tenant resolution.

### 2. Core Domain (`pangolin_core`)
- **Responsibility**: Defines the data models and business logic.
- **Key Models**:
    - `Tenant`: Represents a customer or organization.
    - `Catalog`: A collection of namespaces (e.g., `warehouse`).
    - `Namespace`: Logical grouping of assets.
    - `Asset`: Represents a table, view, or model.
    - `Branch`: A named reference to a commit history.
    - `Commit`: A record of changes to assets.

### 2. Storage Layer (`pangolin_store`)
- **Responsibility**: Abstraction over physical storage.
- **Components**:
    - `CatalogStore` Trait: Defines operations for managing tenants, namespaces, assets, and branches.
    - `MemoryStore`: In-memory implementation for testing and development.
    - `S3Store`: S3-backed implementation using `object_store`.
    - **Metadata IO**: Handles reading and writing of Iceberg metadata files (`metadata.json`).

### 3. API Layer (`pangolin_api`)
- **Framework**: Axum
- **Responsibility**: Handles HTTP requests, routing, and serialization.
- **Modules**:
    - `iceberg_handlers`: Implements standard Iceberg REST endpoints.
    - `pangolin_handlers`: Implements extended Pangolin endpoints (Branching, Tenant Mgmt).
    - `middleware`: Handles authentication and tenant resolution.

## Data Model

### Branching Model
Pangolin uses a Git-like branching model:
- **Main Branch**: The default branch for production data.
- **Ingest Branch**: For isolating write operations.
- **Experimental Branch**: For testing and experimentation.

### Asset Identification
Assets are identified by `Tenant -> Namespace -> Name`.
To support branching in standard Iceberg clients, we use a `table@branch` syntax in the table name.

## Request Flow
1. **Request**: Client sends HTTP request (e.g., `GET /v1/ns/tables/mytable@dev`).
2. **Middleware**: Resolves Tenant ID from Auth header.
3. **Handler**: Parses `mytable@dev` to extract table name `mytable` and branch `dev`.
4. **Store**: Queries the `CatalogStore` for the asset on the specified branch.
5. **Response**: Returns metadata or error.
