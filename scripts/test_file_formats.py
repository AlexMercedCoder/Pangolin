import os
import sys
import uuid
import time
import pandas as pd

# Ensure we import local pypangolin
sys.path.insert(0, os.path.abspath("pypangolin/src"))

from pypangolin import PangolinClient
from pypangolin.assets import CsvAsset, JsonAsset, LanceAsset

def log(msg):
    print(f"[{time.strftime('%H:%M:%S')}] {msg}")

def main():
    log("=== Starting File Format Tests (CSV, JSON, Lance) ===")
    
    # Setup
    client = PangolinClient(uri="http://localhost:8080")
    client.login("admin", "password")
    log("✅ Logged in as Root")
    
    # Create tenant
    tenant_name = f"format_test_{uuid.uuid4().hex[:8]}"
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
    
    # Create warehouse and catalog
    wh = client.warehouses.create_s3(
        "format_wh",
        "s3://test-bucket",
        access_key="minioadmin",
        secret_key="minioadmin"
    )
    cat = client.catalogs.create("formats", "format_wh")
    log("✅ Created catalog: formats")
    
    # Create namespace
    ns_client = client.catalogs.namespaces("formats")
    ns_client.create(["files"])
    log("✅ Created namespace: files")
    
    # Create test data
    test_data = pd.DataFrame({
        'id': [1, 2, 3, 4, 5],
        'name': ['Alice', 'Bob', 'Charlie', 'Diana', 'Eve'],
        'age': [25, 30, 35, 28, 32],
        'city': ['New York', 'London', 'Paris', 'Tokyo', 'Berlin']
    })
    
    # Test 1: CSV Format
    log("--- Testing CSV Format ---")
    try:
        # Write CSV (local filesystem)
        csv_location = "/tmp/test_csv/data.csv"
        csv_asset = CsvAsset.write(
            client,
            catalog="formats",
            namespace="files",
            name="test_csv",
            data=test_data,
            location=csv_location
        )
        log(f"✅ Wrote CSV asset: {csv_asset.get('name')}")
        
        # Read CSV
        df_read = CsvAsset.read(client, "formats", "files", "test_csv")
        log(f"✅ Read CSV: {len(df_read)} rows, columns: {list(df_read.columns)}")
        
        # Verify data integrity
        if len(df_read) == len(test_data) and list(df_read.columns) == list(test_data.columns):
            log("✅ CSV data integrity verified")
        else:
            log("❌ CSV data mismatch")
            
    except Exception as e:
        log(f"❌ CSV test failed: {str(e)}")
    
    # Test 2: JSON Format
    log("--- Testing JSON Format ---")
    try:
        # Write JSON (local filesystem)
        json_location = "/tmp/test_json/data.json"
        json_asset = JsonAsset.write(
            client,
            catalog="formats",
            namespace="files",
            name="test_json",
            data=test_data,
            location=json_location
        )
        log(f"✅ Wrote JSON asset: {json_asset.get('name')}")
        
        # Read JSON
        df_read = JsonAsset.read(client, "formats", "files", "test_json")
        log(f"✅ Read JSON: {len(df_read)} rows, columns: {list(df_read.columns)}")
        
        # Verify data integrity
        if len(df_read) == len(test_data):
            log("✅ JSON data integrity verified")
        else:
            log("❌ JSON data mismatch")
            
    except Exception as e:
        log(f"❌ JSON test failed: {str(e)}")
    
    # Test 3: Lance Format
    log("--- Testing Lance Format ---")
    try:
        # Write Lance (uses local filesystem for lancedb)
        lance_location = "/tmp/test_lance"
        lance_asset = LanceAsset.write(
            client,
            catalog="formats",
            namespace="files",
            name="test_lance",
            data=test_data,
            location=lance_location
        )
        log(f"✅ Wrote Lance asset: {lance_asset.get('name')}")
        
        # Read Lance
        df_read = LanceAsset.read(client, "formats", "files", "test_lance")
        log(f"✅ Read Lance: {len(df_read)} rows, columns: {list(df_read.columns)}")
        
        # Verify data integrity
        if len(df_read) == len(test_data):
            log("✅ Lance data integrity verified")
        else:
            log("❌ Lance data mismatch")
            
    except Exception as e:
        log(f"❌ Lance test failed: {str(e)}")
    
    # Cleanup
    log("--- Cleanup ---")
    client.catalogs.delete("formats")
    client.warehouses.delete("format_wh")
    client.login("admin", "password")
    client.tenants.delete(tenant.id)
    log("✅ Cleaned up test resources")
    
    log("=== File Format Tests Complete ===")

if __name__ == "__main__":
    main()
