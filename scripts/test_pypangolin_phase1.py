import sys
import os
import uuid

# Add src to path to import pypangolin without installing if needed
sys.path.append(os.path.join(os.getcwd(), "pypangolin/src"))

from pypangolin import PangolinClient
from pypangolin.exceptions import AuthenticationError

def test_phase1(uri="http://localhost:8080"):
    print(f"Testing PyPangolin Phase 1 against {uri}")
    
    # 1. Login as Root
    print("\n[1] Login as Root...")
    try:
        root_client = PangolinClient(uri, "admin", "password")
        print(f"✅ Login successful. Token: {root_client.token[:10]}...")
    except Exception as e:
        print(f"❌ Login failed: {e}")
        return

    # 2. Key Step: Create Tenant
    print("\n[2] Creating Tenant...")
    tenant_name = f"test_tenant_{uuid.uuid4().hex[:8]}"
    try:
        tenant = root_client.tenants.create(tenant_name)
        print(f"✅ Created tenant: {tenant.name} ({tenant.id})")
    except Exception as e:
        print(f"❌ Tenant creation failed: {e}")
        return

    # 3. Create Tenant Admin User
    print("\n[3] Creating Tenant Admin User...")
    user_name = f"admin_{tenant_name}"
    user_email = f"admin@{tenant_name}.com"
    user_pw = "tenant_password"
    try:
        user = root_client.users.create(
            username=user_name,
            email=user_email,
            role="tenant-admin",
            tenant_id=tenant.id,
            password=user_pw
        )
        print(f"✅ Created user: {user.username} (Role: {user.role})")
        
        # Verify user exists via list (must switch context to tenant, or filtered list)
        print(f"   [Debug] Switching Root context to tenant {tenant.id} to list users...")
        root_client.set_tenant(tenant.id)
        all_users = root_client.users.list()
        target_user = next((u for u in all_users if u.username == user_name), None)
        if target_user:
            print(f"✅ User found in list: {target_user.username}, Tenant: {target_user.tenant_id}")
        else:
            print(f"❌ User NOT found in list for tenant {tenant.id}!")
            print(f"   Users found: {[u.username for u in all_users]}")
        
    except Exception as e:
        print(f"❌ User creation failed: {e}")
        return

    # 4. Login as Tenant Admin
    print("\n[4] Login as Tenant Admin...")
    try:
        client = PangolinClient(uri, user_name, user_pw, tenant_id=tenant.id)
        print(f"✅ Login successful as Tenant Admin.")
        # Implicitly sets tenant context due to constructor
        print(f"✅ Context set to tenant: {tenant.id}")
    except Exception as e:
        print(f"❌ User login failed: {e}")
        return

    # 5. Warehouses (as Tenant Admin)
    print("\n[5] Creating Warehouse...")
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

    # 6. Catalogs
    print("\n[6] Creating Catalog...")
    cat_name = "analytics"
    try:
        cat = client.catalogs.create(cat_name, warehouse=wh_name)
        print(f"✅ Created catalog: {cat.name}")
        
        # 7. Namespaces
        print("\n[7] Creating Namespace...")
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
