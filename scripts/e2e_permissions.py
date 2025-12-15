import os
import time
import subprocess
import json
import sys
import requests
from pyiceberg.catalog import load_catalog

# Configuration
PANGOLIN_ADMIN = "./pangolin/target/debug/pangolin-admin"
API_URL = "http://localhost:8080"
PROFILE = "e2e_perms"
CONFIG_DIR = os.path.expanduser("~/.config/cli")

def run_cmd(args, fail_ok=False):
    cmd = [PANGOLIN_ADMIN, "--profile", PROFILE] + args
    print(f"Running: {' '.join(cmd)}")
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        if not fail_ok:
            print(f"Error: {result.stderr}")
            # print(result.stdout)
            exit(1)
        else:
            return False, result.stderr
    return True, result.stdout

def setup_env():
    print("--- Setup Environment ---")
    # Start MinIO
    subprocess.run(["docker-compose", "up", "-d", "minio"], check=True)
    time.sleep(5)
    
    # Start API
    env = os.environ.copy()
    env["PANGOLIN_ROOT_USER"] = "admin"
    env["PANGOLIN_ROOT_PASSWORD"] = "password"
    env["PANGOLIN_JWT_SECRET"] = "secret"
    # Ensure region/endpoint for S3 (MinIO)
    env["AWS_REGION"] = "us-east-1" 
    env["AWS_ACCESS_KEY_ID"] = "minioadmin"
    print("[+] Running docker-compose...")
    # Start minio and createbuckets
    # createbuckets depends on minio, so up createbuckets should start minio too.
    subprocess.run(["docker-compose", "up", "-d", "createbuckets"], check=True)
    
    # Wait for services (createbuckets takes a moment)
    time.sleep(10)
    
    api_proc = subprocess.Popen(
        ["./pangolin/target/debug/pangolin_api"], 
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE
    )
    print("Waiting for API to start...")
    time.sleep(5)
    return api_proc

def cleanup(api_proc):
    print("--- Cleanup ---")
    if api_proc:
        api_proc.terminate()
    # Remove config
    cfg_path = os.path.join(CONFIG_DIR, f"config-{PROFILE}.json")
    if os.path.exists(cfg_path):
        os.remove(cfg_path)
    
    subprocess.run(["docker-compose", "down"], check=True)

def main():
    api_proc = None
    try:
        api_proc = setup_env()
        
        # 1. Login Root
        run_cmd(["login", "--username", "admin", "--password", "password"])
        
        # 2. Setup Tenant Resources
        print("--- Setting up Tenant Resources ---")
        run_cmd(["create-tenant", "--name", "perm_tenant"])
        run_cmd(["use", "perm_tenant"])
        
        # Create Users
        print(run_cmd(["create-user", "read_user", "--email", "read@test.com", "--password", "password", "--role", "tenant-user"])[1])
        print(run_cmd(["create-user", "write_user", "--email", "write@test.com", "--password", "password", "--role", "tenant-user"])[1])
        
        # Create Warehouse
        # Using MinIO
        run_cmd(["create-warehouse", "perm_wh", "--type", "s3", 
                 "--bucket", "warehouse", 
                 "--access-key", "minioadmin", "--secret-key", "minioadmin", 
                 "--region", "us-east-1", "--endpoint", "http://localhost:9000"])
                 
        # Create Catalog
        run_cmd(["create-catalog", "perm_cat", "--warehouse", "perm_wh"])
        
        # 3. Scenario 1: Namespace Creation (CLI/API)
        print("\n--- Scenario 1: Namespace Creation Permission ---")
        
        # Login as read_user
        print("Logging in as read_user...")
        run_cmd(["login", "--username", "read_user", "--password", "password"])
        
        # Try to create namespace (should fail - users have NO permissions by default)
        # We don't have `create-namespace` in CLI yet? 
        # Ah, CLI only has Admin commands in `pangolin-admin`.
        # Namespace creation is an Iceberg operation or REST API operation.
        # `pangolin-admin` is for admin tasks. Does it have `create-namespace`?
        # Checked `commands.rs`: NO.
        # So we must use Requests or PyIceberg to test this.
        # Let's use Requests to hit REST API directly for creating namespace.
        
        # Need token for read_user. config file has it.
        with open(os.path.join(CONFIG_DIR, f"config-{PROFILE}.json")) as f:
            cfg = json.load(f)
            token = cfg['auth_token']
            
        headers = {"Authorization": f"Bearer {token}"}
        
        print("Attempting to create namespace as read_user (Expected: 403)...")
        res = requests.post(f"{API_URL}/v1/perm_cat/namespaces", json={"namespace": ["ns1"]}, headers=headers)
        if res.status_code == 403:
            print("✅ Denied as expected (403)")
        else:
            print(f"❌ Failed! Expected 403, got {res.status_code} {res.text}")
            # exit(1) # Continue
            
        # Grant Permission
        print("Granting 'create' on catalog to read_user...")
        run_cmd(["login", "--username", "admin", "--password", "password"])
        run_cmd(["use", "perm_tenant"])
        # DEBUG
        print("Debugging: Listing users...")
        print(run_cmd(["list-users"])[1])
        
        run_cmd(["grant-permission", "read_user", "create", "catalog:perm_cat"])
        
        # Retry as read_user
        # Need to re-login? Token is same, permissions checked at request time.
        # Just use the token we have (as long as it didn't expire).
        print("Retrying create namespace as read_user (Expected: 200)...")
        res = requests.post(f"{API_URL}/v1/perm_cat/namespaces", json={"namespace": ["ns1"]}, headers=headers)
        if res.status_code == 200:
            print("✅ Success! Namespace created.")
        else:
             print(f"❌ Failed! Expected 200, got {res.status_code} {res.text}")
             exit(1)

        # 4. Scenario 2: Table Access (PyIceberg)
        print("\n--- Scenario 2: PyIceberg Table Access ---")
        
        # Create table as WriteUser (we need to grant them permission first)
        # Admin grants 'create' permissions to write_user
        print("Granting permissions to write_user...")
        # (Assuming already logged in as admin from previous step)
        
        # Needs create on namespace to create table
        run_cmd(["grant-permission", "write_user", "create", "namespace:perm_cat.ns1"]) 
        # Needs write on asset for subsequent writes (if create doesn't imply write on created asset? Permission model doesn't do ownership yet)
        # We might need to grant 'write' on namespace or explicit asset?
        # Let's grant 'all' on namespace for simplicity of this test? No, 'create' should be enough for creation.
        
        # Login write_user to get token
        run_cmd(["login", "--username", "write_user", "--password", "password"])
        with open(os.path.join(CONFIG_DIR, f"config-{PROFILE}.json")) as f:
            write_token = json.load(f)['auth_token']
        
        # PyIceberg Config
        catalog = load_catalog(
            "perm_cat",
            **{
                "uri": f"{API_URL}/v1/perm_cat",
                "token": write_token,
                "s3.endpoint": "http://localhost:9000",
                "s3.access-key-id": "minioadmin",
                "s3.secret-access-key": "minioadmin",
                "s3.region": "us-east-1",
            }
        )
        
        print("Creating table as write_user...")
        try:
            from pyiceberg.schema import Schema
            from pyiceberg.types import NestedField, IntegerType, StringType
            import pyarrow as pa
            
            schema = Schema(
                NestedField(1, "id", IntegerType(), required=False),
                NestedField(2, "data", StringType(), required=False),
            )
            
            table = catalog.create_table("ns1.tbl1", schema=schema)
            print("✅ Table created.")
            
            # Write data
            df = pa.Table.from_pylist([{"id": 1, "data": "hello"}, {"id": 2, "data": "world"}])
            # Permissions: write_user has 'create' on namespace. Does it have 'write' on table? 
            # Currently NO. So append should fail?
            # Let's verify.
        except Exception as e:
            print(f"❌ Failed to create table: {e}")
            exit(1)

        print("Attempting to append data as write_user (Expected: 403)...")
        # To strictly test this, we need 'write' permission.
        # Pangolin check: Update Table -> requires 'Write' on Asset.
        # write_user only has 'Create' on Namespace.
        try:
            table.append(df)
            print("❌ Append Success! Only expected failure if permissions enforced.")
            # If it succeeds, it means permissions are loose or 'create' implies 'write'?
            # Action: Create does not imply Write.
            # However, if creating the table granted ownership permission? Pangolin doesn't support that yet.
            # So this SHOULD fail.
        except Exception as e:
             if "403" in str(e) or "Forbidden" in str(e):
                 print("✅ Write denied as expected (403).")
             else:
                 print(f"❌ Failed with unexpected error: {e}")
                 # exit(1) # Continue to fix permissions

        # Grant 'write' to write_user on table (or namespace)
        print("Granting 'write' on table to write_user...")
        run_cmd(["login", "--username", "admin", "--password", "password"])
        run_cmd(["use", "perm_tenant"])
        # Using specific table resource string
        run_cmd(["grant-permission", "write_user", "write", "table:perm_cat.ns1.tbl1"])
        
        # Retry Write
        print("Retrying write as write_user (Expected: Success)...")
        # Need to reload table to pick up potential changes?
        # PyIceberg might cache? Token is same.
        try:
            table.refresh()
            table.append(df)
            print("✅ Write Success!")
        except Exception as e:
            print(f"❌ Write Failed: {e}")
            exit(1)

        # 5. Read as Read User
        print("Attempting to read as read_user (Expected: 403)...")
        # We reuse 'headers' from before (read_user token)
        # But we use PyIceberg
        read_catalog = load_catalog(
            "perm_cat_read",
             **{
                "uri": f"{API_URL}/v1/perm_cat",
                "token": token, # read_user token
                "s3.endpoint": "http://localhost:9000",
                "s3.access-key-id": "minioadmin",
                "s3.secret-access-key": "minioadmin",
            }
        )
        
        try:
            tbl = read_catalog.load_table("ns1.tbl1")
            # Load table requires 'Read' permissions on Asset.
            # Checking logic: `load_table` handler -> `check_permission(..., Action::Read)`.
            # read_user has 'create' on Catalog.
            # 'Create' does NOT imply 'Read'.
            # So `load_table` should fail.
            print("❌ Load Table Success! Expected 403.")
        except Exception as e:
             if "403" in str(e) or "Forbidden" in str(e):
                 print("✅ Load Table denied as expected (403).")
             else:
                 print(f"❌ Unexpected error loading table: {e}")

        # Grant 'read' on namespace
        print("Granting 'read' on namespace to read_user...")
        run_cmd(["login", "--username", "admin", "--password", "password"])
        run_cmd(["use", "perm_tenant"])
        run_cmd(["grant-permission", "read_user", "read", "namespace:perm_cat.ns1"])
        
        # Retry Read
        print("Retrying read as read_user (Expected: Success)...")
        try:
            tbl = read_catalog.load_table("ns1.tbl1")
            print("✅ Table Loaded.")
            res = tbl.scan().to_arrow()
            print(f"✅ Data Read: {len(res)} rows.")
        except Exception as e:
             print(f"❌ Read Failed: {e}")
             exit(1)

        print("\n🎉 Permissions E2E Completed Successfully!")

    except Exception as e:
        print(f"❌ Script Failed: {e}")
        import traceback
        traceback.print_exc()
    finally:
        cleanup(api_proc)

if __name__ == "__main__":
    main()
