import os
import subprocess
import time
import requests
import sys
import json
from pyiceberg.catalog import load_catalog
import pyarrow as pa
import jwt
import datetime

# Constants
PANGOLIN_PORT = 8080
MINIO_ENDPOINT = "http://localhost:9000"
AWS_ACCESS_KEY = "minioadmin"
AWS_SECRET_KEY = "minioadmin"
WAREHOUSE_LOCATION = "s3://warehouse/"

# Colors for output
GREEN = '\033[92m'
RED = '\033[91m'
RESET = '\033[0m'
BLUE = '\033[94m'

def log(msg):
    print(f"{BLUE}[E2E] {msg}{RESET}")

def success(msg):
    print(f"{GREEN}✓ {msg}{RESET}")

def fail(msg):
    print(f"{RED}✗ {msg}{RESET}")
    sys.exit(1)

def run_command(cmd, env=None):
    log(f"Running: {cmd}")
    full_env = os.environ.copy()
    if env:
        full_env.update(env)
    
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True, env=full_env)
    if result.returncode != 0:
        fail(f"Command failed: {cmd}\nOutput: {result.stdout}\nError: {result.stderr}")
    return result.stdout.strip()

def check_server_health():
    try:
        r = requests.get(f"http://localhost:{PANGOLIN_PORT}/health")
        return r.status_code == 200
    except:
        return False

def wait_for_server():
    log("Waiting for Pangolin server...")
    for i in range(30):
        if check_server_health():
            success("Server is up!")
            return
        time.sleep(1)
    fail("Server failed to start")

def generate_jwt(secret):
    payload = {
        "sub": "00000000-0000-0000-0000-000000000000", # Root ID proxy
        "username": "admin",
        "role": "root",
        "exp": datetime.datetime.utcnow() + datetime.timedelta(hours=1),
        "iat": datetime.datetime.utcnow()
    }
    return jwt.encode(payload, secret, algorithm="HS256")

def main():
    # 1. Setup Infrastructure
    log("Starting MinIO...")
    run_command("docker-compose up -d")
    time.sleep(5) # Wait for MinIO to be fully ready and buckets created

    # 2. Build and Start Pangolin Server
    log("Building Pangolin...")
    run_command("cd pangolin && cargo build")
    
    server_process = subprocess.Popen(
        ["./pangolin/target/debug/pangolin_api"],
        env={
            **os.environ,
            "PANGOLIN_ROOT_USER": "admin",
            "PANGOLIN_ROOT_PASSWORD": "password",
            "PANGOLIN_JWT_SECRET": "mysecretkey12345",
            "PORT": str(PANGOLIN_PORT),
            "PANGOLIN_STORAGE_TYPE": "memory", # Admin config stored in memory for this test, but data in S3
            "AWS_REGION": "us-east-1",
            "S3_ENDPOINT": MINIO_ENDPOINT,
            "AWS_ACCESS_KEY_ID": AWS_ACCESS_KEY,
            "AWS_SECRET_ACCESS_KEY": AWS_SECRET_KEY,
            "AWS_ALLOW_HTTP": "true" # Required for MinIO
        }
    )
    
    try:
        wait_for_server()
        
        # 3. Admin CLI Setup
        log("Configuring Pangolin via CLI...")
        
        # Login
        run_command(f"./pangolin/target/debug/pangolin-admin --url http://localhost:8080 --profile e2e login -u admin -p password")
        success("Logged in")

        # Create Tenant (No auth needed after login, but needs profile)
        run_command("./pangolin/target/debug/pangolin-admin --profile e2e create-tenant --name end2end_tenant")
        success("Tenant created")
        
        # Create User
        # CLI: create-user <USERNAME> --email ... --role ... --password ... --tenant-id ...
        # Problem: We need Tenant ID. `list-tenants` gives output.
        # Let's try to pass name if supported? No, struct says `tenant_id`.
        # We need to fetch tenant ID.
        
        # List Tenants to get ID
        tenants_output = run_command("./pangolin/target/debug/pangolin-admin --profile e2e list-tenants")
        # Output format:
        # +--------------------------------------+----------------+
        # | ID                                   | Name           |
        # +--------------------------------------+----------------+
        # | 00000000-0000-0000-0000-000000000000 | default        |
        # | <UUID>                               | end2end_tenant |
        # ...
        
        tenant_id = None
        for line in tenants_output.splitlines():
            if "end2end_tenant" in line:
                parts = [p.strip() for p in line.split("|")]
                if len(parts) > 1:
                    tenant_id = parts[1] # The first empty string is before |
                    # Wait, splitting "| ID | Name |" gives ["", " ID ", " Name ", ""]
                    # So index 1 is ID.
                    tenant_id = parts[1]
                    break
        
        if not tenant_id:
            fail("Could not find tenant ID for end2end_tenant")
        log(f"Resolved Tenant ID: {tenant_id}")

        run_command(f"./pangolin/target/debug/pangolin-admin --profile e2e create-user pyiceberg_user --email py@test.com --role tenant-user --password password --tenant-id {tenant_id}")
        success("User created")
        
        # Create Warehouse (S3)
        # Using new args: --bucket --access-key --secret-key --region --endpoint
        # IMPORTANT: 'warehouse_location' is "s3://warehouse/". We need just bucket name "warehouse".
        run_command(f"./pangolin/target/debug/pangolin-admin --profile e2e create-warehouse s3_warehouse --type s3 --bucket warehouse --access-key {AWS_ACCESS_KEY} --secret-key {AWS_SECRET_KEY} --region us-east-1 --endpoint {MINIO_ENDPOINT}")
        success("Warehouse created")
        
        # Create Catalog
        run_command(f"./pangolin/target/debug/pangolin-admin --profile e2e --tenant {tenant_id} create-catalog dev_catalog --warehouse s3_warehouse")
        success("Catalog created")
        
        # 4. PyIceberg Operations
        log("Testing PyIceberg Operations...")
        
        # Generate Token
        token = generate_jwt("mysecretkey12345")
        log(f"Generated Token for PyIceberg: {token}")

        # Configure PyIceberg Catalog
        catalog = load_catalog(
            "pangolin",
            **{
                "type": "rest",
                "uri": f"http://localhost:{PANGOLIN_PORT}/v1/dev_catalog",
                "header.X-Pangolin-Tenant": tenant_id,
                "token": token,
                "s3.endpoint": MINIO_ENDPOINT,
                "s3.access-key-id": AWS_ACCESS_KEY,
                "s3.secret-access-key": AWS_SECRET_KEY,
                "s3.region": "us-east-1",
            }
        )
        
        # Create Namespace
        log("Creating Namespace...")
        catalog.create_namespace("hr_dept")
        success("Namespace 'hr_dept' created")
        
        # Create Table
        log("Creating Table...")
        schema = pa.schema([
            pa.field("id", pa.int64()),
            pa.field("name", pa.string()),
            pa.field("role", pa.string()),
        ])
        
        table = catalog.create_table(
            "hr_dept.employees",
            schema=schema,
        )
        success("Table 'hr_dept.employees' created")
        
        # Append Data (V1)
        log("Appending Data (V1)...")
        data_v1 = pa.Table.from_pylist([
            {"id": 1, "name": "Alice", "role": "Engineer"},
            {"id": 2, "name": "Bob", "role": "Designer"},
        ], schema=schema)
        table.append(data_v1)
        success("Data V1 appended")
        
        # Verify V1 Data
        df = table.scan().to_arrow().to_pylist()
        assert len(df) == 2, f"Expected 2 rows, got {len(df)}"
        success("Verified V1 Data")
        
        # 5. Branching Workflow
        log("Starting Branching Workflow...")
        
        # Create Branch via CLI (as PyIceberg doesn't support generic REST branching endpoints likely?)
        # Pangolin has custom endpoints /api/v1/branches
        # Admin CLI should have `create-branch` command?
        # Let's check `pangolin-admin --help` or source.
        # ... Wait, I added `create-branch` to `pangolin-user` CLI usually, or is it in `admin`?
        # The user request asks to "use cli for admin tasks".
        # Let's use `curl` for the branching API if CLI doesn't support it yet, or assume it's there.
        # Actually I worked on `user-branches` documentation, so it's likely in `pangolin-user`.
        # Admin CLI manages infra. User CLI manages data/branches.
        # Let's shell out to `curl` for simplicity or use `requests` here since this IS a python script.
        
        log("Creating Branch 'dev_feature'...")
        r = requests.post(
            f"http://localhost:{PANGOLIN_PORT}/api/v1/branches",
            headers={
                "X-Pangolin-Tenant": tenant_id,
                "Authorization": "Basic YWRtaW46cGFzc3dvcmQ="
            },
            json={
                "name": "dev_feature",
                "from_branch": "main",
                "catalog": "dev_catalog",
                "assets": ["hr_dept.employees"]
            }
        )
        if r.status_code != 201:
            fail(f"Failed to create branch: {r.text}")
        success("Branch 'dev_feature' created")
        
        # 6. Modify Data on Branch
        log("Modifying Data on Branch 'dev_feature'...")
        
        # Load table with 'branch' query param or name?
        # PyIceberg supports `ref` in table name? e.g. "db.table@branch"
        # Pangolin supports this parsing!
        
        table_dev = catalog.load_table("hr_dept.employees@dev_feature")
        
        # Append Data (V2)
        data_v2 = pa.Table.from_pylist([
            {"id": 3, "name": "Charlie", "role": "Manager"},
        ], schema=schema)
        table_dev.append(data_v2)
        success("Data V2 appended to dev_feature")
        
        # 7. Verify Isolation
        log("Verifying Isolation...")
        
        # Check Main (Should have 2 rows)
        table_main = catalog.load_table("hr_dept.employees") # Default is main
        rows_main = table_main.scan().to_arrow().to_pylist()
        assert len(rows_main) == 2, f"Main branch isolation failed! Expected 2 rows, got {len(rows_main)}"
        success("Main branch isolated (2 rows)")
        
        # Check Dev (Should have 3 rows)
        # Note: PyIceberg append on a branch usually creates a NEW snapshot on that branch.
        # Re-load to be sure we see latest
        table_dev = catalog.load_table("hr_dept.employees@dev_feature")
        rows_dev = table_dev.scan().to_arrow().to_pylist()
        assert len(rows_dev) == 3, f"Dev branch update failed! Expected 3 rows, got {len(rows_dev)}"
        success("Dev branch updated (3 rows)")
        
        # 8. Merge
        log("Merging 'dev_feature' into 'main'...")
        r = requests.post(
            f"http://localhost:{PANGOLIN_PORT}/api/v1/branches/merge",
            headers={
                "X-Pangolin-Tenant": tenant_id,
                "Authorization": "Basic YWRtaW46cGFzc3dvcmQ="
            },
            json={
                "source_branch": "dev_feature",
                "target_branch": "main",
                "catalog": "dev_catalog"
            }
        )
        if r.status_code != 200:
            fail(f"Merge failed: {r.text}")
        success("Merge successful")
        
        # 9. Final Verification
        log("Verifying Merge Result...")
        table_main = catalog.load_table("hr_dept.employees") # Reload main
        rows_main_final = table_main.scan().to_arrow().to_pylist()
        assert len(rows_main_final) == 3, f"Merge verification failed! Expected 3 rows, got {len(rows_main_final)}"
        success("Main branch now has 3 rows")
        
        success("🎉 E2E TEST PASSED! 🎉")

    finally:
         log("Cleaning up...")
         server_process.kill()
         run_command("docker-compose down")

if __name__ == "__main__":
    main()
