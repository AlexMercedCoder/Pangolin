#!/usr/bin/env python3
"""
PyIceberg Integration Test for Pangolin
Tests the warehouse/catalog refactor with real PyIceberg client using Bearer token auth
"""

import sys
import jwt
import datetime

try:
    from pyiceberg.catalog import load_catalog
    from pyiceberg.schema import Schema
    from pyiceberg.types import NestedField, IntegerType, StringType
    import pyarrow as pa
except ImportError as e:
    print(f"ERROR: Missing required package: {e}")
    print("Please install: pip install 'pyiceberg[s3fs,pyarrow]' pyjwt")
    sys.exit(1)

TENANT_ID = "00000000-0000-0000-0000-000000000001"
CATALOG_URI = "http://localhost:8080"
JWT_SECRET = "secret"  # Should match PANGOLIN_JWT_SECRET env var

def generate_token(tenant_id: str, secret: str) -> str:
    """Generate JWT token with tenant_id"""
    payload = {
        "sub": "test-user",
        "tenant_id": tenant_id,
        "roles": ["User"],
        "exp": datetime.datetime.utcnow() + datetime.timedelta(hours=1)
    }
    return jwt.encode(payload, secret, algorithm="HS256")

def test_connection():
    """Test 1: Basic connection to Pangolin"""
    print("\n=== Test 1: Connection ===")
    try:
        token = generate_token(TENANT_ID, JWT_SECRET)
        
        catalog = load_catalog(
            "pangolin",
            **{
                "uri": CATALOG_URI,
                "prefix": "analytics",  # Catalog name as prefix
                "token": token,  # Bearer token authentication
            }
        )
        print(f"✓ Connected to catalog: {catalog}")
        return catalog
    except Exception as e:
        print(f"✗ Connection failed: {e}")
        raise

def test_namespaces(catalog):
    """Test 2: Namespace operations"""
    print("\n=== Test 2: Namespaces ===")
    try:
        # List existing namespaces
        namespaces = catalog.list_namespaces()
        print(f"Existing namespaces: {namespaces}")
        
        # Create namespace
        catalog.create_namespace("test_namespace")
        print("✓ Created namespace: test_namespace")
        
        # Verify
        namespaces = catalog.list_namespaces()
        print(f"✓ Updated namespaces: {namespaces}")
        
        return True
    except Exception as e:
        print(f"✗ Namespace operations failed: {e}")
        raise

def test_table_creation(catalog):
    """Test 3: Table creation"""
    print("\n=== Test 3: Table Creation ===")
    try:
        schema = Schema(
            NestedField(1, "id", IntegerType(), required=True),
            NestedField(2, "name", StringType(), required=False),
            NestedField(3, "email", StringType(), required=False),
        )
        
        table = catalog.create_table(
            identifier="test_namespace.users",
            schema=schema,
            location="memory://test-bucket/analytics/test_namespace/users"
        )
        
        print(f"✓ Created table: {table}")
        return table
    except Exception as e:
        print(f"✗ Table creation failed: {e}")
        raise

def test_data_write(table):
    """Test 4: Write data"""
    print("\n=== Test 4: Data Write ===")
    try:
        data = pa.Table.from_pydict({
            "id": [1, 2, 3],
            "name": ["Alice", "Bob", "Charlie"],
            "email": ["alice@example.com", "bob@example.com", "charlie@example.com"]
        })
        
        table.append(data)
        print(f"✓ Wrote {len(data)} rows")
        return True
    except Exception as e:
        print(f"✗ Data write failed: {e}")
        raise

def test_data_read(table):
    """Test 5: Read data"""
    print("\n=== Test 5: Data Read ===")
    try:
        scan = table.scan()
        result = scan.to_arrow()
        
        print(f"✓ Read {len(result)} rows")
        print("\nData:")
        print(result.to_pandas())
        return True
    except Exception as e:
        print(f"✗ Data read failed: {e}")
        raise

def test_list_tables(catalog):
    """Test 6: List tables"""
    print("\n=== Test 6: List Tables ===")
    try:
        tables = catalog.list_tables("test_namespace")
        print(f"✓ Tables in test_namespace: {tables}")
        return True
    except Exception as e:
        print(f"✗ List tables failed: {e}")
        raise

def main():
    print("=" * 60)
    print("Pangolin PyIceberg Integration Test")
    print("Using Bearer Token Authentication (Iceberg REST Spec)")
    print("=" * 60)
    
    try:
        # Run tests
        catalog = test_connection()
        test_namespaces(catalog)
        table = test_table_creation(catalog)
        test_data_write(table)
        test_data_read(table)
        test_list_tables(catalog)
        
        print("\n" + "=" * 60)
        print("✓ ALL TESTS PASSED!")
        print("=" * 60)
        return 0
        
    except Exception as e:
        print("\n" + "=" * 60)
        print(f"✗ TESTS FAILED: {e}")
        print("=" * 60)
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    sys.exit(main())

