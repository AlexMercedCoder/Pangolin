import subprocess
import time
import sys
import json
import os

# Configuration
API_URL = "http://localhost:8080"
ROOT_USER = "admin"
ROOT_PASSWORD = "password"
CLI_BIN = "/home/alexmerced/development/personal/Personal/2026/pangolin/pangolin/target/debug/pangolin-admin"

def run_command(command, input_str=None, expect_fail=False):
    """Runs a CLI command and returns the output. Optionally expects failure."""
    print(f"Running: {command}")
    try:
        result = subprocess.run(
            command,
            input=input_str,
            text=True,
            capture_output=True,
            shell=True,
            check=False 
        )
        print("Stdout:", result.stdout) # ALWAYS PRINT
        print("Stderr:", result.stderr) # ALWAYS PRINT
        if expect_fail and result.returncode == 0:
            print(f"❌ Expected command to fail but it succeeded: {command}")
            print("Output:", result.stdout)
            sys.exit(1)
        elif not expect_fail and result.returncode != 0:
            print(f"❌ Command failed with code {result.returncode}: {command}")
            print("Stdout:", result.stdout)
            print("Stderr:", result.stderr)
            sys.exit(1)
        
        return result.stdout.strip()
    except Exception as e:
        print(f"Exception running command: {e}")
        sys.exit(1)

def setup_cli_config():
    """Ensure a clean CLI configuration."""
    config_dir = os.path.expanduser("~/.config/pangolin")
    if os.path.exists(config_dir):
        # Backup? Or just assume testing environment can nuke it?
        # Let's just create a test config file path passing --config if possible, but CLI uses standard path.
        # We will iterate assuming we can modify the standard config for this test user.
        pass

def test_root_actions():
    print("\n--- Testing Root User Actions ---")

    # 1. Login as Root
    # We need to pipe input for interactive login prompts if flags aren't supported.
    # Looking at main.rs, arguments are supported!
    # But login command reads password interactively if not provided?
    # Let's try to find if we can pass password via env or pipe.
    # The handlers.rs uses `Password::new().with_prompt...`. 
    # We can pipe it.
    
    print("1. Logging in as Root...")
    # Passing Tenant ID? No, Root login clears context.
    # We need to automate the interactive login prompts: Username, Password.
    # Or strict arguments? `pangolin-admin login` doesn't take flags in the code I saw earlier?
    # Wait, `handle_login(client, username_opt, password_opt)` takes Options.
    # Checking `main.rs` (from memory/previous view): `Login { username: Option<String> }`?
    # If main.rs supports flags, great. If not, pipe.
    # Let's try piping "admin\npassword\n".
    
    # Use flags for non-interactive login
    run_command(f"{CLI_BIN} login --username {ROOT_USER} --password {ROOT_PASSWORD}")

    # 2. Create Tenant (Allowed)
    print("2. Creating Tenant 'test_tenant' (Should Succeed)...")
    run_command(f"{CLI_BIN} create-tenant --name test_tenant")

    # 3. Create Tenant Admin (Allowed)
    print("3. Creating Tenant Admin 't_admin' (Should Succeed)...")
    # create-user <username> [options]
    # Prompts for password.
    # Need to specify tenant-id? Root creating for *which* tenant?
    # Usage: `create-user t_admin --role tenant-admin`
    # Does it ask for tenant_id? The handler `handle_create_user` has `tenant_id_opt`.
    # Does the CLI arg parsing expose it? `create-user` usually just args.
    # Let's look at `UserRole::TenantAdmin` logic in `user_handlers.rs` API side. checks `req.tenant_id`.
    # CLI handler: `tenant-id` comes from `tenant_id_opt` arg OR `client.config.tenant_id`.
    # As Root, `client.config.tenant_id` is None.
    # So we MUST provide `--tenant-id`. 
    # I need to fetch the ID of `test_tenant` first.
    
    out = run_command(f"{CLI_BIN} list-tenants")
    # Parse output to find ID of test_tenant. 
    # Output format is table.
    tenant_id = None
    for line in out.splitlines():
        if "test_tenant" in line:
            parts = line.split("|")
            # | ID | Name |
            # | ... | test_tenant |
            if len(parts) > 1:
                tenant_id = parts[1].strip()
                break
    
    if not tenant_id:
        print("❌ Could not find test_tenant ID from list-tenants output")
        print(out)
        sys.exit(1)
    
    print(f"   Found Tenant ID: {tenant_id}")
    run_command(f"{CLI_BIN} create-user t_admin --role tenant-admin --email admin@test.com --tenant-id {tenant_id} --password password123")

    # 4. Create Warehouse (Forbidden)
    print("4. Creating Warehouse as Root (Should FAIL)...")
    # This should verify the CLI check I added or the API 403.
    # Attempt: create-warehouse --name root_wh --type s3 ...
    # Pipe inputs for bucket etc.
    res = run_command(f"{CLI_BIN} create-warehouse --name root_wh --type memory", expect_fail=True)
    run_command(f"{CLI_BIN} create-warehouse root_wh --type memory", expect_fail=True)

    # 5. Create Catalog (Forbidden)
    print("5. Creating Catalog as Root (Should FAIL)...")
    res = run_command(f"{CLI_BIN} create-catalog root_cat --warehouse root_wh", expect_fail=True)

    # 6. Create Tenant User (Forbidden)
    print("6. Creating Tenant User as Root (Should FAIL)...")
    run_command(f"{CLI_BIN} create-user regular_user --role tenant-user --tenant-id {tenant_id} --email test@test.com --password password123", expect_fail=True)

    print("✅ Root User Restrictions Verified.")


def test_tenant_admin_actions():
    print("\n--- Testing Tenant Admin Actions ---")
    
    # 1. Login as Tenant Admin
    print("1. Logging in as 't_admin'...")
    run_command(f"{CLI_BIN} login --username t_admin --password password123")
    
    # 2. Create Warehouse (Allowed)
    print("2. Creating Warehouse 't_wh' (Should Succeed)...")
    run_command(f"{CLI_BIN} create-warehouse t_wh --type memory")
    
    # 3. Creating Catalog 't_cat' (Should Succeed)...")
    print("3. Creating Catalog 't_cat' (Should Succeed)...")
    run_command(f"{CLI_BIN} create-catalog t_cat --warehouse t_wh")
    
    # 4. Creating Regular User 't_user' (Should Succeed)
    print("4. Creating Regular User 't_user' (Should Succeed)...")
    run_command(f"{CLI_BIN} create-user t_user --role tenant-user --email user@test.com --password password123", input_str="userpass\n")

    # 5. Check List Users
    print("5. Listing Users...")
    out = run_command(f"{CLI_BIN} list-users")
    if "t_user" in out:
        print("✅ Found created user.")
    else:
        print("❌ User t_user not found in list.")
        sys.exit(1)

def main():
    print("Starting E2E Root Restriction Test...")
    
    # Prerequisite: MinIO buckets, Server running
    # We assume the caller handles server lifecycle, or we can add check.
    
    try:
        test_root_actions()
        test_tenant_admin_actions()
        print("\n✅✅✅ ALL RESTRICTIONS VERIFIED SUCCESSFULLY ✅✅✅")
    except Exception as e:
        print(f"\n❌ Test Suite Failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
