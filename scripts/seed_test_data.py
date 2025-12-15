#!/usr/bin/env python3
"""
Seed script for Pangolin Management UI testing
Populates the database with test data for manual testing
"""

import requests
import json
from typing import Dict, Any

# Configuration
API_BASE = "http://localhost:8080/api/v1"
USERNAME = "admin"
PASSWORD = "password"

class PangolinSeeder:
    def __init__(self):
        self.tenant_id = None
        self.catalog_id = None
        self.warehouse_id = None
        
    def headers(self, tenant_id=None):
        """Get headers with optional tenant"""
        h = {}
        if tenant_id:
            h["X-Pangolin-Tenant"] = tenant_id
        return h
    
    def create_tenant(self, name: str, description: str = None) -> str:
        """Create a tenant"""
        print(f"🏢 Creating tenant: {name}")
        data = {"name": name}
        if description:
            data["description"] = description
            
        response = requests.post(
            f"{API_BASE}/tenants",
            headers=self.headers(),
            json=data
        )
        response.raise_for_status()
        tenant_id = response.json()["id"]
        print(f"✅ Created tenant: {name} (ID: {tenant_id})")
        return tenant_id
    
    def create_warehouse(self, name: str, tenant_id: str) -> str:
        """Create a warehouse"""
        print(f"🏭 Creating warehouse: {name}")
        data = {
            "name": name,
            "use_sts": False,
            "storage_config": {
                "type": "s3",
                "bucket": f"{name}-bucket",
                "region": "us-east-1",
                "endpoint": ""
            }
        }
        
        response = requests.post(
            f"{API_BASE}/warehouses",
            headers=self.headers(tenant_id),
            json=data
        )
        response.raise_for_status()
        warehouse_id = response.json()["id"]
        print(f"✅ Created warehouse: {name}")
        return warehouse_id
    
    def create_catalog(self, name: str, warehouse_name: str, tenant_id: str) -> str:
        """Create a catalog"""
        print(f"📚 Creating catalog: {name}")
        data = {
            "name": name,
            "warehouse_name": warehouse_name,
            "storage_location": f"s3://{warehouse_name}-bucket/{name}/"
        }
        
        response = requests.post(
            f"{API_BASE}/catalogs",
            headers=self.headers(tenant_id),
            json=data
        )
        response.raise_for_status()
        catalog_id = response.json()["id"]
        print(f"✅ Created catalog: {name}")
        return catalog_id
    
    def create_branch(self, catalog: str, name: str, branch_type: str, tenant_id: str):
        """Create a branch"""
        print(f"🌿 Creating branch: {name} for catalog {catalog}")
        data = {
            "name": name,
            "catalog": catalog,
            "branch_type": branch_type
        }
        
        response = requests.post(
            f"{API_BASE}/branches",
            headers=self.headers(tenant_id),
            json=data
        )
        response.raise_for_status()
        print(f"✅ Created branch: {name}")
    
    def create_user(self, username: str, email: str, role: str, tenant_id: str) -> str:
        """Create a user"""
        print(f"👤 Creating user: {username}")
        data = {
            "username": username,
            "email": email,
            "password": "password123",
            "role": role,
            "tenant_id": tenant_id
        }
        
        response = requests.post(
            f"{API_BASE}/users",
            headers=self.headers(),
            json=data
        )
        response.raise_for_status()
        user_id = response.json()["id"]
        print(f"✅ Created user: {username}")
        return user_id
    
    def create_role(self, name: str, description: str, tenant_id: str) -> str:
        """Create a role"""
        print(f"🎭 Creating role: {name}")
        data = {
            "name": name,
            "description": description,
            "tenant_id": tenant_id
        }
        
        response = requests.post(
            f"{API_BASE}/roles",
            headers=self.headers(tenant_id),
            json=data
        )
        response.raise_for_status()
        role_id = response.json()["id"]
        print(f"✅ Created role: {name}")
        return role_id
    
    def grant_permission(self, user_id: str, scope: Dict, actions: list, tenant_id: str):
        """Grant a permission"""
        print(f"🔑 Granting permission to user")
        data = {
            "user_id": user_id,
            "scope": scope,
            "actions": actions
        }
        
        response = requests.post(
            f"{API_BASE}/permissions",
            headers=self.headers(tenant_id),
            json=data
        )
        response.raise_for_status()
        print(f"✅ Granted permission")
    
    def seed(self):
        """Run the full seeding process"""
        print("\n" + "="*60)
        print("🌱 Starting Pangolin Seed Script (NO-AUTH MODE)")
        print("="*60 + "\n")
        
        print("⚠️  Running in NO-AUTH mode for seeding")
        print("    Server must be started with PANGOLIN_NO_AUTH=true\n")
        
        # Create tenants
        print("\n📋 Creating Tenants...")
        acme_tenant = self.create_tenant("ACME Corp", "Main production tenant")
        test_tenant = self.create_tenant("Test Org", "Testing and development tenant")
        
        # Create warehouses
        print("\n📋 Creating Warehouses...")
        prod_warehouse = self.create_warehouse("prod-warehouse", acme_tenant)
        dev_warehouse = self.create_warehouse("dev-warehouse", acme_tenant)
        test_warehouse = self.create_warehouse("test-warehouse", test_tenant)
        
        # Create catalogs
        print("\n📋 Creating Catalogs...")
        analytics_catalog = self.create_catalog("analytics", "prod-warehouse", acme_tenant)
        staging_catalog = self.create_catalog("staging", "dev-warehouse", acme_tenant)
        test_catalog = self.create_catalog("test-catalog", "test-warehouse", test_tenant)
        
        # Create branches
        print("\n📋 Creating Branches...")
        self.create_branch("analytics", "main", "production", acme_tenant)
        self.create_branch("analytics", "dev", "experimental", acme_tenant)
        self.create_branch("analytics", "feature-x", "experimental", acme_tenant)
        self.create_branch("staging", "main", "production", acme_tenant)
        
        # Create users
        print("\n📋 Creating Users...")
        alice = self.create_user("alice", "alice@acme.com", "TenantAdmin", acme_tenant)
        bob = self.create_user("bob", "bob@acme.com", "TenantUser", acme_tenant)
        charlie = self.create_user("charlie", "charlie@test.com", "TenantUser", test_tenant)
        
        # Create roles
        print("\n📋 Creating Roles...")
        analyst_role = self.create_role(
            "Data Analyst",
            "Can read and analyze data",
            acme_tenant
        )
        engineer_role = self.create_role(
            "Data Engineer",
            "Can read, write, and manage data pipelines",
            acme_tenant
        )
        
        # Grant permissions
        print("\n📋 Granting Permissions...")
        # Catalog-level read permission
        self.grant_permission(
            bob,
            {"Catalog": "analytics"},
            ["Read", "List"],
            acme_tenant
        )
        
        # Tag-based permission (ABAC example)
        self.grant_permission(
            bob,
            {"Tag": "pii"},
            ["Read"],
            acme_tenant
        )
        
        print("\n" + "="*60)
        print("✅ Seeding Complete!")
        print("="*60)
        print("\n📊 Summary:")
        print(f"  • Tenants: 2 (ACME Corp, Test Org)")
        print(f"  • Warehouses: 3")
        print(f"  • Catalogs: 3")
        print(f"  • Branches: 4")
        print(f"  • Users: 3 (alice, bob, charlie)")
        print(f"  • Roles: 2")
        print(f"  • Permissions: 2")
        print("\n🔐 Login Credentials:")
        print(f"  • Admin: admin / password")
        print(f"  • Alice: alice / password123")
        print(f"  • Bob: bob / password123")
        print(f"  • Charlie: charlie / password123")
        print()

if __name__ == "__main__":
    seeder = PangolinSeeder()
    try:
        seeder.seed()
    except Exception as e:
        print(f"\n❌ Error: {e}")
        import traceback
        traceback.print_exc()
