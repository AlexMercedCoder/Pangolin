#!/usr/bin/env python3
import json
import os
import sys
import time
import urllib.request
import urllib.error
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, IntegerType, TimestampType
import pyarrow as pa

API_URL = "http://localhost:8080"
ADMIN_USER = "admin"
ADMIN_PASS = "password"
WAREHOUSE_LOC = "s3://warehouse"

def json_request(endpoint, method="GET", data=None, token=None):
    url = f"{API_URL}{endpoint}"
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
        
    body = json.dumps(data).encode("utf-8") if data else None
    req = urllib.request.Request(url, data=body, headers=headers, method=method)
    
    try:
        with urllib.request.urlopen(req) as response:
            if response.status >= 200 and response.status < 300:
                resp_data = response.read()
                if resp_data:
                    return json.loads(resp_data)
                return {}
            else:
                print(f"Error {response.status}: {response.read().decode()}")
                return None
    except urllib.error.HTTPError as e:
        print(f"HTTP Error {e.code} for {url}: {e.read().decode()}")
        return None
    except Exception as e:
        print(f"Request failed: {e}")
        return None

def wait_for_api():
    print("Waiting for API to be ready...")
    for _ in range(30):
        try:
            with urllib.request.urlopen(f"{API_URL}/health") as response:
                if response.status == 200:
                    print("API is ready.")
                    return True
        except:
            time.sleep(1)
    print("API timed out.")
    return False

def seed():
    if not wait_for_api():
        sys.exit(1)

    print("\n1. Logging in as Root...")
    login_resp = json_request("/api/v1/users/login", "POST", {"username": ADMIN_USER, "password": ADMIN_PASS})
    if not login_resp:
        print("Failed to login as root")
        sys.exit(1)
    root_token = login_resp["token"]
    print("Root logged in.")

    print("\n2. Creating Tenant 'manual_test_tenant'...")
    # Check if exists first? Local store is ephemeral, so likely clean.
    tenant_resp = json_request("/api/v1/tenants", "POST", {"name": "manual_test_tenant", "provider": "local"}, token=root_token)
    if not tenant_resp:
        print("Failed to create tenant (might already exist, continuing...)")
        # Try to find it or exit might be too harsh if we are re-running.
        # Ideally we list tenants and find it.
        tenants = json_request("/api/v1/tenants", "GET", token=root_token)
        found = next((t for t in tenants if t["name"] == "manual_test_tenant"), None)
        if found:
            tenant_id = found["id"]
            print(f"Found existing tenant: {tenant_id}")
        else:
            print("Could not create or find tenant.")
            sys.exit(1)
    else:
        tenant_id = tenant_resp.get("id")
        print(f"Tenant Created: {tenant_id}")

    print("\n3. Creating Tenant Admin 'manual_admin'...")
    # Try to create, ignore status if 400 (exists)
    admin_create_resp = json_request("/api/v1/users", "POST", {
        "username": "manual_admin", 
        "email": "admin@manual.test", 
        "password": "password", 
        "role": "tenant-admin", 
        "tenant_id": tenant_id
    }, token=root_token)
    
    if admin_create_resp:
        print("Tenant Admin Created.")
    else:
        print("Tenant Admin might already exist.")

    print("\n4. Logging in as 'manual_admin'...")
    admin_login = json_request("/api/v1/users/login", "POST", {"username": "manual_admin", "password": "password"})
    if not admin_login:
        print("Failed to login as manual_admin")
        sys.exit(1)
    admin_token = admin_login["token"]
    print("Manual Admin Logged In.")

    print("\n5. Creating Catalog 'lakehouse_cat'...")
    # Create Catalog via API
    cat_req = {
        "name": "lakehouse_cat",
        "type": "rest", # or managed/local
        "storage_location": f"{WAREHOUSE_LOC}/lakehouse_cat",
        "tenant_id": tenant_id
    }
    # Check if catalog exists
    # If not create.
    # Actually, let's just try creation.
    # Note: Catalog creation endpoint
    cat_resp = json_request("/api/v1/catalogs", "POST", cat_req, token=admin_token)
    if cat_resp:
        print("Catalog 'lakehouse_cat' created.")
    else:
        print("Catalog creation skipped (may exist).")

    print("\n6. Seeding Iceberg Data (Namespace & Table)...")
    # Initialize PyIceberg Catalog
    try:
        catalog = load_catalog(
            "pangolin",
            **{
                "uri": f"{API_URL}/v1/lakehouse_cat", # Prefix with catalog name for REST catalog routing in Pangolin
                "s3.endpoint": "http://localhost:9000",
                "s3.access-key-id": "minioadmin",
                "s3.secret-access-key": "minioadmin",
                "s3.path-style-access": "true",
                "s3.region": "us-east-1",
                "token": admin_token # Use the admin token for auth
            }
        )
        
        # Create Namespace
        try:
            catalog.create_namespace("sales")
            print("Namespace 'sales' created.")
        except Exception as e:
            print(f"Namespace 'sales' likely exists: {e}")

        # Create Table
        try:
            schema = Schema(
                NestedField(1, "id", IntegerType(), required=True),
                NestedField(2, "product", StringType(), required=False),
                NestedField(3, "amount", IntegerType(), required=False),
                NestedField(4, "created_at", TimestampType(), required=False),
            )
            table = catalog.create_table(
                identifier="sales.orders",
                schema=schema,
                location=f"{WAREHOUSE_LOC}/lakehouse_cat/sales/orders"
            )
            print("Table 'sales.orders' created.")
            
            # Insert Data
            data = pa.table({
                "id": [1, 2, 3],
                "product": ["Widget A", "Widget B", "Widget C"],
                "amount": [100, 200, 300],
                "created_at": [
                    pa.scalar(int(time.time() * 1000000), type=pa.timestamp('us')),
                    pa.scalar(int(time.time() * 1000000), type=pa.timestamp('us')),
                    pa.scalar(int(time.time() * 1000000), type=pa.timestamp('us')),
                ]
            })
            table.append(data)
            print("Data inserted into 'sales.orders'.")
            
        except Exception as e:
            print(f"Table operation skipped/failed: {e}")
            
    except Exception as e:
        print(f"PyIceberg initialization failed: {e}")

    print("\n=== SEEDING COMPLETE ===")
    print(f"Tenant ID: {tenant_id}")
    print("User: manual_admin / password")
    
if __name__ == "__main__":
    seed()
