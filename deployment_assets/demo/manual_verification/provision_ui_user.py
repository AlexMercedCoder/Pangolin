
import sys
import os
import uuid

# Add pypangolin to path
sys.path.append(os.path.abspath("../../../pypangolin/src"))

from pypangolin import PangolinClient

def provision_user():
    # Admin credentials (root)
    admin_user = "admin"
    admin_pass = "password"
    
    # Initialize client
    client = PangolinClient(uri="http://localhost:8080")
    
    # Login as admin
    print("Logging in as admin...")
    client.login(admin_user, admin_pass)
    
    # Create a new tenant with a unique name
    tenant_name = f"ui_test_tenant_{uuid.uuid4()}"
    print(f"Creating tenant: {tenant_name}")
    tenant = client.tenants.create(tenant_name)
    print(f"Tenant created. ID: {tenant.id}")
    
    # Create a tenant admin user
    username = f"ui_user_{uuid.uuid4().hex[:8]}"
    password = "password123"
    print(f"Creating user: {username}")
    
    user = client.users.create(
        username, 
        f"{username}@test.com", 
        "tenant-admin",
        tenant_id=tenant.id, 
        password=password
    )
    print(f"User created. ID: {user.id}")
    
    print("\n--- CREDENTIALS FOR UI TESTING ---")
    print(f"Tenant ID: {tenant.id}")
    print(f"Username: {username}")
    print(f"Password: {password}")
    print("----------------------------------")

if __name__ == "__main__":
    provision_user()
