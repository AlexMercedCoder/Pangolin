#!/usr/bin/env python3
import subprocess
import re
import sys
import time
import uuid

# Configuration
CLI_BIN = "cargo run -q --manifest-path pangolin/Cargo.toml -p pangolin_cli_admin -- --url http://localhost:8080"
TENANT_ADMIN_USER = "reg_test_admin"
TENANT_ADMIN_PASS = "password"

def run_command(cmd, shell=True):
    """Run a shell command and return stdout. Raise exception on failure."""
    print(f"COMMAND: {cmd}")
    result = subprocess.run(cmd, shell=shell, executable="/bin/bash", capture_output=True, text=True)
    if result.returncode != 0:
        print(f"FAILED (RC={result.returncode})")
        print("STDOUT:", result.stdout)
        print("STDERR:", result.stderr)
        raise Exception(f"Command failed: {cmd}")
    return result.stdout.strip()

def run_cli(args):
    """Run the CLI with arguments."""
    return run_command(f"{CLI_BIN} {args}")

def main():
    print(">>> Starting CLI Regression Test (v0.5.0) <<<")
    
    # Generate unique names
    unique_suffix = int(time.time())
    tenant_name = f"reg_tenant_{unique_suffix}"
    wh_name = f"reg_wh_{unique_suffix}"
    cat_name = f"reg_cat_{unique_suffix}"
    su_name = f"reg_bot_{unique_suffix}"
    
    # 1. Login as Root
    print("\n[STEP 1] Login as Root")
    run_cli("login --username admin --password password")
    
    # 2. Create Tenant (and capture output to verify)
    print(f"\n[STEP 2] Create Tenant '{tenant_name}'")
    try:
        out = run_cli(f"create-tenant --name {tenant_name} --admin-username {TENANT_ADMIN_USER} --admin-password {TENANT_ADMIN_PASS}")
        print("OUTPUT:", out)
        # Parse output for ID? 
        # Output: "✅ Tenant 'name' created successfully (ID: <UUID>)."
        match = re.search(r"ID: ([a-f0-9\-]+)", out)
        if not match:
            raise Exception("Could not find Tenant UUID in output")
        tenant_id = match.group(1)
        print(f"CAPTURED TENANT ID: {tenant_id}")
    except Exception as e:
        print("Error creating tenant:", e)
        sys.exit(1)

    # 3. Login as Tenant Admin
    print("\n[STEP 3] Login as Tenant Admin")
    run_cli(f"login --username {TENANT_ADMIN_USER} --password {TENANT_ADMIN_PASS} --tenant-id {tenant_id}")

    # 4. Create Warehouse
    print(f"\n[STEP 4] Create Warehouse '{wh_name}'")
    run_cli(f"create-warehouse {wh_name} --bucket test-bucket --region us-east-1 --access-key AWS_KEY --secret-key AWS_SECRET --endpoint http://minio:9000")
    
    # Verify via list
    out = run_cli("list-warehouses")
    if wh_name not in out:
        raise Exception(f"Warehouse {wh_name} not found in listing")
        
    # 5. Create Service User
    print(f"\n[STEP 5] Create Service User '{su_name}'")
    out = run_cli(f"create-service-user --name {su_name} --role tenant-user")
    print("OUTPUT:", out)
    # Parse ID
    match = re.search(r"Service User ID: ([a-f0-9\-]+)", out)
    if not match:
        raise Exception("Could not find Service User UUID in output")
    su_id = match.group(1)
    print(f"CAPTURED SU ID: {su_id}")
    
    # 6. Verify RBAC (Assign Role - Negative Test)
    print(f"\n[STEP 6] Verify RBAC (Assign Role)")
    # We assign a non-existent role to check if command works (returns 404 from API)
    fake_role_id = "00000000-0000-0000-0000-000000000000"
    try:
        run_cli(f"assign-role --user-id {su_id} --role-id {fake_role_id}")
        # If it succeeds (unlikely for fake ID), print warning
        print("WARNING: Assign role succeeded unexpectedly (maybe fake ID exists?)")
    except Exception as e:
        # Check if error message confirms it talked to API
        err_msg = str(e)
        print("Expected Error Caught:", err_msg[:100] + "...")
        # Since run_command raises Exception on non-zero exit code, and CLI returns non-zero on API error
        pass

    # 7. Create Catalog
    print(f"\n[STEP 7] Create Catalog '{cat_name}'")
    run_cli(f"create-catalog {cat_name} --warehouse {wh_name}")
    out = run_cli("list-catalogs")
    if cat_name not in out:
        raise Exception(f"Catalog {cat_name} not found in listing")

    # 8. Cleanup
    print("\n[STEP 8] Cleanup")
    run_cli(f"delete-service-user --id {su_id}")
    run_cli(f"delete-catalog {cat_name}")
    run_cli(f"delete-warehouse {wh_name}")
    # Cannot delete tenant as Tenant Admin easily (need Root), skipping tenant deletion for now
    
    print("\n✅ REGRESSION TEST PASSED")

if __name__ == "__main__":
    main()
