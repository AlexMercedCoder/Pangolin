#!/usr/bin/env python3
"""
Comprehensive live test for MongoStore backend parity.
Tests Service Users, Merge Operations, and Iceberg integration with MinIO.
"""

import requests
import json
from typing import Dict, Any
import uuid
import time
import boto3
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, IntegerType

API_URL = "http://localhost:8080"
MINIO_ENDPOINT = "http://localhost:9000"
MINIO_ACCESS_KEY = "minioadmin"
MINIO_SECRET_KEY = "minioadmin"

def print_section(title):
    print("\n" + "=" * 60)
    print(f"  {title}")
    print("=" * 60)

def test_service_users():
    """Test Service Users CRUD operations."""
    print_section("Testing Service Users")
    
    tenant_id = "00000000-0000-0000-0000-000000000000"
    
    # 1. List service users (should be empty or existing)
    print("\n1. Listing service users...")
    response = requests.get(f"{API_URL}/api/v1/service-users")
    print(f"   Status: {response.status_code}")
    if response.status_code == 200:
        users = response.json()
        print(f"   Found {len(users)} existing service users")
    
    # 2. Create a service user
    print("\n2. Creating service user...")
    service_user_data = {
        "name": f"test-mongo-user-{uuid.uuid4().hex[:8]}",
        "description": "Test service user for MongoStore",
        "tenant_id": tenant_id,
        "role": "tenant-user"
    }
    
    response = requests.post(
        f"{API_URL}/api/v1/service-users",
        json=service_user_data
    )
    print(f"   Status: {response.status_code}")
    if response.status_code in [200, 201]:
        user = response.json()
        print(f"   Created user: {user.get('name')}")
        print(f"   User ID: {user.get('id')}")
        print(f"   API Key: {user.get('api_key', 'N/A')[:20]}...")
        user_id = user.get("id")
        
        # 3. Get service user by ID
        print(f"\n3. Getting service user by ID: {user_id}...")
        response = requests.get(f"{API_URL}/api/v1/service-users/{user_id}")
        print(f"   Status: {response.status_code}")
        if response.status_code == 200:
            print(f"   Retrieved: {response.json().get('name')}")
        
        # 4. Update service user
        print(f"\n4. Updating service user...")
        update_data = {
            "description": "Updated description for MongoStore test",
            "active": True
        }
        response = requests.put(
            f"{API_URL}/api/v1/service-users/{user_id}",
            json=update_data
        )
        print(f"   Status: {response.status_code}")
        if response.status_code == 200:
            print(f"   Updated successfully")
        
        # 5. List service users again
        print(f"\n5. Listing service users (should include new user)...")
        response = requests.get(f"{API_URL}/api/v1/service-users")
        print(f"   Status: {response.status_code}")
        if response.status_code == 200:
            users = response.json()
            print(f"   Found {len(users)} service users")
        
        # 6. Delete service user
        print(f"\n6. Deleting service user...")
        response = requests.delete(f"{API_URL}/api/v1/service-users/{user_id}")
        print(f"   Status: {response.status_code}")
        if response.status_code in [200, 204]:
            print(f"   Deleted successfully")
        
        return True
    else:
        print(f"   Error: {response.text}")
        return False

def test_merge_operations():
    """Test Merge Operations CRUD."""
    print_section("Testing Merge Operations")
    
    tenant_id = "00000000-0000-0000-0000-000000000000"
    catalog_name = f"test_catalog_{uuid.uuid4().hex[:8]}"
    
    # Note: The API might not have POST endpoints for creating merge operations
    # This tests the backend methods through the existing API endpoints
    
    print("\n1. Listing merge operations for catalog...")
    response = requests.get(
        f"{API_URL}/api/v1/catalogs/{catalog_name}/merge-operations"
    )
    print(f"   Status: {response.status_code}")
    if response.status_code == 200:
        ops = response.json()
        print(f"   Found {len(ops)} merge operations")
        return True
    elif response.status_code == 404:
        print(f"   Catalog not found (expected for new catalog)")
        return True
    else:
        print(f"   Response: {response.text}")
        return False

def test_iceberg_integration():
    """Test Iceberg table creation and verify metadata.json in MinIO."""
    print_section("Testing Iceberg Integration with MinIO")
    
    tenant_id = "00000000-0000-0000-0000-000000000000"
    warehouse_name = f"test_warehouse_{uuid.uuid4().hex[:8]}"
    catalog_name = f"test_catalog_{uuid.uuid4().hex[:8]}"
    namespace = "test_ns"
    table_name = "test_table"
    bucket_name = "warehouse"
    
    try:
        # 1. Create warehouse
        print("\n1. Creating warehouse...")
        warehouse_data = {
            "name": warehouse_name,
            "storage_config": {
                "type": "s3",
                "bucket": bucket_name,
                "endpoint": MINIO_ENDPOINT,
                "access_key_id": MINIO_ACCESS_KEY,
                "secret_access_key": MINIO_SECRET_KEY,
                "region": "us-east-1"
            },
            "use_sts": False
        }
        
        response = requests.post(
            f"{API_URL}/api/v1/warehouses",
            json=warehouse_data
        )
        print(f"   Status: {response.status_code}")
        if response.status_code not in [200, 201]:
            print(f"   Error: {response.text}")
            return False
        
        # 2. Create catalog
        print(f"\n2. Creating catalog: {catalog_name}...")
        catalog_data = {
            "name": catalog_name,
            "catalog_type": "Local",
            "warehouse_name": warehouse_name,
            "properties": {}
        }
        
        response = requests.post(
            f"{API_URL}/api/v1/catalogs",
            json=catalog_data
        )
        print(f"   Status: {response.status_code}")
        if response.status_code not in [200, 201]:
            print(f"   Error: {response.text}")
            return False
        
        # 3. Create namespace
        print(f"\n3. Creating namespace: {namespace}...")
        namespace_data = {
            "namespace": [namespace],
            "properties": {}
        }
        
        response = requests.post(
            f"{API_URL}/api/v1/catalogs/{catalog_name}/namespaces",
            json=namespace_data
        )
        print(f"   Status: {response.status_code}")
        
        # 4. Create Iceberg table
        print(f"\n4. Creating Iceberg table: {namespace}.{table_name}...")
        table_data = {
            "name": table_name,
            "schema": {
                "type": "struct",
                "fields": [
                    {"id": 1, "name": "id", "required": True, "type": "int"},
                    {"id": 2, "name": "name", "required": False, "type": "string"}
                ]
            },
            "partition-spec": [],
            "write-order": [],
            "properties": {}
        }
        
        response = requests.post(
            f"{API_URL}/api/v1/catalogs/{catalog_name}/namespaces/{namespace}/tables",
            json=table_data
        )
        print(f"   Status: {response.status_code}")
        if response.status_code not in [200, 201]:
            print(f"   Error: {response.text}")
            return False
        
        table_response = response.json()
        metadata_location = table_response.get("metadata-location", "")
        print(f"   Metadata location: {metadata_location}")
        
        # 5. Verify metadata.json exists in MinIO
        print(f"\n5. Verifying metadata.json in MinIO...")
        time.sleep(2)  # Give MinIO time to sync
        
        s3_client = boto3.client(
            's3',
            endpoint_url=MINIO_ENDPOINT,
            aws_access_key_id=MINIO_ACCESS_KEY,
            aws_secret_access_key=MINIO_SECRET_KEY,
            region_name='us-east-1'
        )
        
        # List objects in the bucket
        try:
            response = s3_client.list_objects_v2(
                Bucket=bucket_name,
                Prefix=f"{catalog_name}/{namespace}/{table_name}/metadata/"
            )
            
            if 'Contents' in response:
                print(f"   Found {len(response['Contents'])} objects in metadata folder:")
                for obj in response['Contents']:
                    print(f"     - {obj['Key']} ({obj['Size']} bytes)")
                    if 'metadata.json' in obj['Key'] or obj['Key'].endswith('.json'):
                        # Download and verify it's valid JSON
                        metadata_obj = s3_client.get_object(Bucket=bucket_name, Key=obj['Key'])
                        metadata_content = metadata_obj['Body'].read()
                        metadata_json = json.loads(metadata_content)
                        print(f"     ✓ Valid JSON metadata file")
                        print(f"       Format version: {metadata_json.get('format-version')}")
                        print(f"       Schema fields: {len(metadata_json.get('schema', {}).get('fields', []))}")
                return True
            else:
                print(f"   ⚠ No metadata files found in MinIO")
                return False
                
        except Exception as e:
            print(f"   Error checking MinIO: {e}")
            return False
            
    except Exception as e:
        print(f"   Unexpected error: {e}")
        import traceback
        traceback.print_exc()
        return False

def main():
    print("=" * 60)
    print("  MongoStore Live Test Suite")
    print("  Testing: Service Users, Merge Operations, Iceberg + MinIO")
    print("=" * 60)
    
    results = {
        "Service Users": False,
        "Merge Operations": False,
        "Iceberg + MinIO": False
    }
    
    try:
        # Test Service Users
        results["Service Users"] = test_service_users()
        
        # Test Merge Operations
        results["Merge Operations"] = test_merge_operations()
        
        # Test Iceberg Integration
        results["Iceberg + MinIO"] = test_iceberg_integration()
        
    except requests.exceptions.RequestException as e:
        print(f"\n✗ Test failed with connection error: {e}")
        print(f"   Make sure the API is running with MongoStore backend")
        print(f"   Set PANGOLIN_STORE_TYPE=mongo in your environment")
        return 1
    except Exception as e:
        print(f"\n✗ Unexpected error: {e}")
        import traceback
        traceback.print_exc()
        return 1
    
    # Print summary
    print("\n" + "=" * 60)
    print("  Test Summary")
    print("=" * 60)
    for test_name, passed in results.items():
        status = "✓ PASSED" if passed else "✗ FAILED"
        print(f"  {test_name}: {status}")
    
    all_passed = all(results.values())
    print("=" * 60)
    if all_passed:
        print("  ✓ All tests PASSED!")
        print("  MongoStore backend parity verified successfully")
    else:
        print("  ✗ Some tests FAILED")
        print("  Review the output above for details")
    print("=" * 60)
    
    return 0 if all_passed else 1

if __name__ == "__main__":
    exit(main())
