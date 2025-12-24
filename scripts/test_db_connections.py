import os
import sys
import uuid
import time

# Ensure we import local pypangolin
sys.path.insert(0, os.path.abspath("pypangolin/src"))

from pypangolin import PangolinClient
from pypangolin.assets.connections import PostgreSQLAsset, MySQLAsset, MongoDBAsset

def log(msg):
    print(f"[{time.strftime('%H:%M:%S')}] {msg}")

def main():
    log("=== Starting Database Connection Assets Verification ===")
    
    # Setup
    client = PangolinClient(uri="http://localhost:8080")
    client.login("admin", "password")
    log("✅ Logged in as Root")
    
    # Create tenant
    tenant_name = f"db_test_{uuid.uuid4().hex[:8]}"
    tenant = client.tenants.create(tenant_name)
    client.set_tenant(tenant.id)
    log(f"✅ Created tenant: {tenant_name}")
    
    # Create admin user
    username = f"admin_{tenant_name}"
    client.users.create(
        username=username,
        email=f"{username}@example.com",
        role="tenant-admin",
        tenant_id=tenant.id,
        password="Admin123!"
    )
    client.login(username, "Admin123!", tenant_id=tenant.id)
    log("✅ Logged in as Tenant Admin")
    
    # Create catalog for connections
    wh = client.warehouses.create_s3("conn_wh", "s3://connections", access_key="key", secret_key="secret")
    cat = client.catalogs.create("connections", "conn_wh")
    log("✅ Created catalog: connections")
    
    # Create namespace
    ns_client = client.catalogs.namespaces("connections")
    ns_client.create(["databases"])
    log("✅ Created namespace: databases")
    
    # Test 1: PostgreSQL with auto-managed key
    log("--- Testing PostgreSQL (Auto-managed Key) ---")
    try:
        pg_asset = PostgreSQLAsset.register(
            client,
            catalog="connections",
            namespace="databases",
            name="test_postgres",
            connection_string="postgresql://localhost:5432/testdb",
            credentials={
                "username": "testuser",
                "password": "testpass"
            },
            store_key=True,
            description="Test PostgreSQL connection"
        )
        log(f"✅ Registered PostgreSQL asset: {pg_asset.get('name')}")
        
        # Debug: Check what was stored
        asset_check = PostgreSQLAsset._get_asset(client, "connections", "databases", "test_postgres")
        log(f"DEBUG: Asset properties keys: {list(asset_check.get('properties', {}).keys())}")
        if "encryption_key" in asset_check.get('properties', {}):
            log(f"DEBUG: Encryption key IS stored in properties")
        else:
            log(f"DEBUG: Encryption key NOT in properties!")
        
        # Connect to PostgreSQL
        conn = PostgreSQLAsset.connect(client, "connections", "databases", "test_postgres")
        cursor = conn.cursor()
        cursor.execute("SELECT version()")
        version = cursor.fetchone()[0]
        log(f"✅ Connected to PostgreSQL: {version[:50]}...")
        cursor.close()
        conn.close()
        
    except Exception as e:
        log(f"❌ PostgreSQL test failed: {str(e)}")
    
    # Test 2: MySQL with user-managed key
    log("--- Testing MySQL (User-managed Key) ---")
    try:
        # Generate a proper Fernet key (32 bytes base64-encoded)
        from cryptography.fernet import Fernet
        my_key = Fernet.generate_key().decode('utf-8')
        
        mysql_asset = MySQLAsset.register(
            client,
            catalog="connections",
            namespace="databases",
            name="test_mysql",
            connection_string="mysql://localhost:3307/testdb",
            credentials={
                "username": "testuser",
                "password": "testpass"
            },
            encryption_key=my_key,
            store_key=False,  # Don't store key
            description="Test MySQL connection"
        )
        log(f"✅ Registered MySQL asset: {mysql_asset.get('name')}")
        
        # Connect to MySQL (must provide key)
        conn = MySQLAsset.connect(
            client, "connections", "databases", "test_mysql",
            encryption_key=my_key
        )
        cursor = conn.cursor()
        cursor.execute("SELECT VERSION()")
        version = cursor.fetchone()[0]
        log(f"✅ Connected to MySQL: {version}")
        cursor.close()
        conn.close()
        
    except Exception as e:
        log(f"❌ MySQL test failed: {str(e)}")
    
    # Test 3: MongoDB
    log("--- Testing MongoDB ---")
    try:
        mongo_asset = MongoDBAsset.register(
            client,
            catalog="connections",
            namespace="databases",
            name="test_mongo",
            connection_string="mongodb://localhost:27017/admin",  # Use admin database for auth
            credentials={
                "username": "testuser",
                "password": "testpass"
            },
            store_key=True,
            description="Test MongoDB connection"
        )
        log(f"✅ Registered MongoDB asset: {mongo_asset.get('name')}")
        
        # Connect to MongoDB
        mongo_client = MongoDBAsset.connect(client, "connections", "databases", "test_mongo")
        server_info = mongo_client.server_info()
        log(f"✅ Connected to MongoDB: version {server_info['version']}")
        mongo_client.close()
        
    except Exception as e:
        log(f"❌ MongoDB test failed: {str(e)}")
    
    # Test 4: Verify encryption (credentials should not be readable)
    log("--- Testing Encryption Security ---")
    try:
        asset = PostgreSQLAsset._get_asset(client, "connections", "databases", "test_postgres")
        properties = asset.get("properties", {})
        
        # Check that password is encrypted
        if "encrypted_password" in properties:
            encrypted_pw = properties["encrypted_password"]
            log(f"✅ Password is encrypted: {encrypted_pw[:20]}...")
            
            # Verify we can't see plain text password
            if "testpass" not in encrypted_pw:
                log("✅ Plain text password not visible in properties")
            else:
                log("❌ WARNING: Plain text password visible!")
        else:
            log("❌ No encrypted_password found in properties")
            
    except Exception as e:
        log(f"❌ Encryption test failed: {str(e)}")
    
    # Cleanup
    log("--- Cleanup ---")
    client.catalogs.delete("connections")
    client.warehouses.delete("conn_wh")
    client.login("admin", "password")
    client.tenants.delete(tenant.id)
    log("✅ Cleaned up test resources")
    
    log("=== Database Connection Assets Verification Complete ===")

if __name__ == "__main__":
    main()
