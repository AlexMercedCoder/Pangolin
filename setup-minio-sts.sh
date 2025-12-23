#!/bin/bash
# Setup MinIO for STS testing
# This script configures MinIO with IAM policies and users for AssumeRole testing

set -e

echo "🔧 Setting up MinIO for STS testing..."

# Configure MinIO client alias
echo "Configuring MinIO client..."
docker exec pangolin-minio mc alias set local http://localhost:9000 minioadmin minioadmin

# Create test bucket if it doesn't exist
echo "Creating test bucket..."
docker exec pangolin-minio mc mb local/test-bucket 2>/dev/null || echo "Bucket already exists"

# Create IAM user for STS
echo "Creating IAM user 'testuser'..."
docker exec pangolin-minio mc admin user add local testuser testpassword 2>/dev/null || echo "User may already exist"

# Create IAM policy for test bucket
echo "Creating IAM policy..."
docker exec pangolin-minio sh -c 'cat > /tmp/test-policy.json <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": ["s3:*"],
      "Resource": ["arn:aws:s3:::test-bucket/*", "arn:aws:s3:::test-bucket"]
    }
  ]
}
EOF'

# Add policy to MinIO
docker exec pangolin-minio mc admin policy create local test-policy /tmp/test-policy.json 2>/dev/null || echo "Policy may already exist"

# Attach policy to user
echo "Attaching policy to user..."
docker exec pangolin-minio mc admin policy attach local test-policy --user testuser

echo "✅ MinIO STS setup complete!"
echo ""
echo "Configuration:"
echo "  Endpoint: http://localhost:9000"
echo "  User: testuser"
echo "  Password: testpassword"
echo "  Bucket: test-bucket"
echo "  Policy: test-policy (allows s3:* on test-bucket)"
