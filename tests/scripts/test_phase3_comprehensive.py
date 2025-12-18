import os
import requests
import json
import time
import base64
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, IntegerType
import pyarrow as pa

# Configuration
API_URL = "http://localhost:8080"
MINIO_URL = "http://localhost:9000"
AWS_ACCESS_KEY = "minioadmin"
AWS_SECRET_KEY = "minioadmin"

# Colors for output
class bcolors:
    HEADER = '\033[95m'
    OKBLUE = '\033[94m'
    OKCYAN = '\033[96m'
    OKGREEN = '\033[92m'
    WARNING = '\033[93m'
    FAIL = '\033[91m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'
    UNDERLINE = '\033[4m'

def print_step(msg):
    print(f"\n{bcolors.HEADER}=== {msg} ==={bcolors.ENDC}")

def print_result(msg, success=True):
    color = bcolors.OKGREEN if success else bcolors.FAIL
    symbol = "✓" if success else "✗"
    print(f"{color}{symbol} {msg}{bcolors.ENDC}")

def print_header(msg):
    print(f"\n{bcolors.BOLD}{bcolors.OKBLUE}--- {msg} ---{bcolors.ENDC}")

def get_root_auth():
    """Returns Basic Auth header for Root user"""
    # Assuming PANGOLIN_ROOT_USER=admin and PANGOLIN_ROOT_PASSWORD=password
    msg = "admin:password"
    encoded = base64.b64encode(msg.encode('ascii')).decode('ascii')
    return {"Authorization": f"Basic {encoded}"}

def create_user(username, email, password, tenant_id, role, auth_headers):
    payload = {
        "username": username,
        "email": email,
        "password": password,
        "tenant_id": tenant_id,
        "role": role
    }
    resp = requests.post(f"{API_URL}/api/v1/users", json=payload, headers=auth_headers)
    if resp.status_code in [200, 201]:
        print_result(f"Created user {username}")
        return True
    else:
        print_result(f"Failed to create user {username}: {resp.text}", False)
        return False

def login(username, password):
    payload = {
        "username": username,
        "password": password
    }
    resp = requests.post(f"{API_URL}/api/v1/users/login", json=payload)
    if resp.status_code == 200:
        return resp.json()["token"]
    print_result(f"Login failed for {username}: {resp.text}", False)
    return None

def main():
    print_header("Starting Comprehensive Live Verification (Phase 3)")

    # Root Auth for Setup
    root_headers = get_root_auth()
    
    # 1. Setup Tenant A
    print_step("Setting up Tenant A (via Root)")
    tenant_payload = {"name": "tenant_a", "properties": {}}
    resp = requests.post(f"{API_URL}/api/v1/tenants", json=tenant_payload, headers=root_headers)
    if resp.status_code in [200, 201]:
        tenant_a_id = resp.json()["id"]
        print_result(f"Created Tenant A: {tenant_a_id}")
    else:
        print_result(f"Create Tenant A failed: {resp.text}", False)
        return

    # Create Admin for Tenant A
    if not create_user("admin_a", "admin_a@example.com", "password123", tenant_a_id, "tenant-admin", root_headers):
        return

    # Login as Admin A
    admin_token = login("admin_a", "password123")
    if not admin_token:
        return
    
    headers_a = {"Authorization": f"Bearer {admin_token}", "X-Pangolin-Tenant": tenant_a_id}

    # 2. Setup Tenant B (for Federation)
    print_step("Setting up Tenant B (via Root)")
    resp = requests.post(f"{API_URL}/api/v1/tenants", json={"name": "tenant_b", "properties": {}}, headers=root_headers)
    if resp.status_code in [200, 201]:
        tenant_b_id = resp.json()["id"]
        print_result(f"Created Tenant B: {tenant_b_id}")
    else:
        print_result(f"Create Tenant B failed: {resp.text}", False)
        return

    # Create Admin for Tenant B
    if not create_user("admin_b", "admin_b@example.com", "password123", tenant_b_id, "tenant-admin", root_headers):
        return

    # Login as Admin B
    user_b_token = login("admin_b", "password123")
    if not user_b_token:
        headers_b = {} # Should not happen if return above
    else:
        headers_b = {"Authorization": f"Bearer {user_b_token}", "X-Pangolin-Tenant": tenant_b_id}
    # 3. Create Warehouse A (AwsStatic for successful operations)
    print_step("Creating Warehouse A (AwsStatic)")
    wh_payload = {
        "name": "warehouse_a",
        "storage_config": {
            "type": "s3",
            "bucket": "warehouse", 
            "region": "us-east-1",
            "endpoint": MINIO_URL,
            "access_key_id": AWS_ACCESS_KEY,
            "secret_access_key": AWS_SECRET_KEY
        },
        "vending_strategy": {
            "AwsStatic": {
                "access_key_id": AWS_ACCESS_KEY,
                "secret_access_key": AWS_SECRET_KEY
            }
        },
        "use_sts": False 
    }
    resp = requests.post(f"{API_URL}/api/v1/warehouses", json=wh_payload, headers=headers_a)
    if resp.status_code in [200, 201]:
        print_result("Created Warehouse A")
    else:
        print_result(f"Failed to create Warehouse A: Status {resp.status_code} Body: {resp.text}", False)
        return

    # 3b. Create Warehouse C (AwsSts)
    print_step("Creating Warehouse C (AwsSts)")
    # Note: For MinIO, we need a valid role or we expect a specific failure.
    # We will use a dummy role and expect Pangolin to TRY to use it.
    # If MinIO is configured to allow AssumeRole (standard in some dev modes), it might work.
    wh_c_payload = {
        "name": "warehouse_c",
        "storage_config": {
            "type": "s3",
            "bucket": "warehouse-c",
            "region": "us-east-1",
            "endpoint": MINIO_URL,
            # Pangolin needs credentials to CALL AssumeRole. We provide them in storage_config usually for S3 backend.
            # But for purely STS vending, we might expect env vars or these values.
            # Based on Pangolin logic, it builds client from storage_config params + vending config.
            "access_key_id": AWS_ACCESS_KEY,
            "secret_access_key": AWS_SECRET_KEY
        },
        "vending_strategy": {
            "AwsSts": {
                "role_arn": "arn:aws:iam::123456789012:role/MinioRole",
                "external_id": None
            }
        }
    }
    resp = requests.post(f"{API_URL}/api/v1/warehouses", json=wh_c_payload, headers=headers_a)
    if resp.status_code in [200, 201]:
        print_result("Created Warehouse C (STS)")
    else:
        # Note: Failed call might be due to validation or server check
        print_result(f"Failed to create Warehouse C: Status {resp.status_code} Body: {resp.text}", False)


    # 4. Create Catalog A (Static) and Catalog C (STS)
    print_step("Creating Catalogs")
    cat_payload = {
        "name": "catalog_a",
        "catalog_type": "Local",
        "warehouse_name": "warehouse_a",
        "storage_location": "s3://warehouse-a/catalog_a",
        "properties": {}
    }
    resp = requests.post(f"{API_URL}/api/v1/catalogs", json=cat_payload, headers=headers_a)
    if resp.status_code in [200, 201]:
        print_result("Created Catalog A (Static)")
    else:
        print_result(f"Failed to create Catalog A: {resp.text}", False)
        return

    cat_c_payload = {
        "name": "catalog_c",
        "catalog_type": "Local",
        "warehouse_name": "warehouse_c",
        "storage_location": "s3://warehouse-c/catalog_c",
        "properties": {}
    }
    resp = requests.post(f"{API_URL}/api/v1/catalogs", json=cat_c_payload, headers=headers_a)

    if resp.status_code in [200, 201]:
        print_result("Created Catalog C (STS)")
    else:
        # Not blocking if STS verification is optional/experimental
        print_result(f"Failed to create Catalog C: {resp.text}", False)

    # 5. PyIceberg Operations (Tensor A - Static)
    print_step("PyIceberg Operations (Catalog A - Static)")
    
    # Configure PyIceberg
    # We use the REST catalog pointing to Pangolin
    # Note: Pangolin uses /v1/:prefix where prefix is catalog name. Tenant is inferred from valid token.
    catalog_a = load_catalog(
        "catalog_a",
        **{
            "uri": f"{API_URL}/v1/catalog_a",
            "token": admin_token,
            "s3.endpoint": MINIO_URL,
            "py-io-impl": "pyiceberg.io.pyarrow.PyArrowFileIO",
            # No static keys here, relying on vending!
        }
    )
    
    # Configure PyIceberg for STS
    print_step("PyIceberg Operations (Catalog C - STS)")
    try:
        catalog_c = load_catalog(
            "catalog_c",
            **{
                "uri": f"{API_URL}/v1/catalog_c",
                "token": admin_token,
                "s3.endpoint": MINIO_URL,
                "py-io-impl": "pyiceberg.io.pyarrow.PyArrowFileIO",
            }
        )
        # Attempt simple operation
        catalog_c.create_namespace("db_c")
        print_result("Created Namespace 'db_c' in Catalog C (STS vending successful)")
    except Exception as e:
        print_result(f"STS Vending Verification Failed (Expected if MinIO not configured for STS): {e}", False)
        print("NOTE: MinIO STS failure often due to missing role policy in MinIO. This confirms logic was attempted.")

    # Proceed with Catalog A tests

    
    # Override client config to allow http
    os.environ["AWS_REGION"] = "us-east-1"
    # os.environ["AWS_ACCESS_KEY_ID"] = "minioadmin" # We comment these out to test VENDING.
    # But wait, if Vending works, PyIceberg S3FileIO will use vended keys.
    # If Vending fails (STS), we might need fallback.
    # Catalog A is STATIC vending. So it should work without env vars IF vending works.
    # Let's clean env to be sure.
    if "AWS_ACCESS_KEY_ID" in os.environ: del os.environ["AWS_ACCESS_KEY_ID"]
    if "AWS_SECRET_ACCESS_KEY" in os.environ: del os.environ["AWS_SECRET_ACCESS_KEY"]
    os.environ["S3_ENDPOINT_URL"] = MINIO_URL
    # Note: If we really want to test vending, we should unset these and rely on the catalog response.
    # But MinIO STS isn't real. AwsStatic vends these *exact* keys theoretically.
    # The key test is: does access work?
    
    try:
        # Create Namespace
        print("Creating namespace 'db_a'...")
        catalog_a.create_namespace("db_a")
        print_result("Created Namespace 'db_a'")
        
        # Create Table
        print("Creating table 'db_a.table_1'...")
        schema = Schema(
            NestedField(1, "id", IntegerType(), required=True),
            NestedField(2, "data", StringType(), required=False),
        )
        table = catalog_a.create_table("db_a.table_1", schema)
        print_result("Created Table 'db_a.table_1'")
        print(f"Table Location: {table.location()}")
        print(f"Table Properties: {table.properties}")
        
        # Write Data
        print("Writing data to 'db_a.table_1'...")
        df = pa.Table.from_pylist([
            {"id": 1, "data": "foo"},
            {"id": 2, "data": "bar"},
        ])
        table.append(df)
        print_result("Wrote Data to 'db_a.table_1'")
        
        # Read Data
        print("Reading data from 'db_a.table_1'...")
        read_df = table.scan().to_arrow()
        print(read_df.to_pylist())
        if len(read_df) == 2:
            print_result("Read Data Verification Success")
        else:
            print_result("Read Data Verification Failed", False)
            
        # Update Data (Append)
        print("Updating/Appending data to 'db_a.table_1'...")
        df2 = pa.Table.from_pylist([
            {"id": 3, "data": "baz"},
        ])
        table.append(df2)
        print_result("Appended Data")
        
        # Verify Update
        read_df_2 = table.scan().to_arrow()
        if len(read_df_2) == 3:
            print_result("Update Verification Success")
        
            # 8. Reverse Federation (A reads B)
            print_step("Reverse Federation: Tenant A reads Tenant B")
            
            if tenant_b_id:
                try:
                    # 1. Setup Data in Tenant B
                    # Create Catalog B in Tenant B
                    cat_b_payload = {
                        "name": "catalog_b",
                        "warehouse_name": "warehouse_a", # B shares warehouse for simplicity or allow default
                        "storage_location": f"s3://warehouse/catalog_b"
                    }
                    # We need admin_b_token for this, which is user_b_token from earlier.
                    # Let's rename user_b_token to admin_b_token for clarity in this section.
                    admin_b_token = user_b_token
                    requests.post(f"{API_URL}/api/v1/catalogs", json=cat_b_payload, headers=headers_b)
                    
                    # Setup PyIceberg for Tenant B
                    cat_b_client = load_catalog(
                        "catalog_b",
                        **{
                            "uri": f"{API_URL}/v1/catalog_b",
                            "token": admin_b_token,
                            "s3.endpoint": "http://localhost:9000",
                            "s3.access-key-id": "minioadmin",
                            "s3.secret-access-key": "minioadmin",
                        }
                    )
                    
                    # Create Namespace and Table in B
                    print("Creating Data in Tenant B...")
                    cat_b_client.create_namespace("db_b")
                    schema_b = Schema(NestedField(1, "id", IntegerType(), required=True))
                    cat_b_client.create_table("db_b.table_b", schema=schema_b)
                    print("✓ Created db_b.table_b in Tenant B")
                    
                    # 2. Federate B into A
                    # Tenant A creates "fed_b" -> pointing to Tenant B's "catalog_b"
                    fed_b_payload = {
                        "name": "fed_b",
                        "catalog_type": "Federated",
                        "federated_config": {
                            "base_url": f"{API_URL}/v1/catalog_b",
                            "auth_type": "BearerToken",
                            "credentials": {
                                "token": admin_b_token
                            }
                        }
                    }
                    resp = requests.post(f"{API_URL}/api/v1/catalogs", json=fed_b_payload, headers=headers_a)
                    if resp.status_code == 201:
                        print("✓ Created Federated Catalog 'fed_b' in Tenant A")
                    else:
                        print_result(f"Failed to create 'fed_b': {resp.text}", False)

                    # 3. Access via Tenant A
                    print("Accessing 'fed_b' as Tenant A User...")
                    fed_b_client = load_catalog(
                        "fed_b",
                        **{
                            "uri": f"{API_URL}/v1/fed_b",
                            "token": admin_token, # Tenant A Token
                            # S3 credentials still needed for data access unless vending works perfectly across federation (which is complex)
                            # For now, providing direct credentials to verify METADATA visibility first.
                            "s3.endpoint": "http://localhost:9000",
                            "s3.access-key-id": "minioadmin",
                            "s3.secret-access-key": "minioadmin",
                        }
                    )
                    
                    ns_list = fed_b_client.list_namespaces()
                    print(f"Namespaces in 'fed_b' (expecting db_b): {ns_list}")
                    
                    if ("db_b",) in ns_list or "db_b" in ns_list:
                        print_result("Tenant A successfully listed namespaces in Tenant B")
                        
                        # Check Table
                        tbls = fed_b_client.list_tables("db_b")
                        print(f"Tables in db_b: {tbls}")
                        if any("table_b" in str(t) for t in tbls):
                            print_result("Tenant A successfully listed tables in Tenant B")
                        else:
                            print_result("Tenant A failed to see table_b", False)
                    else:
                        print_result("Tenant A failed to see db_b", False)

                except Exception as e:
                    print_result(f"Reverse Federation Failed: {e}", False)
                    import traceback
                    traceback.print_exc()

            print("\n=== Phase 3 Verification Complete ===")
        else:
            print_result("Update Verification Failed", False)

        # 6. Branching
        print_step("Branching Operations")
        print("Creating branch 'dev'...")
        # PyIceberg manage_snapshots() to create branch? or direct API?
        # PyIceberg currently has limited branch creation support in high-level API, 
        # but we can try using the snapshot reference APIs if available or verify via Pangolin API.
        # Or verify using table.manage_snapshots().create_branch("dev", snapshot_id)
        try:
            print("Creating branch 'dev'...")
            snapshot_id = table.current_snapshot().snapshot_id
            # Use keyword arguments to avoid potential client-side validation bug
            table.manage_snapshots().create_branch(name="dev", snapshot_id=snapshot_id).commit()
            print("✓ Created Branch 'dev'")
            
            # Write to branch
            # table.append(df, branch="dev") -> Not fully supported in PyIceberg 0.7 yet?
            # It usually writes to main.
            # We can check if we can switch branch?
            # For this verification, simpler to just start by creating it.
            
            # Verify branch exists via Pangolin API
            b_resp = requests.get(f"{API_URL}/api/v1/branches", headers=headers_a)
            branches = b_resp.json()
            # Branch naming in Pangolin might be "catalog_name.branch_name" or just "branch_name" depending on scope?
            # Or filtering by catalog?
            # For now, just check if 'dev' is present in the list.
            if any(b['name'] == 'dev' for b in branches):
                print_result("Verified Branch 'dev' in Pangolin API")
            else:
                print_result("Branch 'dev' not found in API", False)

        except Exception as e:
            print_result(f"Branching failed: {e}", False)


    except Exception as e:
        print_result(f"PyIceberg Operations Failed: {e}", False)
        import traceback
        traceback.print_exc()

    # 7. Federated Catalog (Tenant B)
    print_step("Federated Catalog (Tenant B -> Tenant A)")
    
    # Create Tenant B and Admin B handled at start
    
    if tenant_b_id:
        # Create Federated Catalog
        fed_cat_payload = {
            "name": "federated_a",
            "catalog_type": "Federated",
            "federated_config": {
                "base_url": f"{API_URL}/v1/catalog_a",
                "auth_type": "BearerToken",
                "credentials": {
                    "token": admin_token # Giving B admin access to A for simplicity of test
                },
                "timeout_seconds": 30
            },
            "properties": {}
        }
        
        # Using generic catalog creation endpoint, explicitly specifying Federated type
        resp = requests.post(f"{API_URL}/api/v1/catalogs", json=fed_cat_payload, headers=headers_b)
        if resp.status_code in [200, 201]:
            print_result("Created Federated Catalog in Tenant B")
            
            # Verify Access via PyIceberg (Tenant B)
            print("Accessing Federated Catalog from Tenant B...")
            try:
                # For Federated Catalog, the prefix in URL must match the catalog name in Tenant B
                catalog_b = load_catalog(
                    "federated_a",
                    **{
                        "uri": f"{API_URL}/v1/federated_a",
                        "token": user_b_token,
                        "s3.endpoint": MINIO_URL,
                        # "s3.access-key-id": "minioadmin", # Should vend from Tenant A -> B? No, B proxies A.
                        # Actually, Federated catalog in B proxies requests to A. 
                        # A returns S3 creds/paths.
                        # Client B needs to access those S3 paths.
                        # So Client B needs MinIO access.
                        "s3.access-key-id": "minioadmin",
                        "s3.secret-access-key": "minioadmin",
                    }
                )
                
                # List namespaces (Should see db_a from Tenant A)
                ns = catalog_b.list_namespaces()
                print(f"Namespaces in Federated Catalog: {ns}")
                if ("db_a",) in ns or "db_a" in ns: # PyIceberg returns tuples
                    print_result("Federated List Namespaces Success")
                    
                    # Read Table
                    tbl_fed = catalog_b.load_table("db_a.table_1")
                    fed_df = tbl_fed.scan().to_arrow()
                    print(f"Federated Data rows: {len(fed_df)}")
                    if len(fed_df) == 3: # 2 original + 1 appended
                        print_result("Federated Read Data Success")
                    else:
                        print_result("Federated Read Data Failed", False)
                        
                else:
                    print_result("Federated List Namespaces Failed or Empty", False)
                    
            except Exception as e:
                 print_result(f"Federated Access Failed: {e}", False)
                 import traceback
                 traceback.print_exc()

        else:
             print_result(f"Failed to create Federated Catalog: {resp.text}", False)

    # 8. Business Metadata API
    print_step("Business Metadata API")
    # Need Asset ID of table 'db_a.table_1'
    # Use Search API
    resp = requests.get(f"{API_URL}/api/v1/assets/search?query=table_1", headers=headers_a)
    if resp.status_code == 200:
        assets = resp.json()
        target_asset = next((a for a in assets if a["name"] == "table_1"), None) # Name might be db_a.table_1? Check logic.
        # Asset names are usually name.
        if not target_asset and assets:
             target_asset = assets[0] # Grab first match
             print(f"Using found asset: {target_asset['name']}")

        if target_asset:
            asset_id = target_asset["id"]
            
            # Create Metadata
            meta_payload = {
                "description": "Sensitive customer data",
                "tags": ["pii", "gdpr"],
                "properties": {"owner": "data_team"},
                "discoverable": True
            }
            
            meta_url = f"{API_URL}/api/v1/assets/{asset_id}/metadata"
            resp = requests.post(meta_url, json=meta_payload, headers=headers_a) # POST based on lib.rs line 130

            
            if resp.status_code in [200, 201]:
                print_result("Created Business Metadata")
                
                # Get Metadata
                resp = requests.get(meta_url, headers=headers_a)
                if resp.status_code == 200:
                    data = resp.json()
                    # Response is {"metadata": {...}}
                    meta = data.get("metadata", {})
                    if meta.get("description") == "Sensitive customer data" and "pii" in meta.get("tags", []):
                        print_result("Verified Business Metadata Retrieval")
                    else:
                        print_result("Metadata Content Mismatch", False)
                else:
                    print_result("Failed to Get Metadata", False)
            else:
                 print_result(f"Failed to Create Metadata: {resp.text} (URL: {meta_url})", False)
        
        else:
            print_result("Could not find Asset ID for metadata test", False)
    else:
        print_result("Failed to list assets", False)

if __name__ == "__main__":
    main()
