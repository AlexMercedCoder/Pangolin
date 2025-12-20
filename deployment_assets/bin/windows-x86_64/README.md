# Windows (x86_64) Binaries

To build binaries for Windows:

```bash
# Install Windows target
rustup target add x86_64-pc-windows-gnu

# Install MinGW (on Ubuntu/Debian)
sudo apt-get install mingw-w64

# Build
cd pangolin/
cargo build --release --target x86_64-pc-windows-gnu --bin pangolin_api --bin pangolin-admin --bin pangolin-user
```

Binaries will be in `target/x86_64-pc-windows-gnu/release/` with `.exe` extensions.

Copy them here for distribution.

## Note
Windows binaries may require additional DLL files. Consider using Docker for Windows deployments as an alternative.
