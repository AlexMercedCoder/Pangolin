# Regenerating OpenAPI Documentation

This guide explains how to regenerate the OpenAPI specification files after making changes to API handlers or models.

## Overview

The Pangolin API uses [utoipa](https://github.com/juhaku/utoipa) to automatically generate OpenAPI 3.0 specifications from Rust code annotations. The OpenAPI spec is available in two formats:

- **JSON**: `docs/api/openapi.json` (4734 lines)
- **YAML**: `docs/api/openapi.yaml` (3051 lines)

## When to Regenerate

Regenerate the OpenAPI documentation when you:

- Add new API endpoints
- Modify existing endpoint parameters or responses
- Add or update request/response models
- Change API descriptions or metadata
- Update security schemes

## Prerequisites

Ensure you have:
- Rust toolchain installed
- All dependencies up to date (`cargo update`)
- The `pangolin_api` crate compiles successfully

## Regeneration Commands

### Generate JSON Format

```bash
cd pangolin
cargo run -p pangolin_api --bin export_openapi json 2>/dev/null > ../docs/api/openapi.json
```

**Or** from the project root:

```bash
cd pangolin && cargo run -p pangolin_api --bin export_openapi json 2>/dev/null > ../docs/api/openapi.json
```

### Generate YAML Format

```bash
cd pangolin
cargo run -p pangolin_api --bin export_openapi yaml 2>/dev/null > ../docs/api/openapi.yaml
```

**Or** from the project root:

```bash
cd pangolin && cargo run -p pangolin_api --bin export_openapi yaml 2>/dev/null > ../docs/api/openapi.yaml
```

### Generate Both Formats

```bash
cd pangolin

# Generate JSON
cargo run -p pangolin_api --bin export_openapi json 2>/dev/null > ../docs/api/openapi.json

# Generate YAML
cargo run -p pangolin_api --bin export_openapi yaml 2>/dev/null > ../docs/api/openapi.yaml
```

## Verification

After regeneration, verify the output:

### Check File Sizes

```bash
wc -l docs/api/openapi.json docs/api/openapi.yaml
```

Expected output (approximate):
```
 4734 docs/api/openapi.json
 3051 docs/api/openapi.yaml
 7785 total
```

### Validate JSON

```bash
cat docs/api/openapi.json | jq . > /dev/null && echo "✓ Valid JSON"
```

### View YAML Header

```bash
head -20 docs/api/openapi.yaml
```

Should show:
```yaml
openapi: 3.0.3
info:
  title: Pangolin Catalog API
  description: Multi-tenant Apache Iceberg REST Catalog...
  ...
```

## Testing the Swagger UI

After regeneration, test the Swagger UI to ensure all endpoints appear correctly:

1. **Start the API server**:
   ```bash
   cd pangolin
   cargo run -p pangolin_api
   ```

2. **Open Swagger UI** in your browser:
   ```
   http://localhost:8080/swagger-ui
   ```

3. **Verify**:
   - All 13 API tags are present
   - All 67 endpoints are listed
   - Schemas are properly rendered
   - "Try it out" functionality works

## Troubleshooting

### Issue: Empty or Invalid Output

**Symptom**: Generated file is empty or contains compilation errors

**Solution**: Check that stderr is redirected properly:
```bash
# Correct (redirects stderr to /dev/null)
cargo run -p pangolin_api --bin export_openapi json 2>/dev/null > ../docs/api/openapi.json

# Incorrect (stderr mixed with stdout)
cargo run -p pangolin_api --bin export_openapi json > ../docs/api/openapi.json
```

### Issue: Compilation Errors

**Symptom**: Export command fails with compilation errors

**Solution**:
1. Ensure all code compiles:
   ```bash
   cargo check -p pangolin_api
   ```

2. Fix any compilation errors before regenerating

3. Ensure all models referenced in handlers have `#[derive(ToSchema)]`

### Issue: Missing Endpoints

**Symptom**: Some endpoints don't appear in the generated spec

**Solution**:
1. Verify the handler has `#[utoipa::path(...)]` annotation
2. Ensure the handler is imported in `openapi.rs`:
   ```rust
   use crate::your_handlers::*;
   ```
3. Add the handler function name to the `paths(...)` list in `openapi.rs`

### Issue: Missing Schemas

**Symptom**: Request/response types show as empty objects

**Solution**:
1. Add `#[derive(ToSchema)]` to the struct:
   ```rust
   use utoipa::ToSchema;
   
   #[derive(Serialize, Deserialize, ToSchema)]
   pub struct YourStruct {
       // fields...
   }
   ```

2. Import the type in `openapi.rs`:
   ```rust
   use crate::your_module::YourStruct;
   ```

3. Add it to the `components(schemas(...))` list in `openapi.rs`

## Adding New Endpoints

When adding a new API endpoint, follow these steps:

### 1. Annotate the Handler

Add `#[utoipa::path(...)]` above your handler function:

```rust
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct CreateItemRequest {
    pub name: String,
}

#[derive(Serialize, ToSchema)]
pub struct ItemResponse {
    pub id: Uuid,
    pub name: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/items",
    tag = "Items",
    request_body = CreateItemRequest,
    responses(
        (status = 201, description = "Item created", body = ItemResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_item(
    State(store): State<AppState>,
    Json(payload): Json<CreateItemRequest>,
) -> impl IntoResponse {
    // implementation...
}
```

### 2. Update openapi.rs

Add imports:
```rust
use crate::item_handlers::*;
```

Add to paths:
```rust
#[openapi(
    paths(
        // ... existing paths ...
        create_item,
        list_items,
        // etc.
    ),
```

Add schemas:
```rust
components(
    schemas(
        // ... existing schemas ...
        CreateItemRequest,
        ItemResponse,
    )
)
```

Add tag (if new category):
```rust
tags(
    // ... existing tags ...
    (name = "Items", description = "Item management endpoints"),
)
```

### 3. Regenerate

```bash
cd pangolin
cargo run -p pangolin_api --bin export_openapi json 2>/dev/null > ../docs/api/openapi.json
cargo run -p pangolin_api --bin export_openapi yaml 2>/dev/null > ../docs/api/openapi.yaml
```

### 4. Verify

Check Swagger UI to ensure your new endpoint appears correctly.

## Export Utility Source

The export utility is located at:
```
pangolin/pangolin_api/src/bin/export_openapi.rs
```

It supports both JSON and YAML formats via command-line argument:
- `cargo run --bin export_openapi json` - Outputs JSON
- `cargo run --bin export_openapi yaml` - Outputs YAML

## Current API Statistics

- **Total Handlers**: 67
- **API Tags**: 13
- **Core Models**: 35+
- **OpenAPI Version**: 3.0.3
- **JSON Size**: ~4734 lines
- **YAML Size**: ~3051 lines

## Related Documentation

- [API Documentation](../api/README.md)
- [Authentication Guide](../api/authentication.md)
- [utoipa Documentation](https://docs.rs/utoipa/latest/utoipa/)
- [OpenAPI 3.0 Specification](https://swagger.io/specification/)

## Automation

Consider adding these commands to your CI/CD pipeline or Makefile:

```makefile
# Makefile
.PHONY: openapi-json openapi-yaml openapi-all

openapi-json:
	cd pangolin && cargo run -p pangolin_api --bin export_openapi json 2>/dev/null > ../docs/api/openapi.json

openapi-yaml:
	cd pangolin && cargo run -p pangolin_api --bin export_openapi yaml 2>/dev/null > ../docs/api/openapi.yaml

openapi-all: openapi-json openapi-yaml
	@echo "✓ OpenAPI documentation regenerated"
```

Then simply run:
```bash
make openapi-all
```
