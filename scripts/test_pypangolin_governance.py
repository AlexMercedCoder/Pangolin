import os
import time
import requests
import uuid
from pypangolin import PangolinClient
from pypangolin.exceptions import ForbiddenError

# Configuration
API_URL = "http://localhost:8080"
USERNAME = "admin"
PASSWORD = "password"

def log(msg):
    print(f"[{time.strftime('%H:%M:%S')}] {msg}")

def main():
    log("=== Starting PyPangolin Governance Verification ===")
    
    # 1. Initialize Client & Login
    client = PangolinClient(API_URL, USERNAME, PASSWORD)
    log("✅ Logged in as Root/Admin")
    
    # 2. Setup: Create Tenant & User
    tenant_name = f"gov_tenant_{str(uuid.uuid4())[:8]}"
    tenant = client.tenants.create(tenant_name)
    client.set_tenant(tenant.id)
    log(f"✅ Created Tenant: {tenant.name} ({tenant.id})")
    
    admin_user = f"admin_{tenant_name}"
    client.users.create(admin_user, f"{admin_user}@example.com", "tenant-admin", tenant_id=tenant.id, password="password")
    
    # Log in as tenant admin
    client = PangolinClient(API_URL, admin_user, "password", tenant_id=tenant.id)
    log(f"✅ Logged in as {admin_user}")

    # 3. Roles
    log("--- Testing Roles ---")
    role_name = "DataSteward"
    role = client.roles.create(role_name)
    log(f"✅ Created Role: {role.name} ({role.id})")
    
    roles = client.roles.list()
    role_names = [r.name for r in roles]
    log(f"Found roles: {role_names}")
    assert role_name in role_names
    
    # 4. Service Users
    log("--- Testing Service Users ---")
    svc_user = client.service_users.create("etl-bot")
    log(f"✅ Created Service User: {svc_user.name}")
    assert svc_user.api_key is not None
    
    svc_users = client.service_users.list()
    log(f"Found service users: {[u.name for u in svc_users]}")
    
    # Rotate Key
    new_svc_user = client.service_users.rotate_key(svc_user.id)
    log(f"✅ Rotated Key for {svc_user.name}")
    assert new_svc_user.api_key != svc_user.api_key
    
    # 5. Business Metadata
    # Need an asset first
    wh_name = f"wh_{str(uuid.uuid4())[:8]}"
    # Ensure bucket exists
    try:
        import s3fs
        fs = s3fs.S3FileSystem(key="minioadmin", secret="minioadmin", client_kwargs={"endpoint_url": "http://localhost:9000"})
        if not fs.exists("pangolin-test-bucket"):
            fs.mkdir("pangolin-test-bucket")
    except Exception as e:
        log(f"⚠️ Bucket check skipped/failed: {e}")

    client.warehouses.create_s3(
        wh_name, 
        "pangolin-test-bucket", 
        "us-east-1", 
        "minioadmin", "minioadmin", 
        prefix="gov_demo", 
        endpoint="http://localhost:9000"
    )
    client.catalogs.create("gov_mem_cat", wh_name, "Local")
    
    # We need to register an asset to attach metadata to.
    client.catalogs.namespaces("gov_mem_cat").create(["gov_ns"])
    
    from pypangolin.assets import ParquetAsset
    import pandas as pd
    df = pd.DataFrame({"id": [1], "val": ["foo"]})
    register_response = ParquetAsset.write(
        client, "gov_mem_cat", "gov_ns", "meta_table", 
        df, 
        f"s3://pangolin-test-bucket/gov_demo/gov_mem_cat/gov_ns/meta_table.parquet",
        storage_options={"key": "minioadmin", "secret": "minioadmin", "client_kwargs": {"endpoint_url": "http://localhost:9000"}}
    )
    
    # Get asset ID from registration response
    # The API returns {"id": "...", ...}
    asset_id = register_response["id"]
    log(f"✅ Registered Asset: {asset_id}")
    
    # 6. Metadata Operations
    log("--- Testing Business Metadata ---")
    
    # Upsert properties
    props = {"contact": "data-team", "sl_level": "gold"}
    client.metadata.upsert(asset_id, tags=["verified"], properties=props, discoverable=True)
    log("✅ Added Metadata")
    
    metadata = client.metadata.get(asset_id)
    log(f"Found metadata: {metadata}")
    
    # metadata is a single BusinessMetadata object now, or list?
    # Client.get returns List[BusinessMetadata] in my original code?
    # Let's check get implementation.
    # Handler get_business_metadata returns MetadataResponse { metadata }.
    # So client.get should return BusinessMetadata.
    
    # Let's update client.get too if needed.
    # Assuming client.get returns the object.
    assert metadata.properties["contact"] == "data-team"
    assert metadata.properties["sl_level"] == "gold"
    assert "verified" in metadata.tags
    
    # 7. Access Requests
    log("--- Testing Access Requests ---")
    request = client.metadata.request_access(asset_id, "Need access for Q4 reporting")
    log(f"✅ Created Access Request: {request.id} ({request.status})")
    
    requests_list = client.metadata.list_requests()
    req_ids = [r.id for r in requests_list]
    log(f"Found requests: {req_ids}")
    assert request.id in req_ids

    log("=== Verification Complete ===") 

if __name__ == "__main__":
    main()
