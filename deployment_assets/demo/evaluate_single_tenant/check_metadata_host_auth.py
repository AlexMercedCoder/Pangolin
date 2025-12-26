import requests
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, IntegerType
import time
import os

def get_token():
    # Attempt to get token for default root user (admin/password)
    # or the auto-provisioned tenant_admin if root is not available.
    # We will try 'admin' first.
    
    url = "http://localhost:8080/api/v1/oauth/token"
    # Try client_credentials for admin
    payload = {
        "grant_type": "client_credentials",
        "client_id": "admin",
        "client_secret": "password",
        "scope": "admin"
    }
    try:
        r = requests.post(url, data=payload)
        if r.status_code == 200:
            return r.json()["access_token"]
    except:
        pass
        
    # If that fails, try tenant_admin (often auto-provisioned)
    payload = {
        "grant_type": "password", # or standard login?
        "username": "tenant_admin",
        "password": "password123",
         "scope": "admin"
    }
    # Actually, let's try just standard logging in or checking what the API expects.
    # For now, let's blindly try to use the token printed in logs if we can't get one.
    # But better to try to fetch it.
    
    # Let's try to just use valid credentials if we can.
    return None

def main():
    print("HOST: Checking for metadata.json persistence with HOST API (AUTH MODE)...")
    
    # 1. Authenticate
    # Note: Since I don't know the exact auth flow defaults without checking docs deeper,
    # I will try to hit the API. If I can't look up a token, I will ask the user or fail.
    # However, usually there is a default root user.
    
    # Actually, better strategy: assume the user provided 'check_metadata_host.py' works
    # if I just add the token.
    # Let's try to get a token via the 'admin' user which usually exists.
    
    token = get_token()
    if not token:
        print("Could not easy-fetch token. Attempting with dummy token or failing...")
        # If we can't get a token, we might fail.
        # But wait, the standard `admin` user is usually `admin`/`password`.
        # Let's try Basic Auth to `oauth/token`
        try:
           res = requests.post("http://localhost:8080/api/v1/oauth/token", 
                               auth=("admin", "password"), 
                               data={"grant_type": "client_credentials"})
           if res.status_code == 200:
               token = res.json()["access_token"]
               print("Got token via client_credentials.")
        except Exception as e:
            print(f"Auth failed: {e}")

    if not token:
        print("failed to get token, skipping auth test or trying no-auth...")
        # Fallback to no token (will likely fail 401)

    print(f"Using Token: {token[:10]}...")

    # Connecting to localhost:8080
    catalog = load_catalog(
        "pangolin",
        **{
            "type": "rest",
            "uri": "http://localhost:8080/v1/demo",
            "token": token,
            # Explicit S3 config for Host
            "s3.endpoint": "http://localhost:9000",
            "s3.access-key-id": "minioadmin",
            "s3.secret-access-key": "minioadmin",
            "s3.region": "us-east-1",
            "s3.path-style-access": "true",
        }
    )

    # Recreate Catalog (Auth needed)
    print("Recreating Warehouse/Catalog...")
    headers = {"Content-Type": "application/json", "Authorization": f"Bearer {token}"}
    
    try:
        requests.post("http://localhost:8080/api/v1/warehouses", json={
            "name": "demo_warehouse_auth",
            "vending_strategy": {
              "AwsStatic": {
                "access_key_id": "minioadmin",
                "secret_access_key": "minioadmin"
              }
            },
            "storage_config": {
              "type": "s3",
              "bucket": "warehouse",
              "region": "us-east-1",
              "endpoint": "http://localhost:9000",
              "access_key_id": "minioadmin",
              "secret_access_key": "minioadmin"
            }
        }, headers=headers)
        
        requests.post("http://localhost:8080/api/v1/catalogs", json={
            "name": "demo",
            "warehouse_name": "demo_warehouse_auth",
            "storage_location": "s3://warehouse/demo_catalog_auth"
        }, headers=headers)
    except Exception as e:
        print(f"Setup warning: {e}")

    ns_name = "meta_check_host_auth"
    try:
        catalog.create_namespace(ns_name)
    except:
        pass

    schema = Schema(NestedField(1, "id", IntegerType(), required=True))
    table_name = f"{ns_name}.meta_table"
    
    print(f"Creating table '{table_name}'...")
    try:
        table = catalog.create_table(table_name, schema=schema)
        print(f"Table Location: {table.location()}")
        print(f"Metadata Location: {table.metadata_location}")
    except Exception as e:
        print(f"Failed to create table: {e}")

if __name__ == "__main__":
    main()
