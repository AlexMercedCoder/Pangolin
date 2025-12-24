import os
import sys
import uuid
import time

# Ensure we import local pypangolin
sys.path.insert(0, os.path.abspath("pypangolin/src"))

from pypangolin import PangolinClient

def log(msg):
    print(f"[{time.strftime('%H:%M:%S')}] {msg}")

def main():
    log("=== Starting PyPangolin Phase 7 Verification ===")
    
    # Setup
    client = PangolinClient(uri="http://localhost:8080")
    client.login("admin", "password")
    log("✅ Logged in as Root")
    
    # Create tenant for testing
    tenant_name = f"phase7_tenant_{uuid.uuid4().hex[:8]}"
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
    
    # 1. Test Warehouse CRUD
    log("--- Testing Warehouse CRUD ---")
    wh = client.warehouses.create_s3("test_wh", "s3://test-bucket", access_key="key", secret_key="secret")
    log(f"Created warehouse: {wh.name}")
    
    # Update warehouse
    updated_wh = client.warehouses.update("test_wh", storage_config={"s3.bucket": "s3://updated-bucket"})
    log(f"Updated warehouse storage config")
    
    # 2. Test Catalog CRUD
    log("--- Testing Catalog CRUD ---")
    cat = client.catalogs.create("test_catalog", "test_wh")
    log(f"Created catalog: {cat.name}")
    
    # Update catalog
    updated_cat = client.catalogs.update("test_catalog", properties={"owner": "data-team"})
    log(f"Updated catalog properties")
    
    # 3. Test Federated Catalogs
    log("--- Testing Federated Catalogs ---")
    
    # Create federated catalog
    fed_cat = client.federated_catalogs.create(
        name="remote_catalog",
        uri="http://remote-catalog:8080",
        warehouse="remote_wh",
        properties={"region": "us-west-2"}
    )
    log(f"Created federated catalog: {fed_cat.name}")
    
    # List federated catalogs
    fed_cats = client.federated_catalogs.list()
    log(f"Listed {len(fed_cats)} federated catalogs")
    assert len(fed_cats) >= 1
    
    # Get federated catalog
    retrieved = client.federated_catalogs.get("remote_catalog")
    assert retrieved.name == "remote_catalog"
    log("✅ Retrieved federated catalog")
    
    # Test connection (will likely fail since remote doesn't exist, but tests the endpoint)
    try:
        result = client.federated_catalogs.test_connection("remote_catalog")
        log(f"Connection test result: {result.get('status')}")
    except Exception as e:
        log(f"Connection test failed as expected (remote not available): {str(e)[:50]}")
    
    # 4. Test Views
    log("--- Testing Views ---")
    
    # Create namespace for views
    ns_client = client.catalogs.namespaces("test_catalog")
    ns_client.create(["view_ns"])
    log("Created namespace for views")
    
    # Create view
    view_client = client.catalogs.views("test_catalog")
    view = view_client.create(
        namespace="view_ns",
        name="sales_summary",
        sql="SELECT * FROM sales WHERE amount > 100",
        properties={"description": "High-value sales"}
    )
    log(f"Created view: {view.name}")
    
    # Get view
    retrieved_view = view_client.get("view_ns", "sales_summary")
    assert retrieved_view.name == "sales_summary"
    assert "SELECT" in retrieved_view.sql
    log("✅ Retrieved view")
    
    # 5. Test Tenant Update
    log("--- Testing Tenant Update ---")
    client.login("admin", "password")  # Switch back to root
    updated_tenant = client.tenants.update(tenant.id, properties={"department": "engineering"})
    log("✅ Updated tenant properties")
    
    # 6. Cleanup - Delete resources
    log("--- Testing Delete Operations ---")
    client.login(username, "Admin123!", tenant_id=tenant.id)
    
    # Delete federated catalog
    client.federated_catalogs.delete("remote_catalog")
    log("✅ Deleted federated catalog")
    
    # Delete catalog
    client.catalogs.delete("test_catalog")
    log("✅ Deleted catalog")
    
    # Delete warehouse
    client.warehouses.delete("test_wh")
    log("✅ Deleted warehouse")
    
    # Delete tenant (as root)
    client.login("admin", "password")
    client.tenants.delete(tenant.id)
    log("✅ Deleted tenant")
    
    log("=== Phase 7 Verification Complete ===")

if __name__ == "__main__":
    main()
