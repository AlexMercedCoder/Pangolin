#!/usr/bin/env python3
"""
Test PyIceberg with Bearer token authentication
"""

from pyiceberg.catalog import load_catalog
import jwt
import datetime

# Generate a simple JWT token with tenant_id
tenant_id = "00000000-0000-0000-0000-000000000001"
secret = "secret"  # Should match PANGOLIN_JWT_SECRET

# Create JWT token
payload = {
    "sub": "test-user",
    "tenant_id": tenant_id,
    "roles": ["User"],
    "exp": datetime.datetime.utcnow() + datetime.timedelta(hours=1)
}

token = jwt.encode(payload, secret, algorithm="HS256")
print(f"Generated token: {token}\n")

# Configure PyIceberg with token
catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080",
        "prefix": "analytics",
        "token": token,  # PyIceberg will use this in Authorization header
    }
)

print("Catalog created. Attempting to list namespaces...")
try:
    namespaces = catalog.list_namespaces()
    print(f"✓ Success! Namespaces: {namespaces}")
except Exception as e:
    print(f"✗ Failed: {e}")
    import traceback
    traceback.print_exc()
