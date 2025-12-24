
import requests
import json
import uuid
import os

BASE_URL = "http://localhost:8080/api/v1"
DATA_DIR = "/tmp/pangolin_data"

def print_step(msg):
    print(f"\n--- {msg} ---")

def api_request(method, endpoint, **kwargs):
    url = f"{BASE_URL}{endpoint}"
    response = requests.request(method, url, **kwargs)
    if response.status_code >= 400:
        print(f"[FAIL] {method} {url}: {response.status_code} - {response.text}")
        return None
    return response.json() if response.content else {}

# 1. Login
print_step("Logging in as Tenant Admin")
# In No Auth mode, password check is bypassed, so this works
resp = requests.post(f"{BASE_URL}/users/login", json={
    "username": "tenant_admin",
    "password": "password123",
    "tenant-id": "00000000-0000-0000-0000-000000000000"
})

if resp.status_code != 200:
    print(f"Login Failed: {resp.text}")
    exit(1)

token = resp.json()["token"]
user_info = resp.json()["user"]
# API uses kebab-case
tenant_id = user_info.get("tenant-id") or "00000000-0000-0000-0000-000000000000"
headers = {"Authorization": f"Bearer {token}"}
print(f"Logged in. Tenant: {tenant_id}")

# 2. Create Local Warehouse
print_step("Creating Local Warehouse")
# Ensure directory exists
os.makedirs(DATA_DIR, exist_ok=True)

wh = api_request("POST", "/warehouses", json={
    "name": "local-warehouse",
    "warehouse_type": "local", # Ignored by backend, it uses storage_config
    "storage_config": {
        "location": f"file://{DATA_DIR}" # Providing location directly
    }
}, headers=headers)

if not wh:
    # Try getting it if it exists
    print("Warehouse might exist, trying to fetch...")
    wh = api_request("GET", "/warehouses/local-warehouse", headers=headers)

if not wh:
    print("Could not create or find warehouse. Exiting.")
    exit(1)
    
print("Warehouse Ready")

# 3. Create Catalog
print_step("Creating Sales Catalog")
cat = api_request("POST", "/catalogs", json={
    "name": "sales",
    "warehouse_name": "local-warehouse",
    "type": "managed",
    "tenant_id": tenant_id
}, headers=headers)

if not cat:
     cat = api_request("GET", "/catalogs/sales", headers=headers)

print("Catalog Ready")

# 4. Create Namespaces and Tables
if cat:
    print(f"Catalog Created/Found: {json.dumps(cat, indent=2)}")
    print_step("Creating Namespace 'q1_2025'")
    # Iceberg REST API: /v1/{prefix}/namespaces
    # With prefix="sales" (catalog name)
    iceberg_base = "http://localhost:8080/v1/sales"
    
    # Create Namespace
    # Note: Backend might expect /v1/sales/namespaces OR /v1/sales/v1/namespaces depending on client
    # lib.rs has both. Let's use /v1/sales/v1/namespaces to correspond with REST Catalog default behavior
    ns_url = f"{iceberg_base}/v1/namespaces"
    ns_resp = requests.post(ns_url, 
        json={"namespace": ["q1_2025"]}, 
        headers=headers)
    print(f"Create Namespace: {ns_resp.status_code} {ns_resp.text}")

    # Create Table: transactions
    print_step("Creating Table 'transactions'")
    tbl_url = f"{iceberg_base}/v1/namespaces/q1_2025/tables"
    tbl_payload = {
        "name": "transactions",
        "schema": {
            "type": "struct",
            "fields": [
                {"id": 1, "name": "tx_id", "type": "string", "required": True},
                {"id": 2, "name": "amount", "type": "double", "required": True},
                {"id": 3, "name": "ts", "type": "timestamp", "required": False}
            ]
        },
        "location": f"{DATA_DIR}/sales/q1_2025/transactions" # Explicit request
    }
    r = requests.post(tbl_url, json=tbl_payload, headers=headers)
    if r.status_code == 200:
        print("Created Table: sales.q1_2025.transactions")
    else:
        print(f"Failed to create table: {r.status_code} {r.text}")

    # Create Table: customers
    print_step("Creating Table 'customers'")
    tbl_payload_2 = {
        "name": "customers",
        "schema": {
            "type": "struct",
            "fields": [
                {"id": 1, "name": "cust_id", "type": "string", "required": True},
                {"id": 2, "name": "name", "type": "string", "required": False},
                {"id": 3, "name": "email", "type": "string", "required": False}
            ]
        },
        "location": f"{DATA_DIR}/sales/q1_2025/customers"
    }
    r = requests.post(tbl_url, json=tbl_payload_2, headers=headers)
    if r.status_code == 200:
        print("Created Table: sales.q1_2025.customers")
        
        # Make discoverable
        # Search asset id
        s = api_request("GET", "/search/assets?q=customers", headers=headers)
        if s and s.get("results"):
             asset_id = s["results"][0]["id"]
             api_request("POST", f"/assets/{asset_id}/metadata", json={
                 "description": "Customer PII Data",
                 "tags": ["pii", "sales"],
                 "discoverable": True
             }, headers=headers)
             print("Made 'customers' discoverable")

print("\nSeeding Complete.")
