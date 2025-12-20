
import requests
import json
import uuid

BASE_URL = "http://localhost:8080"
TENANT_A_ID = str(uuid.uuid4()) # Dynamic UUIDs
TENANT_B_ID = str(uuid.uuid4())

def get_token(username, password):
    resp = requests.post(f"{BASE_URL}/api/v1/users/login", json={"username": username, "password": password})
    resp.raise_for_status()
    return resp.json()["token"]

RUN_ID = str(uuid.uuid4())[:8]
TENANT_A_NAME = f"tenant_a_{RUN_ID}"
TENANT_B_NAME = f"tenant_b_{RUN_ID}"
ADMIN_A_NAME = f"admin_a_{RUN_ID}"
ADMIN_B_NAME = f"admin_b_{RUN_ID}"
USER_A_NAME = f"user_a_{RUN_ID}"

def main():
    # 1. Login as Root
    print("Logging in as Root...")
    root_token = get_token("admin", "password")
    root_headers = {"Authorization": f"Bearer {root_token}"}

    # 2. Create Tenants
    print(f"Creating Tenants ({TENANT_A_NAME}, {TENANT_B_NAME})...")
    resp_a = requests.post(f"{BASE_URL}/api/v1/tenants", json={"name": TENANT_A_NAME}, headers=root_headers)
    resp_a.raise_for_status()
    ACTUAL_TENANT_A_ID = resp_a.json()["id"]
    
    resp_b = requests.post(f"{BASE_URL}/api/v1/tenants", json={"name": TENANT_B_NAME}, headers=root_headers)
    resp_b.raise_for_status()
    ACTUAL_TENANT_B_ID = resp_b.json()["id"]

    # 3. Create Tenant Admins
    print(f"Creating Tenant Admins ({ADMIN_A_NAME}, {ADMIN_B_NAME})...")
    resp = requests.post(f"{BASE_URL}/api/v1/users", json={
        "username": ADMIN_A_NAME, "email": f"{ADMIN_A_NAME}@test.com", "password": "password", "role": "tenant-admin", "tenant_id": ACTUAL_TENANT_A_ID
    }, headers=root_headers)
    resp.raise_for_status()
    print(f"Created {ADMIN_A_NAME}: {resp.json()}")
    
    requests.post(f"{BASE_URL}/api/v1/users", json={
        "username": ADMIN_B_NAME, "email": f"{ADMIN_B_NAME}@test.com", "password": "password", "role": "tenant-admin", "tenant_id": ACTUAL_TENANT_B_ID
    }, headers=root_headers).raise_for_status()

    # 4. Login as Admin A
    print(f"Logging in as {ADMIN_A_NAME}...")
    resp = requests.post(f"{BASE_URL}/api/v1/users/login", json={"username": ADMIN_A_NAME, "password": "password"})
    resp.raise_for_status()
    data = resp.json()
    admin_a_token = data["token"]
    received_tenant_id = data["user"]["tenant-id"]
    print(f"Local TENANT_A_ID: {ACTUAL_TENANT_A_ID}")
    print(f"Received Tenant ID: {received_tenant_id}")
    
    # Decode token to check claims (without verify validation for debug)
    # Payload is 2nd part
    try:
        payload_part = admin_a_token.split('.')[1]
        # Pad base64
        payload_part += '=' * (-len(payload_part) % 4)
        import base64
        decoded_bytes = base64.b64decode(payload_part)
        claims = json.loads(decoded_bytes)
        print(f"Token Claims: {json.dumps(claims, indent=2)}")
        if claims.get("tenant_id") != ACTUAL_TENANT_A_ID:
            print(f"TOKEN CLAIM MISMATCH! Claim: {claims.get('tenant_id')} vs Local: {ACTUAL_TENANT_A_ID}")
    except Exception as e:
        print(f"Failed to decode token: {e}")

    a_headers = {"Authorization": f"Bearer {admin_a_token}"}
    
    # Update globals for subsequent use if needed (usually scoped but here script is linear)
    TENANT_A_ID = ACTUAL_TENANT_A_ID 

    # 5. Create Warehouse A
    print("Creating Warehouse A...")
    requests.post(f"{BASE_URL}/api/v1/warehouses", json={
        "name": "warehouse_a", 
        "use_sts": False,
        "storage_config": {
            "type": "s3", 
            "bucket": "bucket", 
            "region": "us-east-1",
            "endpoint": "http://localhost:9000", 
            "access_key": "minio", 
            "secret_key": "minio123"
        }
    }, headers=a_headers).raise_for_status()

    # 6. Create Catalog A
    print("Creating Catalog A...")
    requests.post(f"{BASE_URL}/api/v1/catalogs", json={
        "name": "catalog_a", "catalog_type": "Local", "warehouse_name": "warehouse_a", "storage_location": "s3://bucket/catalog_a"
    }, headers=a_headers).raise_for_status()

    # 7. Create User A
    print(f"Creating {USER_A_NAME}...")
    try:
        requests.post(f"{BASE_URL}/api/v1/users", json={
            "username": USER_A_NAME, "email": f"{USER_A_NAME}@test.com", "password": "password", "role": "tenant-user", "tenant_id": TENANT_A_ID
        }, headers=a_headers).raise_for_status()
    except requests.exceptions.HTTPError as e:
        print(f"Error creating {USER_A_NAME}: {e}")
        print(f"Response: {e.response.text}")
        raise

    # 8. Create Tables in Catalog A
    print("Creating Tables...")
    rest_base = f"{BASE_URL}/v1/catalog_a"
    try:
        requests.post(f"{rest_base}/namespaces", json={"namespace": ["data"]}, headers=a_headers).raise_for_status()
    except requests.exceptions.HTTPError: pass # Ignore if exists

    schema = {"type": "struct", "fields": [{"id": 1, "name": "id", "required": True, "type": "int"}]}
    
    try:
        requests.post(f"{rest_base}/namespaces/data/tables", json={"name": "table_allow", "schema": schema}, headers=a_headers).raise_for_status()
    except requests.exceptions.HTTPError: pass
    
    try:
        requests.post(f"{rest_base}/namespaces/data/tables", json={"name": "table_discoverable", "schema": schema}, headers=a_headers).raise_for_status()
    except requests.exceptions.HTTPError: pass

    print("State restored successfully!")

if __name__ == "__main__":
    main()
