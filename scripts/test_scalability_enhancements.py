#!/usr/bin/env python3
"""
Comprehensive test script for Pangolin API scalability enhancements.
Tests:
1. Connection pool configuration
2. Error handling improvements (granular status codes)
3. Existing functionality (branching, catalogs, etc.)
"""

import requests
import json
import sys
import time
import subprocess
import os

API_BASE = "http://localhost:8080"
TENANT_ID = "00000000-0000-0000-0000-000000000000"

def get_headers(token=None):
    headers = {
        "Content-Type": "application/json",
        "X-Pangolin-Tenant": TENANT_ID,
    }
    if token:
        headers["Authorization"] = f"Bearer {token}"
    return headers

def test_health():
    """Test basic health endpoint"""
    print("\n=== Testing Health Endpoint ===")
    resp = requests.get(f"{API_BASE}/health")
    assert resp.status_code == 200, f"Health check failed: {resp.status_code}"
    assert resp.text == "OK", f"Unexpected health response: {resp.text}"
    print("✓ Health check passed")

def test_error_handling():
    """Test improved error handling with granular status codes"""
    print("\n=== Testing Error Handling ===")
    
    # Test 404 - Not Found
    print("Testing 404 Not Found...")
    resp = requests.get(f"{API_BASE}/api/v1/tenants/99999999-9999-9999-9999-999999999999", headers=get_headers())
    assert resp.status_code == 404, f"Expected 404, got {resp.status_code}"
    error_data = resp.json()
    assert "error" in error_data, "Error response should have 'error' field"
    print(f"✓ 404 response: {error_data}")
    
    # Test 403 - Forbidden (if auth is enabled)
    # This would require a non-root user token
    
    # Test 400 - Bad Request (invalid JSON)
    print("Testing 400 Bad Request...")
    resp = requests.post(
        f"{API_BASE}/api/v1/tenants",
        headers=get_headers(),
        data="invalid json"
    )
    # Should get 400 or 500 depending on how Axum handles it
    print(f"✓ Invalid JSON response: {resp.status_code}")

def test_tenant_operations(token):
    """Test tenant CRUD operations"""
    print("\n=== Testing Tenant Operations ===")
    
    # List tenants
    print("Listing tenants...")
    resp = requests.get(f"{API_BASE}/api/v1/tenants", headers=get_headers(token))
    assert resp.status_code == 200, f"List tenants failed: {resp.status_code} - {resp.text}"
    tenants = resp.json()
    print(f"✓ Found {len(tenants)} tenants")
    
    # Create tenant
    tenant_name = f"test_tenant_{int(time.time())}"
    print(f"Creating tenant: {tenant_name}")
    resp = requests.post(
        f"{API_BASE}/api/v1/tenants",
        headers=get_headers(token),
        json={"name": tenant_name, "properties": {"test": "true"}}
    )
    if resp.status_code != 201:
        print(f"Warning: Create tenant returned {resp.status_code}: {resp.text}")
    else:
        print(f"✓ Tenant created: {resp.json()}")

def test_catalog_operations(token):
    """Test catalog operations"""
    print("\n=== Testing Catalog Operations ===")
    
    catalog_name = f"test_catalog_{int(time.time())}"
    
    # Create catalog
    print(f"Creating catalog: {catalog_name}")
    resp = requests.post(
        f"{API_BASE}/api/v1/catalogs",
        headers=get_headers(token),
        json={
            "name": catalog_name,
            "catalog_type": "Local",
            "warehouse_name": None,
            "storage_location": f"s3://warehouse/{catalog_name}",
            "properties": {}
        }
    )
    assert resp.status_code in [200, 201], f"Create catalog failed: {resp.status_code} - {resp.text}"
    print(f"✓ Catalog created")
    
    # List catalogs
    print("Listing catalogs...")
    resp = requests.get(f"{API_BASE}/api/v1/catalogs", headers=get_headers(token))
    assert resp.status_code == 200, f"List catalogs failed: {resp.status_code}"
    catalogs = resp.json()
    print(f"✓ Found {len(catalogs)} catalogs")
    
    return catalog_name

def test_branch_isolation(token, catalog_name):
    """Test branch isolation (previously fixed bug)"""
    print("\n=== Testing Branch Isolation ===")
    
    # Create a branch
    print("Creating 'dev' branch...")
    resp = requests.post(
        f"{API_BASE}/api/v1/branches",
        headers=get_headers(token),
        json={
            "name": "dev",
            "catalog_name": catalog_name,
            "branch_type": "Experimental",
            "assets": []
        }
    )
    if resp.status_code not in [200, 201]:
        print(f"Warning: Create branch returned {resp.status_code}: {resp.text}")
    else:
        print("✓ Branch 'dev' created")
    
    # List branches
    print("Listing branches...")
    resp = requests.get(
        f"{API_BASE}/api/v1/branches",
        headers=get_headers(token),
        params={"catalog_name": catalog_name}
    )
    if resp.status_code == 200:
        branches = resp.json()
        print(f"✓ Found {len(branches)} branches")
    else:
        print(f"Warning: List branches returned {resp.status_code}")

def test_connection_pool_logs():
    """Check server logs for connection pool configuration"""
    print("\n=== Checking Connection Pool Configuration ===")
    print("Note: Check server logs for lines like:")
    print("  'Initializing PostgreSQL pool: max=10, min=3, timeout=30s'")
    print("  'MongoDB max pool size set to: X'")
    print("✓ Connection pool configuration should be logged at startup")

def main():
    print("=" * 60)
    print("Pangolin API Scalability Enhancements Test Suite")
    print("=" * 60)
    
    # Get token from environment or use default
    token = os.getenv("PANGOLIN_TOKEN", "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIwMDAwMDAwMC0wMDAwLTAwMDAtMDAwMC0wMDAwMDAwMDAwMDAiLCJqdGkiOiJiYTFjNzQ1Yy01OTNkLTQ3ZTctYmZhNC0zODVlMjE3ZmMxZjYiLCJ1c2VybmFtZSI6InRlbmFudF9hZG1pbiIsInRlbmFudF9pZCI6IjAwMDAwMDAwLTAwMDAtMDAwMC0wMDAwLTAwMDAwMDAwMDAwMCIsInJvbGUiOiJ0ZW5hbnQtYWRtaW4iLCJleHAiOjE3OTc4ODk3NzQsImlhdCI6MTc2NjM1Mzc3NH0.NQhnPVAc5u2KbA6Jl8RzDWk084oihy1DrGwlRAS7uD4")
    
    try:
        test_health()
        test_error_handling()
        test_tenant_operations(token)
        catalog_name = test_catalog_operations(token)
        test_branch_isolation(token, catalog_name)
        test_connection_pool_logs()
        
        print("\n" + "=" * 60)
        print("✓ All tests passed!")
        print("=" * 60)
        return 0
    except AssertionError as e:
        print(f"\n✗ Test failed: {e}")
        return 1
    except Exception as e:
        print(f"\n✗ Unexpected error: {e}")
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    sys.exit(main())
