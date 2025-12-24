import sys
import os
import uuid

# Add src to path to import pypangolin without installing if needed
sys.path.append(os.path.join(os.getcwd(), "pypangolin/src"))

from pypangolin import PangolinClient
from pypangolin.exceptions import AuthenticationError

def test_phase1(uri="http://localhost:8080"):
    print(f"Testing PyPangolin Phase 1 against {uri}")
    
    # 1. Login
    print("\n[1] Testing Authentication...")
    try:
        # Default admin/password for local dev
        client = PangolinClient(uri, "admin", "password")
        print(f"✅ Login successful. Token: {client.token[:10]}...")
    except Exception as e:
        print(f"❌ Login failed: {e}")
        return

    # 2. Tenants
    print("\n[2] Testing Tenants...")
    tenant_name = f"test_tenant_{uuid.uuid4().hex[:8]}"
    try:
        tenant = client.tenants.create(tenant_name)
        print(f"✅ Created tenant: {tenant.name} ({tenant.id})")
        
        client.tenants.switch(tenant.name)
        print(f"✅ Switched to tenant: {tenant.name}")
    except Exception as e:
        print(f"❌ Tenant ops failed: {e}")
        return

    # 3. Warehouses
    print("\n[3] Testing Warehouses...")
    wh_name = f"wh_{uuid.uuid4().hex[:8]}"
    try:
        wh = client.warehouses.create_s3(
            name=wh_name,
            bucket="test-bucket",
            region="us-east-1",
            access_key="minioadmin",
            secret_key="minioadmin"
        )
        print(f"✅ Created warehouse: {wh.name}")
    except Exception as e:
        print(f"❌ Warehouse ops failed: {e}")
        return

    # 4. Catalogs
    print("\n[4] Testing Catalogs...")
    cat_name = "analytics"
    try:
        cat = client.catalogs.create(cat_name, warehouse=wh_name)
        print(f"✅ Created catalog: {cat.name} (type: {cat.type})")
        
        # 5. Namespaces
        print("\n[5] Testing Namespaces...")
        ns_client = client.catalogs.namespaces(cat_name)
        ns = ns_client.create(["sales", "q1"])
        print(f"✅ Created namespace: {ns.name}")
        
        namespaces = ns_client.list()
        print(f"✅ Listed namespaces: {namespaces}")
        
    except Exception as e:
        print(f"❌ Catalog/Namespace ops failed: {e}")
        return

if __name__ == "__main__":
    uri = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:8080"
    test_phase1(uri)
