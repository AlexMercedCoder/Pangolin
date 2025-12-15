#!/bin/bash
# test_cli_live.sh
# Verifies that pangolin-admin and pangolin-user binaries can run and connect.

set -e

# Build everything first
echo "Building CLIs..."
cd pangolin
cargo build -p pangolin_cli_admin -p pangolin_cli_user

ADMIN_BIN="./target/debug/pangolin-admin"
USER_BIN="./target/debug/pangolin-user"

echo "------------------------------------------------"
echo "Testing Pangolin Admin CLI (Help)"
$ADMIN_BIN --help
echo "------------------------------------------------"

echo "------------------------------------------------"
echo "Testing Pangolin User CLI (Help)"
$USER_BIN --help
echo "------------------------------------------------"

# Note: Further tests require a running server.
# To test real connectivity:
# 1. Start server: cargo run -p pangolin_api
# 2. Run: $ADMIN_BIN --url http://localhost:8080 login --username admin
