#!/bin/bash
set -e

# Point to the API
export PANGOLIN_URL="http://localhost:8081"
# No auth mode uses "tenant_admin" / "password123" by default if not strictly checked, 
# but PANGOLIN_NO_AUTH allows bypass or implicit admin.
# CLI client might need login.
# In "no auth" mode, any login might work or we can skip login?
# The CLI handle_login resets context.
# Let's try to login first.

echo "1. Logging in..."
# We use expect or just input piping? CLI uses `dialoguer::Input`.
# `dialoguer` reads from stdin.
# We can use `cargo run -- login --username tenant_admin --password password123` if arguments are supported.
# Looking at `commands.rs`, `Login` command has arguments!
#     Login {
#         #[arg(short, long)]
#         username: Option<String>,
#         #[arg(short, long)]
#         password: Option<String>,
#     },

./target/debug/pangolin-admin login --username tenant_admin --password password123

echo "2. Listing Federated Catalogs (should be empty or contain previous test one if persistent)..."
./target/debug/pangolin-admin list-federated-catalogs

echo "3. Creating Federated Catalog with properties..."
# -P uri=... -P token=...
./target/debug/pangolin-admin create-federated-catalog \
    "cli_federated_catalog" \
    --storage-location "s3://bucket/path" \
    -P uri=http://cli-external:9090 \
    -P token=cli-token-abc \
    -P region=us-west-2

echo "4. Verifying creation..."
./target/debug/pangolin-admin list-federated-catalogs

echo "5. Testing connection (mock)..."
# This might fail networking but check structure
./target/debug/pangolin-admin test-federated-catalog "cli_federated_catalog" || true

echo "✅ CLI Test Complete"
