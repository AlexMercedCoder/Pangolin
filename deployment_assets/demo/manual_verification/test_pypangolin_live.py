import sys
import os
import time
import uuid
import boto3
from botocore.client import Config
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, IntegerType

# Add pypangolin to path
sys.path.append(os.path.abspath("../../../pypangolin/src"))

from pypangolin import PangolinClient
from pypangolin.exceptions import NotFoundError

# MinIO Config
S3_ENDPOINT = "http://127.0.0.1:9000"
S3_ACCESS_KEY = "minioadmin"
S3_SECRET_KEY = "minioadmin"
BUCKET = "warehouse"

def check_minio_persistence(prefix):
    print(f"Checking MinIO for objects with prefix: {prefix}")
    s3 = boto3.client('s3',
                      endpoint_url=S3_ENDPOINT,
                      aws_access_key_id=S3_ACCESS_KEY,
                      aws_secret_access_key=S3_SECRET_KEY,
                      config=Config(signature_version='s3v4'),
                      region_name='us-east-1')
    
    try:
        response = s3.list_objects_v2(Bucket=BUCKET, Prefix=prefix)
        if 'Contents' in response:
            for obj in response['Contents']:
                if obj['Key'].endswith('metadata.json'):
                    print(f"SUCCESS: Found metadata.json: {obj['Key']}")
                    return True
        print("FAILURE: No metadata.json found.")
        return False
    except Exception as e:
        print(f"MinIO Check Failed: {e}")
        return False

def test_no_auth():
    print("--- Testing No-Auth Mode (Persistence) ---")
    tenant_id = "00000000-0000-0000-0000-000000000000"
    
    client = PangolinClient(uri="http://localhost:8080", tenant_id=tenant_id)
    
    unique = str(uuid.uuid4())[:6]
    wh_name = f"noauth_wh_{unique}"
    cat_name = f"noauth_cat_{unique}"
    
    # 1. Create Warehouse
    print(f"Creating warehouse '{wh_name}'...")
    warehouse = client.warehouses.create_s3(
        name=wh_name,
        bucket=BUCKET,
        endpoint=S3_ENDPOINT,
        access_key=S3_ACCESS_KEY,
        secret_key=S3_SECRET_KEY,
        **{"s3.path-style-access": "true"}
    )
    
    # 2. Create Catalog
    print(f"Creating catalog '{cat_name}'...")
    client.catalogs.create(name=cat_name, warehouse=wh_name)
    
    # 3. Use PyIceberg to create table
    print("Using PyIceberg to create table...")
    # Note: In No-Auth, we use the catalog endpoint without token
    # 3. Use PyIceberg
    # The URI for Iceberg REST catalog is http://host:port/v1/{catalog_name}
    # It maps to route /v1/:prefix/v1/config internally in Axum
    catalog_uri = f"http://localhost:8080/v1/{cat_name}"
    
    iceberg_catalog = load_catalog(
        "local",
        **{
            "type": "rest",
            "uri": catalog_uri,
            "header.X-Pangolin-Tenant": tenant_id,
        }
    )
    
    ns = "test_ns"
    try: iceberg_catalog.create_namespace(ns)
    except: pass
    
    table_name = f"{ns}.table_{unique}"
    schema = Schema(NestedField(1, "x", IntegerType(), required=True))
    
    print(f"Creating table {table_name}...")
    table = iceberg_catalog.create_table(table_name, schema=schema)
    print(f"Table created: {table.metadata_location}")

    # 4. Verify Persistence
    # The storage location defaults to s3://{bucket}/{catalog}/{namespace}/{table}
    # Example: s3://warehouse/noauth_cat_123/test_ns/table_123/metadata/...
    
    expected_prefix = f"{cat_name}/{ns}/" 
    if check_minio_persistence(expected_prefix):
        print("Test PASSED")
    else:
        print("Test FAILED")
        sys.exit(1)


def test_auth():
    print("\n--- Testing Auth Mode (Persistence) ---")
    
    unique_id = str(uuid.uuid4())[:8]
    t_name = f"auth_persist_{unique_id}"
    u_name = f"admin_{unique_id}"
    
    try:
        client = PangolinClient(uri="http://localhost:8080")
        
        # Login and Provision
        client.login("admin", "password")
        tenant = client.tenants.create(t_name)
        client.users.create(u_name, f"{u_name}@test.com", "tenant-admin", tenant_id=tenant.id, password="password123")
        client.login(u_name, "password123", tenant_id=tenant.id)
        
        # 1. Create Warehouse
        wh_name = f"auth_wh_{unique_id}"
        print(f"Creating warehouse '{wh_name}'...")
        client.warehouses.create_s3(
            name=wh_name,
            bucket=BUCKET,
            endpoint=S3_ENDPOINT,
            access_key=S3_ACCESS_KEY,
            secret_key=S3_SECRET_KEY,
            **{"s3.path-style-access": "true"}
        )
        
        # 2. Create Catalog
        cat_name = f"auth_cat_{unique_id}"
        print(f"Creating catalog '{cat_name}'...")
        client.catalogs.create(name=cat_name, warehouse=wh_name)
        
        # 3. PyIceberg
        catalog_uri = f"http://localhost:8080/v1/{cat_name}"
        iceberg_catalog = load_catalog(
            "local",
            **{
                "type": "rest",
                "uri": catalog_uri,
                "token": client.token, # Use the JWT token
            }
        )
        
        ns = "test_ns"
        try: iceberg_catalog.create_namespace(ns)
        except: pass
        
        table_name = f"{ns}.table_{unique_id}"
        schema = Schema(NestedField(1, "id", IntegerType(), required=True))
        
        print(f"Creating table {table_name}...")
        table = iceberg_catalog.create_table(table_name, schema=schema)
        
        # 4. Verify
        expected_prefix = f"{cat_name}/{ns}/"
        if check_minio_persistence(expected_prefix):
            print("Test PASSED")
        else:
            print("Test FAILED")
            sys.exit(1)

    except Exception as e:
        print(f"Auth Persistence Test Failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "auth":
        test_auth()
    else:
        test_no_auth()
