
import requests
import json
import time
import os
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, IntegerType

API_URL = "http://localhost:8080"
ROOT_USER = os.getenv("PANGOLIN_ROOT_USER", "admin")
ROOT_PASS = os.getenv("PANGOLIN_ROOT_PASSWORD", "password")

def print_step(msg):
    print(f"\n{'='*50}\n{msg}\n{'='*50}")

def print_result(msg, success=True):
    icon = "✓" if success else "✗"
    print(f"{icon} {msg}")

def get_admin_token():
    print(f"Logging in with {ROOT_USER} / {ROOT_PASS}")
    resp = requests.post(f"{API_URL}/api/v1/users/login", json={"username": ROOT_USER, "password": ROOT_PASS})
    if resp.status_code != 200:
        print(f"Login failed: {resp.text}")
    resp.raise_for_status()
    return resp.json()["token"]

def create_tenant(name):
    token = get_admin_token()
    resp = requests.post(f"{API_URL}/api/v1/tenants", json={"name": name}, headers={"Authorization": f"Bearer {token}"})
    if resp.status_code == 201:
        return resp.json()["id"]
    # If exists, find it
    users = requests.get(f"{API_URL}/api/v1/tenants", headers={"Authorization": f"Bearer {token}"}).json()
    for u in users:
        if u["name"] == name:
            return u["id"]
    return None

def create_tenant_admin(tenant_id, username, password):
    token = get_admin_token()
    payload = {"username": username, "password": password, "tenant_id": tenant_id, "role": "tenant-admin", "email": f"{username}@example.com"}
    resp = requests.post(f"{API_URL}/api/v1/users", json=payload, headers={"Authorization": f"Bearer {token}"})
    print(f"Register {username}: {resp.status_code} {resp.text}")

    # Login to get token
    resp = requests.post(f"{API_URL}/api/v1/users/login", json={"username": username, "password": password})
    if resp.status_code != 200:
        print(f"User login failed: {resp.text}")
    return resp.json()["token"]

def run_debug():
    try:
        print_step("Setting up Tenants")
        id_a = create_tenant("tenant_a_debug")
        id_b = create_tenant("tenant_b_debug")
        print(f"Tenant A: {id_a}, Tenant B: {id_b}")
        
        token_a = create_tenant_admin(id_a, "admin_a_debug", "password")
        token_b = create_tenant_admin(id_b, "admin_b_debug", "password")
        
        headers_a = {"Authorization": f"Bearer {token_a}"}
        headers_b = {"Authorization": f"Bearer {token_b}"}
        
        # 1. Setup B (Provider)
        print_step("Setting up Provider (Tenant B)")
        cat_b = {
            "name": "catalog_b",
            "storage_location": "s3://warehouse/catalog_b"
        }
        res = requests.post(f"{API_URL}/api/v1/catalogs", json=cat_b, headers=headers_b)
        print(f"Create Catalog B: {res.status_code}")

        # PyIceberg B Setup
        client_b = load_catalog("catalog_b", **{
            "uri": f"{API_URL}/v1/catalog_b",
            "token": token_b,
            "s3.endpoint": "http://localhost:9000",
            "s3.access-key-id": "minioadmin",
            "s3.secret-access-key": "minioadmin",
        })
        
        client_b.create_namespace("db_b")
        schema = Schema(NestedField(1, "id", IntegerType(), required=True))
        client_b.create_table("db_b.table_b", schema=schema)
        print("✓ Created db_b.table_b")
        
        # 2. Setup A (Federated)
        print_step("Setting up Federated Catalog (Tenant A)")
        fed_cat = {
            "name": "fed_b",
            "catalog_type": "Federated",
            "storage_location": "s3://warehouse/fed_b",  # Required even for federated
            "federated_config": {
                "base_url": f"{API_URL}/v1/catalog_b",
                "auth_type": "BearerToken",
                "credentials": {"token": token_b},
                "timeout_seconds": 30
            }
        }
        res = requests.post(f"{API_URL}/api/v1/catalogs", json=fed_cat, headers=headers_a)
        print(f"Create Federated Catalog: {res.status_code}")
        if res.status_code != 201:
            print(f"Error: {res.text}")
        
        # 3. Access
        print_step("Verifying Access (Tenant A)")
        client_a = load_catalog("fed_b", **{
            "uri": f"{API_URL}/v1/fed_b",
            "token": token_a,
            "s3.endpoint": "http://localhost:9000",
            "s3.access-key-id": "minioadmin",
            "s3.secret-access-key": "minioadmin",
        })
        
        ns = client_a.list_namespaces()
        print(f"Namespaces found: {ns}")
        
        if ("db_b",) in ns or "db_b" in ns:
             print("SUCCESS: Found namespace")
        else:
             print("FAILURE: Namespace not found")
             
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    run_debug()
