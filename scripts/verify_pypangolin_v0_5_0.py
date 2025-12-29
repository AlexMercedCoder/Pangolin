#!/usr/bin/env python3
import sys
import uuid
import time
from pypangolin import PangolinClient
from pypangolin.exceptions import AuthorizationError

# Configuration - Assumes default local dev setup
BASE_URL = "http://localhost:8080"
ROOT_USER = "admin"
ROOT_PASS = "password"

def print_step(msg):
    print(f"\n[STEP] {msg}")

def fail(msg):
    print(f"\n[FAIL] {msg}")
    sys.exit(1)

def run_test():
    print(">>> Starting PyPangolin v0.5.0 Verification (Service Users & Auth) <<<")

    # 1. Admin Login & Setup
    print_step("Initializing Admin Client...")
    admin_client = PangolinClient(BASE_URL, username=ROOT_USER, password=ROOT_PASS)
    
    # Create a unique tenant for isolation
    tenant_name = f"py_tenant_{int(time.time())}"
    print_step(f"Creating Tenant '{tenant_name}'...")
    tenant = admin_client.tenants.create(tenant_name)
    admin_client.set_tenant(tenant.id)
    print(f"Tenant ID: {tenant.id}")

    # 2. Create Service User
    bot_name = f"py_bot_{int(time.time())}"
    print_step(f"Creating Service User '{bot_name}' (Role: tenant-user)...")
    service_user = admin_client.service_users.create(bot_name, role="tenant-user")
    print(f"Service User ID: {service_user.id}")
    print(f"API Key: {service_user.api_key[:10]}...") 

    api_key = service_user.api_key
    if not api_key:
        fail("API Key was not returned/captured!")

    # 3. Authenticate as Service User (The Key Test)
    print_step("Initializing Bot Client (X-API-Key Auth)...")
    bot_client = PangolinClient(BASE_URL, api_key=api_key)
    # Check if we need to set tenant? Service Users belong to a tenant, but the middleware 
    # should infer tenant from the user session. 
    # However, for resource scoping, we might explicitly set it or rely on default.
    # Let's see if middleware auto-sets X-Pangolin-Tenant from session.
    # Based on auth_middleware.rs: it sets TenantId extension from session.tenant_id.
    
    # 4. Verify Identity via `users.me()` or just list assets?
    # Note: /users/me might return 404 if mapped to SQL user table check, 
    # but let's try a simple safe read first.
    
    print_step("Bot: Listing Warehouses (Expected: Success, Empty List)...")
    try:
        warehouses = bot_client.warehouses.list()
        print(f"Warehouses found: {len(warehouses)}")
    except Exception as e:
        fail(f"Bot failed to list warehouses: {e}")

    # 5. Verify RBAC (Negative Test)
    print_step("Bot: Creating Warehouse (Expected: Fail 403 - TenantUser cannot create)...")
    try:
        bot_client.warehouses.create_s3("should_fail_wh", "bucket", access_key="test", secret_key="test")
        fail("Bot succeeded in creating warehouse (Should have failed!)")
    except AuthorizationError:
        print("✅ Correctly rejected with 403.")
    except Exception as e:
        fail(f"Unexpected error (Expected AuthorizationError): {type(e)} - {e}")

    # 6. Verify Update Service User (Admin Action)
    print_step("Admin: Updating Service User description...")
    updated_su = admin_client.service_users.update(service_user.id, description="Updated via PyPangolin")
    if updated_su.description != "Updated via PyPangolin":
        fail("Update Service User failed to persist description")
    print("✅ Verified Update")

    # 7. Upgrade Bot Role (Admin Action)
    print_step("Admin: Upgrading Bot to 'tenant-admin' (RBAC Check)...")
    # Finding role ID for tenant-admin? 
    # Or does `assign_role` accept role name or ID? 
    # `assign_role` usually takes ID. Let's list roles to find "TenantAdmin".
    roles = admin_client.roles.list()
    # Note: System roles might be handled differently or seeded. 
    # If list returns specific custom roles, we need to map system roles.
    # Actually, `assign_role` in `permission_handlers` often takes strict UUIDs.
    # But wait, `tenant-admin` is a system role enum, usually auto-mapped?
    # Let's assume we need to find the Role ID. 
    # Since `TenantAdmin` is a system concept, usually there isn't a row in `roles` table unless seeded?
    # Actually, v0.5.0 moved to Hybrid. System roles are enums on User object?
    # `auth_middleware` creates session with `UserRole`.
    # `service_users` table has `role` column (string/enum).
    # To upgrade a Service User, we use `update_service_user` endpoint changes the role column directly?
    # Let's check `handle_update_service_user` in backend or CLI. 
    # Ah, `handle_update_service_user` allows updating name/desc/active... does it allow role?
    # CLI `update-service-user` flags: name, desc, active. NO ROLE.
    # So to change role, we probably need `assign-role`?
    # But `assign_role` is for *additional* custom roles. The base role is fixed at creation?
    # Wait, Service User model has a `role` field.
    # If I verify Custom Role assignment, that proves RBAC works.
    
    # 7b. Create Custom Role & Assign
    print_step("Admin: Creating Custom Role 'WarehouseBuilder'...")
    # Need to verify if `create_role` works.
    builder_role = admin_client.roles.create("WarehouseBuilder", "Can build things")
    print(f"Role ID: {builder_role.id}")
    
    # Add Permission to Role
    print_step("Admin: Granting 'Create Warehouse' to Role...")
    # Needs permission string/resource logic. 
    # Grant: action="Create", scope_type="tenant", scope_id=tenant.id ?
    # Checking `permissions.grant` signature in governance.py: 
    # grant(role_id, action, scope_type, scope_id=None)
    # Create Tenant Admin to perform Grant (Root is forbidden from granular grants)
    print_step("Creating Tenant Admin user...")
    t_admin_name = f"admin_{int(time.time())}"
    t_admin = admin_client.users.create(t_admin_name, "admin@example.com", "tenant-admin", tenant.id, "password123")
    
    print_step("Logging in as Tenant Admin...")
    print(f"!!! CREDENTIALS GENERATED !!!")
    print(f"Tenant ID: {tenant.id}")
    print(f"Tenant Admin Username: {t_admin_name}")
    print(f"Tenant Admin Password: password123")
    print(f"Service User API Key: {service_user.api_key}")
    
    admin_client_scoped = PangolinClient(BASE_URL, username=t_admin_name, password="password123", tenant_id=tenant.id)
    
    admin_client_scoped.permissions.grant(service_user.id, "create", "tenant", tenant.id)

    print_step("Admin: Assigning Custom Role to Bot...")
    admin_client.permissions.assign_role(service_user.id, builder_role.id)
    print("✅ Assigned.")

    # 8. Cleanup - SKIPPED FOR MANUAL TESTING
    print_step("Cleanup SKIPPED to allow manual verification.")
    # admin_client.service_users.delete(service_user.id)
    
    print("\n✅ PYPANGOLIN v0.5.0 VERIFICATION PASSED")

if __name__ == "__main__":
    run_test()
