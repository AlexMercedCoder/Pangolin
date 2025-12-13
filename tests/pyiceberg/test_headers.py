#!/usr/bin/env python3
"""
Simple test to verify PyIceberg header propagation
"""

import requests

# Test 1: Direct request with header
print("Test 1: Direct HTTP request with X-Pangolin-Tenant header")
response = requests.get(
    "http://localhost:8080/v1/analytics/namespaces",
    headers={"X-Pangolin-Tenant": "00000000-0000-0000-0000-000000000001"}
)
print(f"Status: {response.status_code}")
print(f"Response: {response.text}\n")

# Test 2: PyIceberg with header
print("Test 2: PyIceberg with header.X-Pangolin-Tenant")
try:
    from pyiceberg.catalog import load_catalog
    
    catalog = load_catalog(
        "test",
        **{
            "uri": "http://localhost:8080",
            "prefix": "analytics",
            "header.X-Pangolin-Tenant": "00000000-0000-0000-0000-000000000001",
        }
    )
    
    # Try to list namespaces
    print(f"Catalog created: {catalog}")
    print("Attempting to list namespaces...")
    
    # Check what headers PyIceberg is actually sending
    import pyiceberg.catalog.rest
    session = catalog._session
    print(f"Session headers: {dict(session.headers)}")
    
    namespaces = catalog.list_namespaces()
    print(f"✓ Success! Namespaces: {namespaces}")
    
except Exception as e:
    print(f"✗ Failed: {e}")
    import traceback
    traceback.print_exc()
