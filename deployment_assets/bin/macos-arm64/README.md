# macOS Apple Silicon (ARM64) Binaries

To build binaries for macOS Apple Silicon:

```bash
rustup target add aarch64-apple-darwin
cd pangolin/
cargo build --release --target aarch64-apple-darwin --bin pangolin_api --bin pangolin-admin --bin pangolin-user
```

Binaries will be in `target/aarch64-apple-darwin/release/`

Copy them here for distribution.
