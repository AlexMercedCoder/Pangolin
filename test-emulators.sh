#!/bin/bash
# Live testing script for credential vending with cloud emulators

set -e

echo "🧪 Live Testing: Credential Vending with Cloud Emulators"
echo "=========================================================="

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if emulators are running
echo -e "\n${YELLOW}Checking emulator status...${NC}"

# Check MinIO
if curl -s http://localhost:9000/minio/health/live > /dev/null 2>&1; then
    echo -e "${GREEN}✓ MinIO is running${NC}"
else
    echo -e "${RED}✗ MinIO is not running. Start with: docker-compose -f docker-compose.emulators.yml up -d${NC}"
    exit 1
fi

# Check Azurite
if nc -z localhost 10000 > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Azurite is running${NC}"
else
    echo -e "${RED}✗ Azurite is not running${NC}"
    exit 1
fi

# Check fake-gcs-server
if curl -s http://localhost:4443/storage/v1/b > /dev/null 2>&1; then
    echo -e "${GREEN}✓ fake-gcs-server is running${NC}"
else
    echo -e "${RED}✗ fake-gcs-server is not running${NC}"
    exit 1
fi

echo -e "\n${YELLOW}Setting up test buckets/containers...${NC}"

# Setup MinIO bucket
echo "Creating MinIO bucket..."
docker exec pangolin-minio mc alias set local http://localhost:9000 minioadmin minioadmin 2>/dev/null || true
docker exec pangolin-minio mc mb local/test-bucket 2>/dev/null || echo "Bucket already exists"
echo -e "${GREEN}✓ MinIO bucket created${NC}"

# Setup Azurite container
echo "Creating Azurite container..."
curl -X PUT "http://localhost:10000/devstoreaccount1/test-container?restype=container" \
  -H "x-ms-version: 2021-08-06" \
  -H "x-ms-date: $(date -u '+%a, %d %b %Y %H:%M:%S GMT')" 2>/dev/null || echo "Container may already exist"
echo -e "${GREEN}✓ Azurite container created${NC}"

# Setup fake-gcs bucket
echo "Creating GCS bucket..."
curl -X POST "http://localhost:4443/storage/v1/b?project=test-project" \
  -H "Content-Type: application/json" \
  -d '{"name": "test-bucket"}' 2>/dev/null || echo "Bucket may already exist"
echo -e "${GREEN}✓ GCS bucket created${NC}"

echo -e "\n${YELLOW}Running live credential vending tests...${NC}"

# Run the live tests
cd pangolin_api
cargo test --test live_credential_vending -- --nocapture

echo -e "\n${GREEN}✅ Live testing complete!${NC}"
