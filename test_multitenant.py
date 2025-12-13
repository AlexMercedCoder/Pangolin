import requests
import uuid
import time
import sys

BASE_URL = "http://localhost:8080"
ROOT_USER = "admin"
ROOT_PASS = "admin123"

def get_root_auth():
    return (ROOT_USER, ROOT_PASS)

def create_tenant(name):
    print(f"Creating tenant: {name}")
    resp = requests.post(f"{BASE_URL}/api/v1/tenants", json={"name": name}, auth=get_root_auth())
    resp.raise_for_status()
    return resp.json()

def create_user(tenant_id, username, password, role="tenant-admin"):
    print(f"Creating user {username} in tenant {tenant_id}")
    resp = requests.post(
        f"{BASE_URL}/api/v1/users", 
        json={
            "username": username,
            "email": f"{username}@example.com",
            "password": password,
            "tenant-id": tenant_id,
            "role": role
        },
        auth=get_root_auth()
    )
    if resp.status_code != 201:
        print(f"Failed to create user: {resp.text}")
    resp.raise_for_status()
    return resp.json()

def login(username, password):
    print(f"Logging in as {username}")
    resp = requests.post(f"{BASE_URL}/api/v1/users/login", json={"username": username, "password": password})
    resp.raise_for_status()
    return resp.json()["token"]

def create_catalog(tenant_id, name):
    print(f"Creating catalog {name} for tenant {tenant_id}")
    resp = requests.post(
        f"{BASE_URL}/api/v1/catalogs",
        json={"name": name, "type": "memory", "warehouse_name": None},
        headers={"X-Pangolin-Tenant": tenant_id}, # Root needs to specify tenant context? 
        # Actually, store.create_catalog takes tenant_id. 
        # But the handler extracts tenant_id from the auth context. 
        # If we act as ROOT, we have Root authority, but do we have a tenant context?
        # RootUser is global. But listing/creating often requires a tenant scope.
        # Let's see if we can create catalogs as the Tenant Admin instead.
        auth=get_root_auth()
    )
    # If 500, it might be because Root doesn't have a specific tenant ID inferred.
    # We might need to handle catalog creation differently or just use the default catalog if it exists per tenant?
    # Our system creates a default catalog? No.
    if resp.status_code == 201:
         return resp.json()
    print(f"Warning: Failed to create catalog as root: {resp.status_code} {resp.text}")
    return None

def main():
    print("Starting Multi-Tenant Isolation Test")
    
    # 1. Create Tenants
    try:
        tenant_a = create_tenant(f"tenant_a_{uuid.uuid4().hex[:8]}")
        tenant_b = create_tenant(f"tenant_b_{uuid.uuid4().hex[:8]}")
        
        id_a = tenant_a["id"]
        id_b = tenant_b["id"]
        
        print(f"Tenant A ID: {id_a}")
        print(f"Tenant B ID: {id_b}")
        
        # 2. Create Users
        user_a = create_user(id_a, f"user_a_{uuid.uuid4().hex[:6]}", "password123")
        user_b = create_user(id_b, f"user_b_{uuid.uuid4().hex[:6]}", "password123")
        
        # 3. Login to get Tokens
        token_a = login(user_a["username"], "password123")
        token_b = login(user_b["username"], "password123")
        
        print(f"Got Token A: {token_a[:10]}...")
        print(f"Got Token B: {token_b[:10]}...")
        
        # 4. Create Namespaces (using Iceberg REST API directly to avoid PyIceberg complexity for now)
        # Verify A can create in A
        headers_a = {"Authorization": f"Bearer {token_a}"}
        headers_b = {"Authorization": f"Bearer {token_b}"}
        
        # We need a catalog. Does creating a tenant create a default catalog?
        # Likely not. We need to Create Catalog.
        # Let's try to create a catalog using the User (TenantAdmin) who definitely belongs to the tenant.
        
        print("Creating catalogs as Tenant Admins...")
        requests.post(f"{BASE_URL}/api/v1/catalogs", json={"name": "main", "type": "memory"}, headers=headers_a).raise_for_status()
        requests.post(f"{BASE_URL}/api/v1/catalogs", json={"name": "main", "type": "memory"}, headers=headers_b).raise_for_status()
        
        print("Creating namespaces...")
        # Create 'ns_a' in Tenant A
        resp = requests.post(f"{BASE_URL}/v1/main/identifiers", json={"name": ["ns_a"]}, headers=headers_a)
        # Iceberg REST: POST /v1/{prefix}/namespaces. 
        # Create namespace: POST /v1/{prefix}/namespaces
        resp = requests.post(f"{BASE_URL}/v1/main/namespaces", json={"namespace": ["ns_a"]}, headers=headers_a)
        if resp.status_code != 200:
             print(f"Failed to create ns_a: {resp.text}")
        resp.raise_for_status()
        print("Created ns_a in Tenant A")
        
        # Create 'ns_b' in Tenant B
        requests.post(f"{BASE_URL}/v1/main/namespaces", json={"namespace": ["ns_b"]}, headers=headers_b).raise_for_status()
        print("Created ns_b in Tenant B")
        
        # 5. Verify Isolation
        print("Verifying Isolation...")
        
        # User A should see ns_a
        resp = requests.get(f"{BASE_URL}/v1/main/namespaces", headers=headers_a)
        resp.raise_for_status()
        namespaces_a = resp.json()["namespaces"]
        print(f"User A sees: {namespaces_a}")
        assert ["ns_a"] in namespaces_a
        
        # User A should NOT see ns_b
        assert ["ns_b"] not in namespaces_a
        
        # User B should see ns_b
        resp = requests.get(f"{BASE_URL}/v1/main/namespaces", headers=headers_b)
        resp.raise_for_status()
        namespaces_b = resp.json()["namespaces"]
        print(f"User B sees: {namespaces_b}")
        assert ["ns_b"] in namespaces_b
        
        # User B should NOT see ns_a
        assert ["ns_a"] not in namespaces_b
        
        print("\n✅ Multi-Tenant Isolation Verified Successfully!")
        
    except Exception as e:
        print(f"\n❌ Test Failed: {e}")
        # Print full response if available
        if hasattr(e, 'response') and e.response is not None:
             print(f"Response: {e.response.status_code} {e.response.text}")
        sys.exit(1)

if __name__ == "__main__":
    main()
