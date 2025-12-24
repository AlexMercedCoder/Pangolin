
import os
import sys
import uuid
import requests
import pandas as pd
import time
from datetime import datetime

# Ensure pypangolin is in path
sys.path.append(os.path.join(os.path.dirname(__file__), "../pypangolin/src"))
from pypangolin import PangolinClient, get_iceberg_catalog
from pypangolin.assets import (
    DeltaAsset, ParquetAsset, CsvAsset, JsonAsset, LanceAsset,
    HudiAsset, PaimonAsset, VortexAsset,
    NimbleAsset, MlModelAsset, DirectoryAsset, VideoAsset, 
    ImageAsset, DbConnectionString, OtherAsset
)

API_URL = "http://localhost:8080"
TENANT_NAME = f"gen_tenant_{str(uuid.uuid4())[:8]}"
ADMIN_USER = f"admin_{TENANT_NAME}"
PASSWORD = "password123"

def log(msg):
    print(f"[{datetime.now().strftime('%H:%M:%S')}] {msg}")

def main():
    log("=== Starting PyPangolin Generic Asset Verification ===")
    
    # 1. Setup Tenant & User (Bootstrapping)
    # We use a temporary bootstrap client or just assumption that we can create tenants open
    # For simplicity, we assume we can login as root or create tenant if env allows
    # But wait, local dev usually allows public tenant creation or we have to be admin.
    # Let's try to create a tenant directly if public signup is allowed, or login as admin first.
    # We'll assume standard flow: Login as 'admin' (root) -> Create Tenant -> Create User
    
    root_client = PangolinClient(API_URL)
    try:
        root_token = root_client.token
        if not root_token:
            # Try to login as root default
            try:
                root_client = PangolinClient(API_URL, "admin", "password")
                log("✅ Logged in as Root.")
            except Exception as e:
                log(f"⚠️ Could not login as root: {e}. Trying public tenant creation...")
    except:
        pass

    # Create Tenant
    try:
        tenant = root_client.tenants.create(TENANT_NAME)
        log(f"✅ Created Tenant: {tenant.name} ({tenant.id})")
    except Exception as e:
        log(f"❌ Failed to create tenant: {e}")
        sys.exit(1)

    # Create Tenant Admin
    try:
        user = root_client.users.create(ADMIN_USER, f"{ADMIN_USER}@example.com", "tenant-admin", tenant_id=tenant.id, password=PASSWORD)
        log(f"✅ Created Admin: {user.username}")
    except Exception as e:
        log(f"❌ Failed to create admin: {e}")
        sys.exit(1)

    # 2. Login as Tenant Admin
    client = PangolinClient(API_URL, ADMIN_USER, PASSWORD, tenant_id=tenant.id)
    log(f"✅ Logged in as {ADMIN_USER}.")

    # 3. Create Warehouse & Catalog
    wh_name = f"wh_{str(uuid.uuid4())[:8]}"
    
    # Ensure S3 Bucket Exists
    try:
        import s3fs
        fs = s3fs.S3FileSystem(
            key="minioadmin", 
            secret="minioadmin", 
            client_kwargs={"endpoint_url": "http://localhost:9000"}
        )
        try:
            fs.mkdir("pangolin-test-bucket")
            log("✅ Created S3 Bucket: pangolin-test-bucket")
        except FileExistsError:
            log("ℹ️ S3 Bucket already exists")
        except Exception as e:
            log(f"⚠️ Failed to create bucket (might exist or connection issue): {e}")
    except ImportError:
        log("⚠️ s3fs not found, skipping explicit bucket creation")

    try:
        warehouse = client.warehouses.create_s3(
            wh_name, 
            bucket="pangolin-test-bucket", 
            access_key="minioadmin", 
            secret_key="minioadmin", # MinIO defaults
            endpoint="http://localhost:9000"
        )
        log(f"✅ Created Warehouse: {warehouse.name}")
    except Exception as e:
        log(f"❌ Failed to create warehouse: {e}")
        sys.exit(1)

    cat_name = "generic_demo"
    try:
        catalog = client.catalogs.create(cat_name, warehouse.name)
        log(f"✅ Created Catalog: {cat_name}")
    except Exception as e:
        log(f"❌ Failed to create catalog: {e}")
        sys.exit(1)

    # 4. Create Namespace
    ns_name = "generic_ns"
    try:
        # We need to register namespace via client
        # client.catalogs.namespaces(cat_name).create([ns_name])
        # Wait, implementation in Phase 1 client.py:
        # def namespaces(self, catalog_name): return NamespaceClient(self.client, catalog_name)
        # NamespaceClient.create(namespace: List[str], ...)
        client.catalogs.namespaces(cat_name).create([ns_name])
        log(f"✅ Created Namespace: {ns_name}")
    except Exception as e:
        log(f"❌ Failed to create namespace: {e}")
        sys.exit(1)

    # === TEST ASSETS ===
    
    base_location = f"s3://pangolin-test-bucket/{cat_name}/{ns_name}"
    
    # 5. Delta Table
    log("--- Testing DeltaAsset ---")
    df = pd.DataFrame({"id": [1, 2], "value": ["delta", "lake"]})
    try:
        DeltaAsset.write(
            client=client, 
            catalog=cat_name, 
            namespace=ns_name, 
            name="delta_table", 
            data=df, 
            location=f"{base_location}/delta_table",
            storage_options={"AWS_ACCESS_KEY_ID": "minioadmin", "AWS_SECRET_ACCESS_KEY": "minioadmin", "AWS_ENDPOINT_URL": "http://localhost:9000", "AWS_REGION": "us-east-1", "AWS_S3_ALLOW_UNSAFE_RENAME": "true"}
        )
        log("✅ DeltaAsset Written & Registered")
    except Exception as e:
        log(f"❌ DeltaAsset Failed: {e}")

    # 6. Parquet File
    log("--- Testing ParquetAsset ---")
    try:
        ParquetAsset.write(
            client=client,
            catalog=cat_name,
            namespace=ns_name,
            name="parquet_file",
            data=df,
            location=f"{base_location}/parquet_file.parquet",
            storage_options={"key": "minioadmin", "secret": "minioadmin", "client_kwargs": {"endpoint_url": "http://localhost:9000"}}
        )
        log("✅ ParquetAsset Written & Registered")
    except Exception as e:
        log(f"❌ ParquetAsset Failed: {e}")

    # 7. CSV File
    log("--- Testing CsvAsset ---")
    try:
        CsvAsset.write(
            client=client,
            catalog=cat_name,
            namespace=ns_name,
            name="csv_file",
            data=df,
            location=f"{base_location}/csv_file.csv",
            storage_options={"key": "minioadmin", "secret": "minioadmin", "client_kwargs": {"endpoint_url": "http://localhost:9000"}}
        )
        log("✅ CsvAsset Written & Registered")
    except Exception as e:
        log(f"❌ CsvAsset Failed: {e}")
        
    # 8. Lance
    log("--- Testing LanceAsset ---")
    # LanceDB local or s3? LanceDB usually creates a folder. 
    # S3 support requires correct uri scheme.
    try:
        # Local lance for simplicity or attempt s3 generic?
        # LanceDB uses s3fs implicitly if s3://
        LanceAsset.write(
            client=client,
            catalog=cat_name,
            namespace=ns_name,
            name="lance_db",
            data=df,
            location=f"/tmp/lance_db_{uuid.uuid4()}", # Local for safety
        )
        log("✅ LanceAsset Written (Local) & Registered")
    except Exception as e:
        log(f"❌ LanceAsset Failed: {e}")

    # 9. Register Only Types (Hudi, Paimon, Vortex, etc)
    log("--- Testing Registration Only Assets ---")
    
    assets_to_register = [
        (HudiAsset, "hudi_table"),
        (PaimonAsset, "paimon_table"),
        (VortexAsset, "vortex_file"),
        (NimbleAsset, "nimble_file"),
        (MlModelAsset, "my_model"),
        (VideoAsset, "demo_video"),
        (ImageAsset, "logo_image"),
        (DbConnectionString, "postgres_conn"),
    ]
    
    for AssetCls, name in assets_to_register:
        try:
            AssetCls.register(
                client=client,
                catalog=cat_name,
                namespace=ns_name,
                name=name,
                location=f"{base_location}/{name}"
            )
            log(f"✅ {AssetCls.__name__} Registered")
        except Exception as e:
            log(f"❌ {AssetCls.__name__} Failed: {e}")

    # 10. List Assets to Verify
    log("--- Verifying Registration ---")
    try:
        assets = client.get(f"/api/v1/catalogs/{cat_name}/namespaces/{ns_name}/assets")
        # client.get returns deserialized json list of AssetSummary
        # We need to manually verify names
        names = [a['name'] for a in assets]
        log(f"Found Assets: {names}")
        
        expected = ["delta_table", "parquet_file", "csv_file", "lance_db", 
                    "hudi_table", "paimon_table", "vortex_file", "nimble_file",
                    "my_model", "demo_video", "logo_image", "postgres_conn"]
        
        missing = [ex for ex in expected if ex not in names]
        if not missing:
            log("✅ All expected assets found!")
        else:
            log(f"⚠️ Missing assets: {missing}")
            
    except Exception as e:
        log(f"❌ Failed to list assets: {e}")

if __name__ == "__main__":
    main()
