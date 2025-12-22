#!/usr/bin/env python3
"""
Test script for new backend endpoint optimizations:
- Dashboard stats (role-based)
- Catalog summary
- Asset search
- Bulk delete
- Name validation
"""

import requests
import json

API_BASE = "http://localhost:8080"

def get_headers():
    return {
        "Content-Type": "application/json",
        "X-Pangolin-Tenant": "00000000-0000-0000-0000-000000000000"
    }

def test_dashboard_stats():
    """Test dashboard statistics endpoint"""
    print("\n=== Testing Dashboard Stats ===")
    
    resp = requests.get(f"{API_BASE}/api/v1/dashboard/stats", headers=get_headers())
    print(f"Status: {resp.status_code}")
    
    if resp.status_code == 200:
        data = resp.json()
        print(f"✓ Dashboard stats retrieved")
        print(f"  Scope: {data.get('scope')}")
        print(f"  Catalogs: {data.get('catalogs_count')}")
        print(f"  Warehouses: {data.get('warehouses_count')}")
        print(f"  Namespaces: {data.get('namespaces_count')}")
        print(f"  Tables: {data.get('tables_count')}")
        return True
    else:
        print(f"✗ Failed: {resp.text}")
        return False

def test_catalog_summary():
    """Test catalog summary endpoint"""
    print("\n=== Testing Catalog Summary ===")
    
    # First, ensure test_catalog exists
    catalog_name = "test_catalog"
    
    resp = requests.get(f"{API_BASE}/api/v1/catalogs/{catalog_name}/summary", headers=get_headers())
    print(f"Status: {resp.status_code}")
    
    if resp.status_code == 200:
        data = resp.json()
        print(f"✓ Catalog summary retrieved for '{catalog_name}'")
        print(f"  Namespaces: {data.get('namespace_count')}")
        print(f"  Tables: {data.get('table_count')}")
        print(f"  Branches: {data.get('branch_count')}")
        print(f"  Storage: {data.get('storage_location')}")
        return True
    else:
        print(f"✗ Failed: {resp.text}")
        return False

def test_asset_search():
    """Test asset search endpoint"""
    print("\n=== Testing Asset Search ===")
    
    # Search for tables with 'test' in the name
    params = {
        "q": "test",
        "limit": 10
    }
    
    resp = requests.get(f"{API_BASE}/api/v1/search/assets", headers=get_headers(), params=params)
    print(f"Status: {resp.status_code}")
    
    if resp.status_code == 200:
        data = resp.json()
        print(f"✓ Search completed")
        print(f"  Total results: {data.get('total')}")
        print(f"  Returned: {len(data.get('results', []))}")
        for result in data.get('results', [])[:3]:
            print(f"    - {result.get('catalog')}.{'.'.join(result.get('namespace', []))}.{result.get('name')}")
        return True
    else:
        print(f"✗ Failed: {resp.text}")
        return False

def test_name_validation():
    """Test name validation endpoint"""
    print("\n=== Testing Name Validation ===")
    
    payload = {
        "resource_type": "catalog",
        "names": ["test_catalog", "new_catalog_123", "another_catalog"]
    }
    
    resp = requests.post(f"{API_BASE}/api/v1/validate/names", headers=get_headers(), json=payload)
    print(f"Status: {resp.status_code}")
    
    if resp.status_code == 200:
        data = resp.json()
        print(f"✓ Validation completed")
        for result in data.get('results', []):
            status = "✓ Available" if result.get('available') else "✗ Taken"
            reason = f" ({result.get('reason')})" if result.get('reason') else ""
            print(f"  {result.get('name')}: {status}{reason}")
        return True
    else:
        print(f"✗ Failed: {resp.text}")
        return False

def test_bulk_delete():
    """Test bulk delete endpoint (without actually deleting)"""
    print("\n=== Testing Bulk Delete (Dry Run) ===")
    
    # Use fake UUIDs to test error handling
    payload = {
        "asset_ids": [
            "00000000-0000-0000-0000-000000000001",
            "invalid-uuid",
            "00000000-0000-0000-0000-000000000002"
        ]
    }
    
    resp = requests.post(f"{API_BASE}/api/v1/bulk/assets/delete", headers=get_headers(), json=payload)
    print(f"Status: {resp.status_code}")
    
    if resp.status_code == 200:
        data = resp.json()
        print(f"✓ Bulk delete processed")
        print(f"  Succeeded: {data.get('succeeded')}")
        print(f"  Failed: {data.get('failed')}")
        if data.get('errors'):
            print(f"  Errors:")
            for error in data.get('errors', [])[:3]:
                print(f"    - {error}")
        return True
    else:
        print(f"✗ Failed: {resp.text}")
        return False

def main():
    print("=" * 70)
    print("Backend Endpoint Optimizations Test Suite")
    print("=" * 70)
    
    results = []
    
    # Run all tests
    results.append(("Dashboard Stats", test_dashboard_stats()))
    results.append(("Catalog Summary", test_catalog_summary()))
    results.append(("Asset Search", test_asset_search()))
    results.append(("Name Validation", test_name_validation()))
    results.append(("Bulk Delete", test_bulk_delete()))
    
    # Summary
    print("\n" + "=" * 70)
    print("Test Summary")
    print("=" * 70)
    
    passed = sum(1 for _, result in results if result)
    total = len(results)
    
    for name, result in results:
        status = "✓ PASS" if result else "✗ FAIL"
        print(f"{status}: {name}")
    
    print(f"\nTotal: {passed}/{total} tests passed")
    print("=" * 70)
    
    return passed == total

if __name__ == "__main__":
    import sys
    sys.exit(0 if main() else 1)
