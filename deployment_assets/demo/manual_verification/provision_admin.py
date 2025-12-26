#!/usr/bin/env python3
"""
Provision admin user for testing.
"""

import requests
import sys

API_URL = "http://localhost:8080"

def provision_admin():
    """Create admin user via the API."""
    # In no-auth mode or with initial setup, we can create users directly
    # Let's try the tenant creation endpoint first
    
    # Create root tenant if needed
    try:
        response = requests.post(
            f"{API_URL}/api/v1/tenants",
            json={
                "name": "default",
                "properties": {}
            }
        )
        if response.status_code in [200, 201, 409]:  # 409 = already exists
            print("✓ Default tenant exists or created")
        else:
            print(f"Tenant creation status: {response.status_code}")
    except Exception as e:
        print(f"Note: {e}")
    
    # Create admin user
    try:
        response = requests.post(
            f"{API_URL}/api/v1/users",
            json={
                "username": "admin",
                "email": "admin@pangolin.local",
                "password": "admin",
                "tenant_id": "00000000-0000-0000-0000-000000000000",
                "role": "root"
            }
        )
        if response.status_code in [200, 201]:
            print("✓ Admin user created")
            return True
        elif response.status_code == 409:
            print("✓ Admin user already exists")
            return True
        else:
            print(f"User creation failed: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print(f"Error creating user: {e}")
        return False

if __name__ == "__main__":
    print("Provisioning admin user...")
    if provision_admin():
        print("\n✓ Provisioning complete!")
        sys.exit(0)
    else:
        print("\n✗ Provisioning failed")
        sys.exit(1)
