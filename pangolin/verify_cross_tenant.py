import requests
import time
import uuid
import jwt
import datetime

# This script assumes pangolin_api is running with PANGOLIN_JWT_SECRET="test_secret" 
# and potentially PANGOLIN_NO_AUTH=false (or we use the secret to sign tokens anyway)

BASE_URL = "http://localhost:8081"
JWT_SECRET = "test_secret"

def create_token(user_id, username, tenant_id, role="tenant-admin"):
    payload = {
        "sub": str(user_id),
        "username": username,
        "tenant_id": str(tenant_id),
        "role": role,
        "exp": datetime.datetime.utcnow() + datetime.timedelta(hours=1),
        "iat": datetime.datetime.utcnow()
    }
    return jwt.encode(payload, JWT_SECRET, algorithm="HS256")

def wait_for_api():
    print("Waiting for API to be ready...")
    for _ in range(10):
        try:
            requests.get(f"{BASE_URL}/health")
            print("API is ready!")
            return
        except requests.exceptions.ConnectionError:
            time.sleep(2)
    raise Exception("API failed to start")

def test_cross_tenant_federation():
    print("Starting Cross-Tenant Federation Test...")
    
    # 1. Setup Tenant A (Consumer)
    tenant_a_id = str(uuid.uuid4())
    print(f"Tenant A ID: {tenant_a_id}")
    
    # Generate explicit Tenant Admin Tokens for A and B
    # Since we can't easily create tenants in NO_AUTH mode (it forces default tenant),
    # we will Simulate Multi-Tenancy by directly generating tokens with DIFFERENT tenant IDs.
    # In 'memory' store, if the tenant doesn't exist, it might fail validation if checks are strict.
    # But usually checks just look for tenant existence.
    # If NO_AUTH is ON, the API overrides the context.
    # TO TEST THIS PROPERLY, we need PANGOLIN_NO_AUTH=FALSE.
    
    user_a_id = str(uuid.uuid4())
    token_a = create_token(user_a_id, "admin_a", tenant_a_id)
    headers_a = {"Authorization": f"Bearer {token_a}"}
    
    # 2. Setup Tenant B (Provider)
    tenant_b_id = str(uuid.uuid4())
    print(f"Tenant B ID: {tenant_b_id}")
    user_b_id = str(uuid.uuid4())
    token_b = create_token(user_b_id, "admin_b", tenant_b_id)
    headers_b = {"Authorization": f"Bearer {token_b}"}
    
    # We first need to CREATE the Tenants if the system enforces FKs.
    # The Root user can create tenants.
    root_token = create_token(uuid.uuid4(), "root", uuid.UUID(int=0), role="root")
    root_headers = {"Authorization": f"Bearer {root_token}"}
    
    print("Creating Tenant A...")
    res_a = requests.post(f"{BASE_URL}/api/v1/tenants", headers=root_headers, json={
        "name": f"tenant_a_{tenant_a_id[:8]}", # Unique name
        "admin_username": "admin_a",
        "admin_password": "password_a"
    })
    if res_a.status_code == 201:
        tenant_a_id = res_a.json()["id"]
        # Regenerate Token A with correct ID
        token_a = create_token(user_a_id, "admin_a", tenant_a_id)
        headers_a = {"Authorization": f"Bearer {token_a}"}
        print(f"Created Tenant A: {tenant_a_id}")
    else:
        print(f"Warning: Failed to create Tenant A ({res_a.status_code}), proceeding with generated ID (might fail if validation logic exists)")

    print("Creating Tenant B...")
    res_b = requests.post(f"{BASE_URL}/api/v1/tenants", headers=root_headers, json={
        "name": f"tenant_b_{tenant_b_id[:8]}",
        "admin_username": "admin_b",
        "admin_password": "password_b"
    })
    if res_b.status_code == 201:
        tenant_b_id = res_b.json()["id"]
        token_b = create_token(user_b_id, "admin_b", tenant_b_id)
        headers_b = {"Authorization": f"Bearer {token_b}"}
        print(f"Created Tenant B: {tenant_b_id}")
    
    
    # 3. Create Resources in Tenant B
    print("Creating Warehouse/Catalog in Tenant B...")
    requests.post(f"{BASE_URL}/api/v1/warehouses", headers=headers_b, json={
        "name": "warehouse_b",
        "storage_type": "memory",
        "config": {}
    })
    requests.post(f"{BASE_URL}/api/v1/catalogs", headers=headers_b, json={
        "name": "catalog_b",
        "warehouse": "warehouse_b",
        "type": "pangea" # or "managed" / "local"
    })
    
    requests.post(f"{BASE_URL}/v1/catalog_b/namespaces", headers=headers_b, json={"namespace": ["ns_b"]})

    # 4. Federate from Tenant A
    print("Federating from Tenant A...")
    
    # The URI is the INTERNAL address of the API itself for catalog_b
    # Important: We must not run this test if NO_AUTH=true because all requests would have same context.
    
    fed_body = {
        "name": "fed_cat_a",
        "catalog_type": "Federated",
        "storage_location": "s3://dummy/loc",
        "federated_config": {
            "properties": {
                "uri": f"{BASE_URL}/v1/catalog_b",
                "token": token_b, 
                "header.X-Pangolin-Tenant": tenant_b_id
            }
        },
        "properties": {}
    }
    
    res = requests.post(f"{BASE_URL}/api/v1/catalogs", headers=headers_a, json=fed_body)
    print(f"Create Catalog Status: {res.status_code}")
    if res.status_code != 201:
        print(f"Result: {res.text}")
        raise Exception("Failed to create federated catalog")
        
    print("Federated Catalog Created. Verifying Access...")
    
    # 5. Access Data
    # List namespaces
    res = requests.get(f"{BASE_URL}/v1/fed_cat_a/namespaces", headers=headers_a)
    print(f"Listing: {res.status_code}")
    print(res.json())
    
    # The response format for list namespaces is {"namespaces": [["ns_b"]]} or similar depending on Iceberg spec
    # Our implementation returns a JSON list or object.
    
    data = res.json()
    namespaces = data.get("namespaces", [])
    # Flatten if needed or check direct existence
    # Iceberg REST returns identifiers (list of strings)
    
    found = False
    for ns in namespaces:
        if ns == ["ns_b"] or ns == "ns_b":
            found = True
            break
            
    if found:
        print("✅ SUCCESS: Cross-Tenant Federation Verified!")
    else:
        print("❌ FAILURE: Could not see Tenant B namespace")
        print(namespaces)
        exit(1)

if __name__ == "__main__":
    wait_for_api()
    test_cross_tenant_federation()
