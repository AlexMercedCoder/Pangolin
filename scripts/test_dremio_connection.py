import os
import sys
import uuid
import time

# Ensure we import local pypangolin
sys.path.insert(0, os.path.abspath("pypangolin/src"))

from pypangolin import PangolinClient
from pypangolin.assets.connections import DremioAsset

def log(msg):
    print(f"[{time.strftime('%H:%M:%S')}] {msg}")

def main():
    log("=== Starting Dremio Connection Test ===")
    
    # Setup
    client = PangolinClient(uri="http://localhost:8080")
    client.login("admin", "password")
    log("✅ Logged in as Root")
    
    # Create tenant
    tenant_name = f"dremio_test_{uuid.uuid4().hex[:8]}"
    tenant = client.tenants.create(tenant_name)
    client.set_tenant(tenant.id)
    log(f"✅ Created tenant: {tenant_name}")
    
    # Create admin user
    username = f"admin_{tenant_name}"
    client.users.create(
        username=username,
        email=f"{username}@example.com",
        role="tenant-admin",
        tenant_id=tenant.id,
        password="Admin123!"
    )
    client.login(username, "Admin123!", tenant_id=tenant.id)
    log("✅ Logged in as Tenant Admin")
    
    # Create catalog for connections
    wh = client.warehouses.create_s3("dremio_wh", "s3://dremio-conn", access_key="key", secret_key="secret")
    cat = client.catalogs.create("data_sources", "dremio_wh")
    log("✅ Created catalog: data_sources")
    
    # Create namespace
    ns_client = client.catalogs.namespaces("data_sources")
    ns_client.create(["analytics"])
    log("✅ Created namespace: analytics")
    
    # Test Dremio Connection
    log("--- Testing Dremio Cloud Connection ---")
    try:
        # Dremio Cloud credentials
        dremio_token = "HkHNGrnlQAuk9kMr450BwFyiVALwAHaZGlFmnXs91gRbbPv3VVMAsRd1uS9GCQ=="
        
        # Register Dremio connection
        dremio_asset = DremioAsset.register(
            client,
            catalog="data_sources",
            namespace="analytics",
            name="dremio_cloud",
            connection_string="grpc+tls://data.dremio.cloud:443",
            credentials={
                "token": dremio_token,
                "tls": "true"
            },
            store_key=False,  # User-managed for security
            description="Dremio Cloud connection"
        )
        log(f"✅ Registered Dremio asset: {dremio_asset.get('name')}")
        
        # Verify encryption
        asset_check = DremioAsset._get_asset(client, "data_sources", "analytics", "dremio_cloud")
        if "encrypted_token" in asset_check.get('properties', {}):
            encrypted_token = asset_check['properties']['encrypted_token']
            log(f"✅ Token is encrypted: {encrypted_token[:30]}...")
            if dremio_token not in encrypted_token:
                log("✅ Plain text token not visible in properties")
        
        # Connect to Dremio (need to provide encryption key since store_key=False)
        # First, let's get the encryption key that was used
        from cryptography.fernet import Fernet
        test_key = Fernet.generate_key().decode('utf-8')
        
        # Re-register with known key for testing
        DremioAsset.register(
            client,
            catalog="data_sources",
            namespace="analytics",
            name="dremio_cloud_test",
            connection_string="grpc+tls://data.dremio.cloud:443",
            credentials={
                "token": dremio_token,
                "tls": "true"
            },
            encryption_key=test_key,
            store_key=False,
            description="Dremio Cloud test connection"
        )
        log("✅ Re-registered with known encryption key")
        
        # Now connect
        dremio_conn = DremioAsset.connect(
            client,
            catalog="data_sources",
            namespace="analytics",
            name="dremio_cloud_test",
            encryption_key=test_key
        )
        log("✅ Connected to Dremio Cloud!")
        
        # Execute test query
        log("Executing test query: SELECT * FROM dremioframe.dec9.users")
        df = dremio_conn.query("SELECT * FROM dremioframe.dec9.users")
        log(f"✅ Query successful! Retrieved {len(df)} rows")
        log(f"Columns: {list(df.columns)}")
        log(f"First row sample: {df.head(1).to_dict('records')}")
        
    except Exception as e:
        log(f"❌ Dremio test failed: {str(e)}")
        import traceback
        traceback.print_exc()
    
    # Cleanup
    log("--- Cleanup ---")
    client.catalogs.delete("data_sources")
    client.warehouses.delete("dremio_wh")
    client.login("admin", "password")
    client.tenants.delete(tenant.id)
    log("✅ Cleaned up test resources")
    
    log("=== Dremio Connection Test Complete ===")

if __name__ == "__main__":
    main()
