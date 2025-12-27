#!/bin/bash
set -e

echo "Starting Sequential Docker Build..."

echo "--- Building API (1/3) ---"
docker buildx build --platform linux/amd64,linux/arm64 -t alexmerced/pangolin-api:latest -t alexmerced/pangolin-api:0.3.0 --push -f pangolin/Dockerfile pangolin

echo "--- Building CLI (2/3) ---"
docker buildx build --platform linux/amd64,linux/arm64 -t alexmerced/pangolin-cli:latest -t alexmerced/pangolin-cli:0.3.0 --push -f pangolin/Dockerfile.tools pangolin

echo "--- Building UI (3/3) ---"
docker buildx build --platform linux/amd64,linux/arm64 -t alexmerced/pangolin-ui:latest -t alexmerced/pangolin-ui:0.3.0 --push -f pangolin_ui/Dockerfile pangolin_ui

echo "All builds completed successfully."
