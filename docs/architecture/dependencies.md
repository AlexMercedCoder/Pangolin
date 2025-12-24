# Project Dependencies & Tech Stack

## Overview

Pangolin is built as a high-performance Rust backend (`pangolin_api`) paired with a modern SvelteKit frontend (`pangolin_ui`). This document outlines the key libraries and tools that power the system.

---

## 🦀 Backend (Rust)

The backend is organized as a Cargo workspace with multiple crates.

### Core Frameworks
| Dependency | Version | Purpose |
| :--- | :--- | :--- |
| **[axum](https://github.com/tokio-rs/axum)** | `0.7` | High-performance, ergonomic async web framework. Handles HTTP routing, extractors, and middleware. |
| **[tokio](https://tokio.rs/)** | `1.0` | Asynchronous runtime. Provides the event loop, async TCP, and task scheduling. |
| **[tower](https://github.com/tower-rs/tower)** | `0.4` | Modular service primitives. Used for middleware (logging, CORS, timeout) composition. |

### Data & Serialization
| Dependency | Version | Purpose |
| :--- | :--- | :--- |
| **[serde](https://serde.rs/)** | `1.0` | Serialization/Deserialization framework. Used everywhere for JSON processing. |
| **[serde_json](https://github.com/serde-rs/json)** | `1.0` | JSON support for Serde. |
| **[uuid](https://github.com/uuid-rs/uuid)** | `1.0` | Generation and parsing of UUIDs (standard for IDs in Pangolin). |
| **[chrono](https://github.com/chronotope/chrono)** | `0.4` | Date and Time handling. |
| **[sqlx](https://github.com/launchbadge/sqlx)** | `0.7` | (In `pangolin_store`) Async SQL toolkit. Used for PostgreSQL and SQLite interaction. |
| **[mongodb](https://github.com/mongodb/mongo-rust-driver)** | `2.8` | (In `pangolin_store`) Official async driver for MongoDB. |

### Storage & Cloud
| Dependency | Version | Purpose |
| :--- | :--- | :--- |
| **[object_store](https://github.com/apache/arrow-rs/tree/master/object_store)** | `0.11` | Unified interface for object storage. Handles interactions with AWS S3, Azure Blob, GCS, and local files. |

### Utility & Logging
| Dependency | Version | Purpose |
| :--- | :--- | :--- |
| **[tracing](https://github.com/tokio-rs/tracing)** | `0.1` | Instrumentation and structural logging. |
| **[thiserror](https://github.com/dtolnay/thiserror)** | `1.0` | Ergonomic custom error type derivation. |
| **[anyhow](https://github.com/dtolnay/anyhow)** | `1.0` | Flexible error handling for application code. |
| **[dashmap](https://github.com/xacrimon/dashmap)** | `6.0` | Concurrent HashMap. Used in `MemoryStore` for thread-safe in-memory state. |
| **[dotenvy](https://github.com/allan2/dotenvy)** | `0.15` | Loads environment variables from `.env` files. |
| **[clap](https://github.com/clap-rs/clap)** | `4.4` | (In CLIs) Command Line Argument Parser. Powers `pangolin-admin` and `pangolin-user`. |

---

## 🎨 Frontend (SvelteKit)

The management UI (`pangolin_ui`) is a Single Page Application (SPA) built with SvelteKit.

### Core Frameworks
| Dependency | Purpose |
| :--- | :--- |
| **[SvelteKit](https://kit.svelte.dev/)** | The meta-framework. Handles routing, SSR/CSR, and build optimization. |
| **[Svelte 5](https://svelte.dev/)** | The UI component framework. |
| **[Vite](https://vitejs.dev/)** | Next-generation frontend tooling. Powers the dev server and build process. |

### Styling & Components
| Dependency | Purpose |
| :--- | :--- |
| **[TailwindCSS](https://tailwindcss.com/)** | Utility-first CSS framework for layout and spacing. |
| **[SMUI](https://github.com/hperrin/smui)** | Svelte Material UI. implementations of Material Design components (Buttons, Inputs, Tables). |
| **[Material Icons](https://fonts.google.com/icons)** | Icon set used throughout the UI. |

### Utilities
| Dependency | Purpose |
| :--- | :--- |
| **[marked](https://github.com/markedjs/marked)** | Markdown parser. Used for rendering descriptions and documentation. |
| **[TypeScript](https://www.typescriptlang.org/)** | Static typing for JavaScript. |

---

## 🛠️ Dev Tools & Testing

| Tool | Purpose |
| :--- | :--- |
| **Docker & Docker Compose** | Containerization for consistent development and deployment environments. |
| **MinIO** | S3-compatible object storage for local testing. |
| **PyIceberg** | Python client for Apache Iceberg. Used for end-to-end integration testing. |
| **Vitest** | Unit testing framework for the generic frontend logic. |
| **Playwright** | End-to-end testing for the frontend. |
