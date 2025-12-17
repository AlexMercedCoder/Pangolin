# Pangolin Release Plan (v0.1.0)

## 1. Code Audit & Quality Assurance

### Priority Blockers
- [ ] **Fix Federated Catalog Deletion**: `pangolin_api/src/federated_catalog_handlers.rs` claims to delete but does nothing. Since `CatalogStore::delete_catalog` exists, wire it up.
- [ ] **Verify MemoryStore Parity**: Ensure `MemoryStore` supports all methods needed for E2E tests (Mocking store for `pangolin_api` tests).
- [ ] **Security Audits**:
    - `federated_catalog_handlers.rs`: `unwrap()` on `federated_config` (Low risk but should be proper error).
    - `permission_handlers.rs`: TODO about "Strict permission check" needs validation or removal if covered by other layers.
    - `pangolin_handlers.rs`: TODO "Get user from auth context" - Ensure system actor is only used where appropriate.

### Technical Debt Cleanup
- [ ] **Remove legacy TODOs**: Scan for TODOs that are already done and remove comments.
- [ ] **Standardize Error Messages**: Ensure all API errors follow `{"error": "message"}` format.
- [ ] **Remove `dead_code` warnings**: Clean up unused imports and variables in `pangolin_api`.

## 2. Build & Packaging Strategy

### Binaries (CLI)
Target Platforms:
- `x86_64-unknown-linux-gnu` (Linux)
- `aarch64-apple-darwin` (macOS Apple Silicon)
- `x86_64-pc-windows-msvc` (Windows)

**Action Items**:
- [ ] Create `cross-compile` Github Action (using `cross-rs`).
- [ ] Release artifacts: `pangolin-admin`, `pangolin-user` (renamed from `pangolin_cli_admin`).

### Docker Images (Backend & UI)
**backend**: `alexmerced/pangolin-api:latest`
- Base: `debian:bookworm-slim`
- Entrypoint: `/usr/local/bin/pangolin_api`
- Environment Variables: `PANGOLIN_STORE_TYPE`, `DATABASE_URL`, `RUST_LOG`.

**frontend**: `alexmerced/pangolin-ui:latest`
- Base: `nginx:alpine` (Build SvelteKit as static adapter or use Node adapter)
- *Recommendation*: Use Node adapter for SSR and dynamic environment variables (`VITE_API_URL`).
- Base: `node:20-alpine`

**Action Items**:
- [x] Create `Dockerfile.backend` in `pangolin_api` (Consolidated to `pangolin/Dockerfile`).
- [x] Create `Dockerfile.ui` in `pangolin_ui`.
- [x] Create `docker-compose.yml` for local full-stack run (Postgres + Pangolin API + UI).
- [x] Verified full stack startup.

## 3. Infrastructure & Deployment

### Helm Chart
Create a Helm chart `charts/pangolin` with:
- Deployment: `pangolin-api` (Scale: 2 replicas).
- Service: `pangolin-api-service` (ClusterIP).
- Ingress: `pangolin-ingress` (Route `/api` to backend, `/` to frontend).
- ConfigMap: Environmental config.
- Secret: Database credentials.

### Terraform (Optional Phase 2)
- AWS RDS (Postgres).
- AWS ECS/EKS for hosting.

## 4. Documentation

### User Guides
- **Installation**: Docker Compose, Helm, Binary.
- **Configuration**: Env vars reference.
- **CLI Reference**: Auto-generated markdown from clap?
- **UI Walkthrough**: Screenshots of RBAC, Discovery, Explorer.

### API Reference
- **OpenAPI Spec**: Generate swagger/openapi.json from Axum (using `utoipa` or similar if annotated, otherwise manual/semi-manual).

## 5. CI/CD Pipeline (GitHub Actions)
- **PR Triggers**: `cargo test`, `npm test` (UI).
- **Main Merge**:
    - Semantic Versioning check.
  - [x] Build Docker Images -> Push to DockerHub (v0.1.0 Released).
- [ ] Build Binaries -> Create GitHub Release.
