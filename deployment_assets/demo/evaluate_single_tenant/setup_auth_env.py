import sys
import os
sys.path.append(os.path.abspath("../../../pypangolin/src"))
from pypangolin import PangolinClient

def setup():
    print("Setting up Auth Environment...")
    client = PangolinClient(uri="http://localhost:8080")
    
    # Login as Root
    try:
        client.login("admin", "password")
        print("Logged in as Root.")
    except:
        print("Failed to login as Root. Is API running in Auth mode?")
        sys.exit(1)

    t_name = "ui_tenant"
    u_name = "ui_user"

    # Create Tenant if not exists
    tenants = client.tenants.list()
    tenant = next((t for t in tenants if t.name == t_name), None)
    if not tenant:
        print(f"Creating tenant: {t_name}")
        tenant = client.tenants.create(t_name)
    else:
        print(f"Tenant {t_name} exists.")

    # Create User if not exists
    # We need to list users in that tenant? Root can list all users? 
    # Or we assume it fails if exists.
    print(f"Creating user {u_name}...")
    try:
        client.users.create(u_name, "ui@test.com", "tenant-admin", tenant_id=tenant.id, password="password123")
        print("User created.")
    except Exception as e:
        print(f"User create skipped/failed: {e}")

if __name__ == "__main__":
    setup()
