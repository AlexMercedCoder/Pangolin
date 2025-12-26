import os
import subprocess
import time
import json
import uuid
import sys
from pyiceberg.catalog import load_catalog
import boto3

# Configuration
PANGOLIN_URL = "http://localhost:8080"
# Use absolute path for CLI binary
CLI_BIN = os.path.abspath("pangolin/target/debug/pangolin-admin")
MINIO_ENDPOINT = "http://localhost:9000"
AWS_ACCESS_KEY_ID = "minioadmin"
AWS_SECRET_ACCESS_KEY = "minioadmin"
BUCKET_NAME = "warehouse" 

def run_cli_command(args, env=None):
    """Runs a CLI command and returns the output."""
    full_env = os.environ.copy()
    full_env["PANGOLIN_URL"] = PANGOLIN_URL
    if env:
        full_env.update(env)
    
    cmd = [CLI_BIN] + args
    print(f"Executing: {' '.join(cmd)}")
    result = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        env=full_env
    )
    if result.returncode != 0:
        print(f"Error executing command: {result.stderr}")
        raise Exception(f"CLI command failed: {result.stderr}")
    return result.stdout.strip()

def setup_minio():
    """Ensures the bucket exists."""
    s3 = boto3.client('s3',
                      endpoint_url=MINIO_ENDPOINT,
                      aws_access_key_id=AWS_ACCESS_KEY_ID,
                      aws_secret_access_key=AWS_SECRET_ACCESS_KEY)
    try:
        s3.create_bucket(Bucket=BUCKET_NAME)
    except Exception as e:
        print(f"Bucket might already exist: {e}")

def main():
    print("Starting CLI Live Test with MinIO...")
    setup_minio()

    # 2. Login via Curl to get Token reliably
    print("Logging in as Tenant Admin via Curl...")
    login_url = f"{PANGOLIN_URL}/api/v1/users/login"
    login_payload = {
        "username": "tenant_admin",
        "password": "password123",
        "tenant-id": "00000000-0000-0000-0000-000000000000"
    }
    
    try:
        res = subprocess.run([
            "curl", "-s", "-X", "POST", login_url,
            "-H", "Content-Type: application/json",
            "-d", json.dumps(login_payload)
        ], capture_output=True, text=True, check=True)
        token_data = json.loads(res.stdout)
        ADMIN_TOKEN = token_data["token"]
        print(f"Got Admin Token: {ADMIN_TOKEN[:10]}...")
    except Exception as e:
        print(f"Login failed: {e}")
        sys.exit(1)

    # 2. Use Default Tenant (Should be set by login, but let's confirm context)
    # Since we can't 'create-tenant' as tenant_admin usually.
    # We will work within the default tenant.
    
    # 3. Create Warehouse
    wh_name = "minio_wh"
    print(f"Creating warehouse: {wh_name}")
    try:
        run_cli_command([
            "create-warehouse", wh_name, 
            "--type", "s3", 
            "--bucket", BUCKET_NAME, 
            "--access-key", AWS_ACCESS_KEY_ID, 
            "--secret-key", AWS_SECRET_ACCESS_KEY, 
            "--region", "us-east-1", 
            "--endpoint", MINIO_ENDPOINT,
            "--property", "s3.path-style-access=true"
        ])
    except Exception as e:
        print(f"Warehouse creation failed (maybe exists?): {e}")

    # 4. Create Catalog
    cat_name = "cli_catalog"
    print(f"Creating catalog: {cat_name}")
    try:
        run_cli_command(["create-catalog", cat_name, "--warehouse", wh_name])
    except Exception as e:
        print(f"Catalog creation failed: {e}")

    # 5. Create User for PyIceberg
    # As tenant_admin, create a user in *this* tenant.
    user_name = "iceberg_user"
    print(f"Creating user: {user_name}")
    try:  
        # Note: --tenant-id is optional if context is set, but better safe.
        # But we don't know the Tenant ID UUID easily without listing.
        # Unless we use `pangolin-admin list-tenants` or rely on context.
        # Let's try relying on context (omitting --tenant-id)
        run_cli_command(["create-user", user_name, "--email", "user@test.com", "--role", "tenant-user", "--password", "password"])
    except Exception as e:
        print(f"User creation failed: {e}")

    # 6. Grant Permissions
    print("Granting permissions...")
    # Grant 'write' on catalog
    run_cli_command(["grant-permission", user_name, "read,write", f"catalog:{cat_name}"])

    # 7. PyIceberg Operations
    print("\n--- PyIceberg Operations ---")

    # Create a Service User for PyIceberg
    print("Creating Service User for PyIceberg...")
    service_user_name = "pyiceberg-test-bot"

    # Use extracted ADMIN_TOKEN
    token = ADMIN_TOKEN

    # Use Curl to create service user to get raw JSON (CLI has output bug hiding ID)
    curl_cmd = [
        "curl", "-s", "-X", "POST",
        f"{PANGOLIN_URL}/api/v1/service-users",
        "-H", "Content-Type: application/json",
        "-H", f"Authorization: Bearer {token}",
        "-d", json.dumps({"name": service_user_name, "role": "tenant-admin"})
    ]
    
    try:
        print(f"Executing Curl for Service User...")
        result = subprocess.run(curl_cmd, capture_output=True, text=True, check=True)
        print(f"Curl Result: {result.stdout}")
        su_data = json.loads(result.stdout)

        client_id = su_data["id"] 
        # API returns 'api_key' in the response for creation
        client_secret = su_data["api_key"]
        print(f"Service User Created: {client_id}")
    except Exception as e:
        print(f"Failed to create service user via curl: {e}")
        #sys.exit(1)

    catalog_conf = {
        # Use Standard /v1/ prefix for Iceberg REST
        "uri": f"{PANGOLIN_URL}/v1/{cat_name}/", 
        "type": "rest",
        "s3.endpoint": MINIO_ENDPOINT,
        "s3.access-key-id": AWS_ACCESS_KEY_ID,
        "s3.secret-access-key": AWS_SECRET_ACCESS_KEY,
        # Standard OAuth2 Configuration
        "credential": f"{client_id}:{client_secret}",
        "oauth2-server-uri": f"{PANGOLIN_URL}/v1/{cat_name}/v1/oauth/tokens",
        "scope": "catalog"
    }

    catalog = load_catalog(
        cat_name,
        **catalog_conf
    )

    ns_name = "test_ns"
    if (ns_name,) not in catalog.list_namespaces():
        print(f"Creating namespace: {ns_name}")
        catalog.create_namespace(ns_name)

    tbl_name = "test_table"
    full_tbl_name = f"{ns_name}.{tbl_name}"
    
    import pyarrow as pa
    schema = pa.schema([
        ("id", pa.int64()),
        ("data", pa.string()),
    ])

    if full_tbl_name in [t[0] + "." + t[1] for t in catalog.list_tables(ns_name)]:
        print(f"Table {full_tbl_name} exists. Loading...")
        tbl = catalog.load_table(full_tbl_name)
    else:
        print(f"Creating table: {full_tbl_name}")
        tbl = catalog.create_table(full_tbl_name, schema)

    # Insert Data
    print("Appending data...")
    df = pa.Table.from_pylist([
        {"id": 1, "data": "cli-test-1"},
        {"id": 2, "data": "cli-test-2"},
    ])
    tbl.append(df)
    
    # Verify Data
    print("Reading data...")
    scan = tbl.scan()
    result_df = scan.to_arrow()
    print("Data Read:")
    print(result_df)
    assert len(result_df) >= 2

    # 8. Verify Metadata Persistence
    print("\n--- Verifying Metadata Persistence ---")
    s3 = boto3.client('s3',
                      endpoint_url=MINIO_ENDPOINT,
                      aws_access_key_id=AWS_ACCESS_KEY_ID,
                      aws_secret_access_key=AWS_SECRET_ACCESS_KEY)
    
    response = s3.list_objects_v2(Bucket=BUCKET_NAME)
    found_metadata = False
    if 'Contents' in response:
        for obj in response['Contents']:
            if obj['Key'].endswith("metadata.json"):
                print(f"Found metadata file: {obj['Key']}")
                found_metadata = True
                break
    
    if found_metadata:
        print("✅ Metadata persistence verified!")
    else:
        print("❌ Metadata file NOT found in MinIO!")
        sys.exit(1)

    print("\n✅ Live Test Completed Successfully!")

if __name__ == "__main__":
    main()
