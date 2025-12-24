import sys
import os
import uuid

# Add src to path to import pypangolin without installing if needed
sys.path.append(os.path.join(os.getcwd(), "pypangolin/src"))

from pypangolin import PangolinClient

def test_noauth(uri="http://localhost:8080"):
    print(f"Testing PyPangolin Phase 1 NO AUTH against {uri}")
    
    # 1. Init Client (No Auth headers) -> Defaults to Tenant Admin
    print("\n[1] Init Client (No Credentials)...")
    client = PangolinClient(uri)
    print("✅ Client initialized.")

    # 2. Work in Default Tenant
    # Docs say default tenant is 00000000-0000-0000-0000-000000000000
    # Operations should just work without setting context if API defaults context.
    # But client.py sets X-Pangolin-Tenant if _current_tenant_id is set.
    # If not set, it sends nothing.
    # API falls back to default tenant in No Auth mode.
    
    # 3. Create Warehouse
    print("\n[3] Creating Warehouse (as Default Admin)...")
    wh_name = f"wh_noauth_{uuid.uuid4().hex[:8]}"
    try:
        wh = client.warehouses.create_s3(
            name=wh_name,
            bucket="noauth-bucket",
            region="us-east-1",
            access_key="minio",
            secret_key="minio123"
        )
        print(f"✅ Created warehouse: {wh.name}")
    except Exception as e:
        print(f"❌ Warehouse creation failed: {e}")
        return

    # 4. Create Catalog
    print("\n[4] Creating Catalog...")
    cat_name = "noauth_catalog"
    try:
        cat = client.catalogs.create(cat_name, warehouse=wh_name, type="Local")
        print(f"✅ Created catalog: {cat.name}")
    except Exception as e:
        print(f"❌ Catalog creation failed: {e}")
        return

    # 5. Namespace
    print("\n[5] Creating Namespace...")
    try:
        ns_client = client.catalogs.namespaces(cat_name)
        ns = ns_client.create(["test", "ns"])
        print(f"✅ Created namespace: {ns.name}")
    except Exception as e:
        print(f"❌ Namespace creation failed: {e}")
        return

    # 6. Create User (as Default Admin)
    print("\n[6] Creating User...")
    user_name = f"user_{uuid.uuid4().hex[:8]}"
    try:
        # Default tenant ID
        default_tid = "00000000-0000-0000-0000-000000000000"
        user = client.users.create(
            username=user_name,
            email=f"{user_name}@example.com",
            role="tenant-user", # Try creating a regular user
            tenant_id=default_tid,
            password="any_password"
        )
        print(f"✅ Created user: {user.username}")
    except Exception as e:
        print(f"❌ User creation failed: {e}")
        return

    # 7. Login Bypass (as New User)
    print("\n[7] Login Bypass (as New User)...")
    try:
        # Login with tenant-id context
        user_client = PangolinClient(uri, user_name, "any_password_works", tenant_id=default_tid)
        print(f"✅ Login successful. Token: {user_client.token[:10]}...")
        
        # Verify context
        # Try to list catalogs (should see the one created)
        cats = user_client.catalogs.list()
        cat_names = [c.name for c in cats]
        if cat_name in cat_names:
            print(f"✅ User sees catalog: {cat_name}")
        else:
            print(f"⚠️ User sees catalogs: {cat_names} (expected {cat_name})")
            
    except Exception as e:
        print(f"❌ Login bypass failed: {e}")
        return

if __name__ == "__main__":
    uri = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:8080"
    test_noauth(uri)
