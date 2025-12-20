# Pangolin Pre-compiled Binaries

This directory contains pre-compiled binaries for Pangolin API server and CLI tools across different platforms.

## Available Binaries

### ✅ Linux (x86_64)
- **pangolin_api** - REST API server (60MB)
- **pangolin-admin** - Admin CLI tool (9.8MB)
- **pangolin-user** - User CLI tool (10MB)

**Usage:**
```bash
# Make executable (if needed)
chmod +x linux-x86_64/*

# Run API server
./linux-x86_64/pangolin_api

# Use CLI tools
./linux-x86_64/pangolin-admin --help
./linux-x86_64/pangolin-user --help
```

### 🔧 macOS & Windows Binaries

**Note**: Cross-compiling to macOS and Windows from Linux requires specialized toolchains:
- **macOS**: Requires macOS SDK and osxcross toolchain
- **Windows**: Requires MinGW-w64 with system permissions

## Recommended Approach: GitHub Actions

The easiest way to build binaries for all platforms is using GitHub Actions, which provides native runners for each platform.

A workflow file is provided at [.github/workflows/build-binaries.yml](../../.github/workflows/build-binaries.yml) that will:
1. Build binaries on native runners (Ubuntu, macOS Intel, macOS ARM, Windows)
2. Upload artifacts for each platform
3. Create GitHub releases with all binaries attached

**To use:**
```bash
# Push a tag to trigger the workflow
git tag v0.1.0
git push origin v0.1.0

# Or manually trigger via GitHub Actions UI
```

## Manual Cross-Compilation (Advanced)

### macOS (requires macOS machine or osxcross)
```bash
# On a Mac:
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# On Linux with osxcross (complex setup):
# See: https://github.com/tpoechtrager/osxcross
```

### Windows (requires MinGW)
```bash
# Install MinGW
sudo apt-get install mingw-w64

# Add target
rustup target add x86_64-pc-windows-gnu

# Build
cargo build --release --target x86_64-pc-windows-gnu
```

## Environment Variables

All binaries respect the same environment variables documented in [docs/getting-started/env_vars.md](../../docs/getting-started/env_vars.md).

### Common Variables:
- `RUST_LOG` - Logging level (info, debug, warn, error)
- `PANGOLIN_STORAGE_TYPE` - Backend storage (memory, postgres, mongodb, sqlite)
- `DATABASE_URL` - Database connection string
- `PANGOLIN_NO_AUTH` - Enable no-auth mode for evaluation (true/false)
- `AWS_*` / `AZURE_*` / `GCP_*` - Cloud provider credentials

## Binary Sizes

The binaries are statically linked and include all dependencies:
- **API Server**: ~60MB (includes all storage backends, cloud SDKs, and Iceberg REST implementation)
- **CLI Tools**: ~10MB each (includes HTTP client and JSON processing)

## Security Notes

1. **Verify Checksums**: Always verify binary checksums before deployment
2. **Use HTTPS**: Run API server behind a reverse proxy with TLS in production
3. **Secrets Management**: Never hardcode credentials; use environment variables or secret managers
4. **Regular Updates**: Keep binaries updated to receive security patches

## Building from Source

To build binaries yourself on any platform:

```bash
cd pangolin/
cargo build --release --bin pangolin_api --bin pangolin-admin --bin pangolin-user

# Binaries will be in target/release/
```

## Docker Alternative

If you prefer containerized deployments, use our Docker images:
- `alexmerced/pangolin-api:0.1.0`
- `alexmerced/pangolin-cli:0.1.0`
- `alexmerced/pangolin-ui:0.1.0`

See [deployment_assets/demo/](../demo/) for Docker Compose examples.
