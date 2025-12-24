# Architecture Documentation

This directory contains detailed technical documentation for the Pangolin architecture.

## Core Components
- **[Models](./models.md)**: Details the core data structures (Structs), including Tenant, Warehouse, and Asset definitions.
- **[Enums](./enums.md)**: Exhaustive list of system enumerations like `AssetType`, `CatalogType`, and `VendingStrategy`.
- **[Traits](./traits.md)**: Deep dive into the `CatalogStore` and `Signer` traits which define the storage and security interfaces.
- **[Handlers](./handlers.md)**: Breakdown of the API handler modules and their functional domains.

## Logic & Patterns
- **[Branching](./branching.md)**: Explanation of the "Git-for-Data" model, including branching, committing, and 3-way merge logic.
- **[Caching](./caching.md)**: Details on the multi-layered caching strategy using `moka` (Metadata) and `DashMap` (ObjectStore).
- **[Dependencies](./dependencies.md)**: Comprehensive list of libraries and frameworks used in Backend and Frontend.

## Legacy Docs
- `catalog-store-trait.md`: (Ref `traits.md`)
- `signer-trait.md`: (Ref `traits.md`)
