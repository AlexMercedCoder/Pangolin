#!/usr/bin/env python3
"""
Test script to verify PostgreSQL backend parity implementation in NO-AUTH mode.
Tests Service Users, System Settings, and Audit Logs endpoints.
"""

import requests
import json
from typing import Dict, Any

API_URL = "http://localhost:8080"

def test_service_users():
    """Test Service User endpoints."""
    print("\n=== Testing Service Users ===")
    
    # 1. List service users (should be empty initially)
    print("1. Listing service users...")
    response = requests.get(f"{API_URL}/api/v1/service-users")
    print(f"   Status: {response.status_code}")
    if response.status_code == 200:
        print(f"   Service users: {response.json()}")
    else:
        print(f"   Error: {response.text}")
        return False
    
    # 2. Create a service user
    print("\n2. Creating service user...")
    response = requests.post(
        f"{API_URL}/api/v1/service-users",
        json={
            "name": "test-service-user",
            "description": "Test service user for backend parity",
            "role": "tenant-user",
            "expires_in_days": 30
        }
    )
    print(f"   Status: {response.status_code}")
    if response.status_code == 201:
        service_user_data = response.json()
        print(f"   Created: {json.dumps(service_user_data, indent=2)}")
        service_user_id = service_user_data["service_user_id"]
        api_key = service_user_data["api_key"]
        
        # 3. Get service user by ID
        print(f"\n3. Getting service user by ID: {service_user_id}...")
        response = requests.get(f"{API_URL}/api/v1/service-users/{service_user_id}")
        print(f"   Status: {response.status_code}")
        if response.status_code == 200:
            print(f"   Service user: {json.dumps(response.json(), indent=2)}")
        
        # 4. Update service user
        print(f"\n4. Updating service user...")
        response = requests.put(
            f"{API_URL}/api/v1/service-users/{service_user_id}",
            json={
                "description": "Updated description",
                "active": True
            }
        )
        print(f"   Status: {response.status_code}")
        if response.status_code == 200:
            print(f"   Updated: {response.json()}")
        
        # 5. List service users again
        print("\n5. Listing service users again...")
        response = requests.get(f"{API_URL}/api/v1/service-users")
        print(f"   Status: {response.status_code}")
        if response.status_code == 200:
            print(f"   Service users: {json.dumps(response.json(), indent=2)}")
        
        # 6. Delete service user
        print(f"\n6. Deleting service user...")
        response = requests.delete(f"{API_URL}/api/v1/service-users/{service_user_id}")
        print(f"   Status: {response.status_code}")
        if response.status_code == 200:
            print(f"   Deleted: {response.json()}")
            return True
    else:
        print(f"   Error: {response.text}")
        return False

def test_system_settings():
    """Test System Settings endpoints."""
    print("\n=== Testing System Settings ===")
    
    # 1. Get system settings
    print("1. Getting system settings...")
    response = requests.get(f"{API_URL}/api/v1/config/settings")
    print(f"   Status: {response.status_code}")
    if response.status_code == 200:
        print(f"   Settings: {json.dumps(response.json(), indent=2)}")
    else:
        print(f"   Error: {response.text}")
        return False
    
    # 2. Update system settings
    print("\n2. Updating system settings...")
    response = requests.put(
        f"{API_URL}/api/v1/config/settings",
        json={
            "allow_public_signup": False,
            "default_warehouse_bucket": "test-bucket",
            "default_retention_days": 90
        }
    )
    print(f"   Status: {response.status_code}")
    if response.status_code == 200:
        print(f"   Updated: {json.dumps(response.json(), indent=2)}")
    else:
        print(f"   Error: {response.text}")
        return False
    
    # 3. Get system settings again
    print("\n3. Getting system settings again...")
    response = requests.get(f"{API_URL}/api/v1/config/settings")
    print(f"   Status: {response.status_code}")
    if response.status_code == 200:
        print(f"   Settings: {json.dumps(response.json(), indent=2)}")
        return True
    return False

def test_audit_logs():
    """Test Audit Logs endpoints."""
    print("\n=== Testing Audit Logs ===")
    
    # 1. List audit logs
    print("1. Listing audit logs...")
    response = requests.get(f"{API_URL}/api/v1/audit/logs")
    print(f"   Status: {response.status_code}")
    if response.status_code == 200:
        logs = response.json()
        print(f"   Found {len(logs)} audit log entries")
        if logs:
            print(f"   Latest entry: {json.dumps(logs[0], indent=2)}")
        return True
    else:
        print(f"   Error: {response.text}")
        return False

def main():
    print("=" * 60)
    print("PostgreSQL Backend Parity Verification Test (NO-AUTH)")
    print("=" * 60)
    
    results = {
        "Service Users": False,
        "System Settings": False,
        "Audit Logs": False
    }
    
    try:
        # Run tests
        results["Service Users"] = test_service_users()
        results["System Settings"] = test_system_settings()
        results["Audit Logs"] = test_audit_logs()
        
        print("\n" + "=" * 60)
        print("Test Results:")
        print("=" * 60)
        for test_name, passed in results.items():
            status = "✓ PASSED" if passed else "✗ FAILED"
            print(f"{test_name}: {status}")
        
        if all(results.values()):
            print("\n✓ All tests completed successfully!")
            print("=" * 60)
            return 0
        else:
            print("\n✗ Some tests failed")
            print("=" * 60)
            return 1
        
    except requests.exceptions.RequestException as e:
        print(f"\n✗ Test failed with error: {e}")
        if hasattr(e, 'response') and e.response is not None:
            print(f"   Response: {e.response.text}")
        return 1
    except Exception as e:
        print(f"\n✗ Unexpected error: {e}")
        return 1

if __name__ == "__main__":
    exit(main())
