import subprocess
import time
import os
import signal
import sys
import json
import shutil

# Configuration
API_BIN = "./pangolin/target/debug/pangolin_api"
ADMIN_CLI = "./pangolin/target/debug/pangolin-admin"
USER_CLI = "./pangolin/target/debug/pangolin-user"
CONFIG_DIR = os.path.expanduser("~/.config/pangolin/cli")
SERVER_PORT = "8081" # Use different port to avoid conflicts
BASE_URL = f"http://localhost:{SERVER_PORT}"

def setup():
    # Clean config
    if os.path.exists(CONFIG_DIR):
        shutil.rmtree(CONFIG_DIR)
    print("🧹 Cleaned config directory.")

def start_server(env_vars, port):
    print(f"🚀 Starting Server with env: {env_vars}")
    # Use cargo run to ensure we run latest code
    proc = subprocess.Popen(["cargo", "run", "--bin", "pangolin_api"], env={**os.environ, **env_vars}, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) # Silence output
    
    # Wait for health check
    for _ in range(30):
        try:
            requests.get(f"http://localhost:{port}/health")
            # print("✅ Server is up!")
            return proc
        except:
            time.sleep(1)
            
    print("❌ Server failed to start")
    proc.terminate()
    sys.exit(1)

def stop_server(proc):
    print("🛑 Stopping Server...")
    proc.terminate()
    proc.wait()

def run_cli(cli_bin, args, env_vars=None, expect_fail=False):
    cmd = [cli_bin, "--url", BASE_URL] + args
    print(f"Running: {' '.join(cmd)}")
    env = os.environ.copy()
    if env_vars:
        env.update(env_vars)
    
    result = subprocess.run(cmd, env=env, capture_output=True, text=True)
    
    prefix = "✅" if result.returncode == 0 else "❌"
    
    if expect_fail and result.returncode != 0:
        print(f"✅ Expected Failure Confirmed: {result.stderr.strip()}")
        return True
    elif expect_fail and result.returncode == 0:
        print(f"❌ Expected Failure but Succeeded: {result.stdout.strip()}")
        return False
    elif not expect_fail and result.returncode != 0:
        print(f"❌ Failed: {result.stderr.strip()}")
        return False
    else:
        print(f"✅ Success: {result.stdout.strip()}")
        return True

def test_no_auth(port):
    print(f"\n--- TEST: NO-AUTH MODE (Port: {port}) ---")
    setup()
    proc = start_server({"PANGOLIN_NO_AUTH": "true"}, port)
    
    success = True
    try:
        # 1. Admin: Create Warehouse (Should work in default tenant)
        if not run_cli(ADMIN_CLI, ["create-warehouse", "wh_noauth", "--type", "memory"]): success = False
        
        # 2. Admin: Create Catalog
        if not run_cli(ADMIN_CLI, ["create-catalog", "cat_noauth", "--warehouse", "wh_noauth"]): success = False
        
        # 3. User: List Catalogs
        if not run_cli(USER_CLI, ["list-catalogs"]): success = False
        
        # 4. Root Command (Create Tenant) -> Should Fail
        if not run_cli(ADMIN_CLI, ["create-tenant", "--name", "bad_tenant"], expect_fail=True): success = False

    finally:
        stop_server(proc)
    
    return success

def test_auth(port):
    print("\n--- TEST: AUTH MODE ---")
    setup()
    env = {
         "PANGOLIN_NO_AUTH": "false",
         "PANGOLIN_ROOT_USER": "admin",
         "PANGOLIN_ROOT_PASSWORD": "password",
         "PANGOLIN_JWT_SECRET": "test_secret_key_12345",
         "PORT": str(port),
         "PANGOLIN_STORAGE_TYPE": "memory"
    }
    proc = start_server(env, port)
    
    global SERVER_PORT
    SERVER_PORT = str(port)
    
    success = True
    try:
        # 1. Unauthenticated -> Should Fail
        if not run_cli(ADMIN_CLI, ["list-tenants"], expect_fail=True): success = False
        
        # 2. Login
        if not run_cli(ADMIN_CLI, ["login", "--username", "admin", "--password", "password"]): success = False
        
        # 3. Authenticated Root -> Create Tenant
        if not run_cli(ADMIN_CLI, ["create-tenant", "--name", "tenant_a"]): success = False
        
        # 4. List Tenants -> Should show tenant_a
        if not run_cli(ADMIN_CLI, ["list-tenants"]): success = False

        # 5. Create User (Alice) in Default Tenant (Root Context??)
        # Note: Root user doesn't strictly have a tenant context unless set.
        # But let's try creating a user.        # 5. Create User (Alice)
        if not run_cli(ADMIN_CLI, ["create-user", "alice", "--email", "alice@test.com", "--role", "root", "--password", "alice_pass"]): 
            print("⚠️ Create User as Root failed or behavior undefined. Proceeding.")
        
        # 6. Login as Alice (User CLI)
        if not run_cli(USER_CLI, ["login", "--username", "alice", "--password", "alice_pass"]):
             print("❌ User Login Failed")
             success = False

    finally:
        stop_server(proc)
    
    return success

if __name__ == "__main__":
    build = subprocess.run(["cargo", "build", "--bin", "pangolin_api", "--bin", "pangolin-admin", "--bin", "pangolin-user"], cwd="pangolin")
    if build.returncode != 0:
        print("Build failed")
        sys.exit(1)

    if not test_no_auth(8081):
        print("\n❌ NO-AUTH TEST FAILED")
        sys.exit(1)
        
    if not test_auth(8082):
        print("\n❌ AUTH TEST FAILED")
        sys.exit(1)

    print("\n🎉 ALL TESTS PASSED")
