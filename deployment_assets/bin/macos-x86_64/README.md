# macOS Intel (x86_64) Binaries

To build binaries for macOS Intel:

```bash
rustup target add x86_64-apple-darwin
cd pangolin/
cargo build --release --target x86_64-apple-darwin --bin pangolin_api --bin pangolin-admin --bin pangolin-user
```

Binaries will be in `target/x86_64-apple-darwin/release/`

Copy them here for distribution.
