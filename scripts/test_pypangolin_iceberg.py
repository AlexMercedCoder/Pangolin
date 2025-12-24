import sys
import os
import uuid
import time

# Add src to path to import pypangolin without installing
sys.path.append(os.path.join(os.getcwd(), "pypangolin/src"))

from pypangolin import PangolinClient, get_iceberg_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, IntegerType

def test_iceberg(uri="http://localhost:8080"):
    print(f"Testing PyPangolin Phase 2 (PyIceberg) against {uri}")
    
    # 1. Login as Root to Setup
    print("\n[1] Setup (Root)...")
    root = PangolinClient(uri, "admin", "password")
    
    # Create Tenant
    tenant_name = f"ice_tenant_{uuid.uuid4().hex[:8]}"
    tenant = root.tenants.create(tenant_name)
    print(f"✅ Created Tenant: {tenant.name}")
    
    # Create Admin
    admin_name = f"admin_{tenant_name}"
    admin_pw = "password123"
    root.users.create(admin_name, "admin@test.com", "tenant-admin", tenant.id, admin_pw)
    print(f"✅ Created Admin: {admin_name}")
    
    # 2. Login as Tenant Admin
    print("\n[2] Login as Tenant Admin...")
    client = PangolinClient(uri, admin_name, admin_pw, tenant_id=tenant.id)
    print(f"✅ Logged in. Token: {client.token[:10]}...")
    
    # 3. Create Resources
    print("\n[3] Create Warehouse & Catalog...")
    # NOTE: Using MinIO credentials. If these fail on your system, ensure MinIO is running 
    # or the credentials match your environment.
    # Docker uses: minio / minio123 usually.
    wh = client.warehouses.create_s3(
        f"wh_{uuid.uuid4().hex[:8]}",
        "warehouse", # Use the pre-created bucket
        "us-east-1",
        "minioadmin", "minioadmin", # Correct docker-compose credentials
        endpoint="http://localhost:9000",
        vending_strategy="AwsStatic"
    )
    print(f"✅ Created Warehouse: {wh.name}")
    
    cat_name = "iceberg_demo"
    client.catalogs.create(cat_name, wh.name, type="Local")
    print(f"✅ Created Catalog: {cat_name}")
    
    # 4. Initialize PyIceberg Catalog
    print("\n[4] Initializing PyIceberg Catalog...")
    # This is the key test: Does get_iceberg_catalog wire everything up correctly?
    try:
        catalog = get_iceberg_catalog(
            cat_name,
            uri=uri,
            token=client.token,
            tenant_id=tenant.id,
            # Extra properties for MinIO/S3 connectivity if not using vending or if vending needs hints
            # Since vending provides credentials, we might still need endpoint connection details for MinIO
            **{
                "s3.endpoint": "http://localhost:9000", # Accessing MinIO from Host
                # "s3.access-key-id": "minio", # Should come from vending
                # "s3.secret-access-key": "minio123" # Should come from vending
            }
        )
        print(f"✅ PyIceberg Catalog initialized: {catalog}")
    except Exception as e:
        print(f"❌ Initialization failed: {e}")
        return

    # 5. Namespace Operations
    print("\n[5] Creating Namespace...")
    try:
        ns_name = "test_ns"
        catalog.create_namespace(ns_name)
        namespaces = catalog.list_namespaces()
        print(f"✅ Namespaces: {namespaces}")
    except Exception as e:
        print(f"❌ Namespace op failed: {e}")
        return

    # 6. Table Operations
    print("\n[6] Creating Table...")
    try:
        schema = Schema(
            NestedField(1, "id", IntegerType(), required=True),
            NestedField(2, "data", StringType(), required=False),
        )
        table_name = f"{ns_name}.my_table"
        table = catalog.create_table(table_name, schema)
        print(f"✅ Created Table: {table.name()}")
        
        # Verify it exists
        tables = catalog.list_tables(ns_name)
        print(f"✅ Tables in namespace: {tables}")
    except Exception as e:
        print(f"❌ Create Table failed: {e}")
        return

    # 7. Write Data (Optional/Environment Dependent)
    # This might fail if MinIO is not reachable or pyiceberg[s3fs] is missing deps in environment
    print("\n[7] Writing Data (Experimental)...")
    try:
        import pyarrow as pa
        df = pa.Table.from_pylist([
            {"id": 1, "data": "hello"},
            {"id": 2, "data": "world"},
        ], schema=schema.as_arrow())
        table.append(df)
        print("✅ Data written successfully.")
        
        # Read back
        read_df = table.scan().to_arrow()
        print(f"✅ Data read back: {read_df.to_pylist()}")
    except ImportError:
        print("⚠️ Skipping write test: pyarrow not installed.")
    except Exception as e:
        print(f"⚠️ Write/Read failed (expected if MinIO not reachable/configured): {e}")

    # --- Part 2: Client-Side Credentials (No Warehouse/No Vending) ---
    print("\n[8] Testing Client-Side Credentials (No Warehouse)...")
    try:
        # Create Catalog without Warehouse
        cat_no_wh = "client_creds_demo"
        client.catalogs.create(cat_no_wh, warehouse="", type="Local") # warehouse="" results in None
        print(f"✅ Created Catalog (No Warehouse): {cat_no_wh}")

        # Init PyIceberg with EXPLICIT credentials (No Vending)
        catalog_manual = get_iceberg_catalog(
            cat_no_wh,
            uri=uri,
            token=client.token,
            tenant_id=tenant.id,
            credential_vending=False, # DISABLE Vending
            **{
                "s3.endpoint": "http://localhost:9000",
                "s3.access-key-id": "minioadmin",     # Client provides these
                "s3.secret-access-key": "minioadmin", # Client provides these
                "s3.region": "us-east-1"
            }
        )
        print(f"✅ PyIceberg Catalog (Manual) initialized.")

        # Create Namespace
        ns_manual = "manual_ns"
        catalog_manual.create_namespace(ns_manual)
        print(f"✅ Created Namespace (Manual): {ns_manual}")

        # Create Table (Server uses Env Vars or fails?, Client uses Props)
        # Note: If Server has no creds (no warehouse, no env vars), this metadata write will fail 
        # unless Pangolin supports 'Ad-Hoc' creds or we run Server with Env Vars.
        # We will assume Server run with AWS Env Vars for this test.
        table_name_manual = f"{ns_manual}.manual_table"
        table_manual = catalog_manual.create_table(table_name_manual, schema)
        print(f"✅ Created Table (Manual): {table_manual.name()}")
        
        # Write Data (Client uses explicit props)
        if "pyarrow" in sys.modules:
             table_manual.append(df)
             print("✅ Data written (Manual Creds).")
    except Exception as e:
        print(f"❌ Manual Creds Test Failed: {e}")

if __name__ == "__main__":
    test_iceberg()
