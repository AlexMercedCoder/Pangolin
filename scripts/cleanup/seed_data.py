import requests
import json
import uuid
import boto3
from botocore.exceptions import ClientError

BASE_URL = "http://localhost:8080/api/v1"
S3_ENDPOINT = "http://localhost:9000"
AWS_ACCESS_KEY = "minioadmin"
AWS_SECRET_KEY = "minioadmin"

# Setup S3 Client for MinIO
s3 = boto3.client(
    's3',
    endpoint_url=S3_ENDPOINT,
    aws_access_key_id=AWS_ACCESS_KEY,
    aws_secret_access_key=AWS_SECRET_KEY,
    region_name='us-east-1'
)

def ensure_bucket(bucket_name):
    try:
        s3.create_bucket(Bucket=bucket_name)
        print(f"[S3] Created bucket: {bucket_name}")
    except ClientError as e:
        if e.response['Error']['Code'] == 'BucketAlreadyOwnedByYou':
            print(f"[S3] Bucket already exists: {bucket_name}")
        else:
            print(f"[S3] Error creating bucket: {e}")

def print_step(msg):
    print(f"\n--- {msg} ---")

def api_request(method, endpoint, **kwargs):
    url = f"{BASE_URL}{endpoint}"
    response = requests.request(method, url, **kwargs)
    if response.status_code >= 400:
        print(f"[FAIL] {method} {url}: {response.status_code} - {response.text}")
        return None
    return response.json() if response.content else {}

# --- Authorization Helpers ---
ROOT_TOKEN = None
def get_root_token():
    global ROOT_TOKEN
    if not ROOT_TOKEN:
        payload = {"username": "admin", "password": "password"}
        resp = api_request("POST", "/users/login", json=payload)
        if resp:
            ROOT_TOKEN = resp["token"]
    return ROOT_TOKEN

def login(username, password, tenant_id=None):
    payload = {"username": username, "password": password}
    if tenant_id:
        payload["tenant-id"] = tenant_id # Use Kebab Case for serialization
    
    resp = requests.post(f"{BASE_URL}/users/login", json=payload)
    if resp.status_code == 200:
        return resp.json()["token"]
    else:
        print(f"[FAIL] Login {username}: {resp.status_code} {resp.text}")
        return None

# --- Main Execution ---

ensure_bucket("test-bucket")

# 1. Root Login
print_step("Logging in as Root")
root_token = get_root_token()
root_headers = {"Authorization": f"Bearer {root_token}"}
print("Root Logged In")

# 2. Get Default Tenant ID
print_step("Getting Default Tenant")
tenants = api_request("GET", "/tenants", headers=root_headers)
if isinstance(tenants, list):
    default_tenant = next((t for t in tenants if t["name"] == "Default Tenant"), None)
    default_tenant_id = default_tenant["id"] if default_tenant else "00000000-0000-0000-0000-000000000000"
else:
    print(f"Warning: /tenants returned non-list: {tenants}")
    default_tenant_id = "00000000-0000-0000-0000-000000000000"
    
print(f"Default Tenant ID: {default_tenant_id}")

# 3. Create Second Tenant
print_step("Creating Marketing Tenant")
mkt_tenant = api_request("POST", "/tenants", 
    json={"name": "Marketing Analytics", "description": "Marketing Dept"}, 
    headers=root_headers
)
mkt_tenant_id = mkt_tenant["id"] if mkt_tenant else None
print(f"Marketing Tenant ID: {mkt_tenant_id}")

# 4. Create Users (Tenant Admins & Users)

# Root creates Admins
admins_to_create = [
    {
        "role": "tenant-admin",
        "username": "eng_admin",
        "email": "admin@eng.com",
        "password": "password",
        "tenant_id": default_tenant_id
    },
    {
        "role": "tenant-admin",
        "username": "mkt_admin",
        "email": "admin@mkt.com",
        "password": "password",
        "tenant_id": mkt_tenant_id
    }
]

print_step("Creating Tenant Admins")
for u in admins_to_create:
    payload = {
        "username": u["username"],
        "email": u["email"],
        "password": u["password"],
        "tenant_id": u["tenant_id"],
        "role": u["role"]
    }
    api_request("POST", "/users", json=payload, headers=root_headers)
    print(f"Created {u['username']} in {u['tenant_id']}")


# 5. Create Infrastructure (Warehouses & Catalogs)
# Login as Eng Admin to create Eng resources
print_step("Configuring Engineering Tenant resources")
eng_token = login("eng_admin", "password", default_tenant_id)
eng_headers = {"Authorization": f"Bearer {eng_token}"}

if eng_token:
    # Create Eng User
    api_request("POST", "/users", json={
        "username": "eng_user",
        "email": "user@eng.com",
        "password": "password",
        "tenant_id": default_tenant_id,
        "role": "tenant-user"
    }, headers=eng_headers)
    print(f"Created eng_user in {default_tenant_id}")

    # S3 Warehouse
    wh = api_request("POST", "/warehouses", json={
        "name": "s3-warehouse",
        "warehouse_type": "s3",
        "storage_config": {
            "bucket": "test-bucket",
            "region": "us-east-1",
            "endpoint": S3_ENDPOINT,
            "path_style_access": "true" # Must be string
        }
    }, headers=eng_headers)

    
    # Engineering Catalog
    cat = api_request("POST", "/catalogs", json={
        "name": "engineering",
        "warehouse_name": "s3-warehouse",
        "type": "managed",
        "tenant_id": default_tenant_id
    }, headers=eng_headers)
    
    # Namespace & Table
    if cat:
        # Create Namespace /v1/{cat_name}/namespaces
        ns_url = f"http://localhost:8080/v1/engineering/namespaces"
        requests.post(ns_url, json={"namespace": ["logs"]}, headers=eng_headers)
        
        # Create Table /v1/{cat_name}/namespaces/{ns}/tables
        tbl_url = f"http://localhost:8080/v1/engineering/namespaces/logs/tables"
        tbl_payload = {
            "name": "server_logs",
            "schema": {
                "type": "struct",
                "fields": [
                    {"id": 1, "name": "timestamp", "type": "timestamp", "required": True},
                    {"id": 2, "name": "level", "type": "string", "required": False},
                    {"id": 3, "name": "message", "type": "string", "required": False}
                ]
            }
        }
        resp = requests.post(tbl_url, json=tbl_payload, headers=eng_headers)
        if resp.status_code == 200:
            print("Created Table: engineering.logs.server_logs")

# Login as Mkt Admin
print_step("Configuring Marketing Tenant resources")
mkt_token = login("mkt_admin", "password", mkt_tenant_id)
mkt_headers = {"Authorization": f"Bearer {mkt_token}"}

if mkt_token:
    # Create Mkt User
    api_request("POST", "/users", json={
        "username": "mkt_user",
        "email": "user@mkt.com",
        "password": "password",
        "tenant_id": mkt_tenant_id,
        "role": "tenant-user"
    }, headers=mkt_headers)
    print(f"Created mkt_user in {mkt_tenant_id}")

    # Need a warehouse for Marketing too - can reuse same config logic since it's logical
    wh = api_request("POST", "/warehouses", json={
        "name": "mkt-warehouse",
        "warehouse_type": "s3",
        "storage_config": {
            "bucket": "test-bucket",
            "region": "us-east-1",
            "endpoint": S3_ENDPOINT,
            "prefix": "marketing/" 
        }
    }, headers=mkt_headers)
    
    # Marketing Catalog
    cat = api_request("POST", "/catalogs", json={
        "name": "campaigns_cat",
        "warehouse_name": "mkt-warehouse",
        "type": "managed",
        "tenant_id": mkt_tenant_id
    }, headers=mkt_headers)
    
     # Namespace & Table
    if cat:
        ns_url = f"http://localhost:8080/v1/campaigns_cat/namespaces"
        requests.post(ns_url, json={"namespace": ["q1_2025"]}, headers=mkt_headers)
        
        tbl_url = f"http://localhost:8080/v1/campaigns_cat/namespaces/q1_2025/tables"
        tbl_payload = {
            "name": "ad_spend",
            "schema": {
                "type": "struct",
                "fields": [
                    {"id": 1, "name": "campaign_id", "type": "int", "required": True},
                    {"id": 2, "name": "spend", "type": "double", "required": True}
                ]
            }
        }
        r = requests.post(tbl_url, json=tbl_payload, headers=mkt_headers)
        if r.status_code == 200:
            print("Created Table: campaigns_cat.q1_2025.ad_spend")
            
            # Make Discoverable!
            # Search to get ID
            s = api_request("GET", "/search/assets?q=ad_spend", headers=mkt_headers)
            if s and s.get("results"):
                asset_id = s["results"][0]["id"]
                # Update Metadata
                api_request("POST", f"/assets/{asset_id}/metadata", json={
                    "description": "Public Ad Spend Data",
                    "tags": ["public", "marketing"],
                    "discoverable": True,
                    "properties": {}
                }, headers=mkt_headers)
                print("Made 'ad_spend' discoverable")


print("\n\n=======================================================")
print("             ENVIRONMENT SEEDED SUCCESSFULLY           ")
print("=======================================================")
print(f"Default Tenant ID: {default_tenant_id}")
print(f"Marketing Tenant ID: {mkt_tenant_id}")
print("-------------------------------------------------------")
print(" Users:")
print(f"   Root:          admin / password")
print(f"   Eng Admin:     eng_admin / password  (Default Tenant)")
print(f"   Eng User:      eng_user / password   (Default Tenant)")
print(f"   Mkt Admin:     mkt_admin / password  (Marketing Tenant)")
print(f"   Mkt User:      mkt_user / password   (Marketing Tenant)")
print("-------------------------------------------------------")
print(" Resources:")
print("   Engineering:   Catalog 'engineering' -> Table 'logs.server_logs'")
print("   Marketing:     Catalog 'campaigns_cat' -> Table 'q1_2025.ad_spend' (Discoverable)")
print("=======================================================")
