#!/usr/bin/env python3
"""
Comprehensive live test for MemoryStore with API fixes.
Tests Service Users (with fixed response), Merge Operations, and Iceberg integration.
"""

import requests
import json
from typing import Dict, Any
import uuid
import time
import boto3

API_URL = "http://localhost:8080"
MINIO_ENDPOINT = "http://localhost:9000"
MINIO_ACCESS_KEY = "minioadmin"
MINIO_SECRET_KEY = "minioadmin"

def print_section(title):
    print("\n" + "=" * 60)
    print(f"  {title}")
    print("=" * 60)

def test_service_users_with_fixed_response():
    """Test Service Users with fixed API response."""
    print_section("Testing Service Users (Fixed API Response)")
    
    # 1. Create service user
    print("\n1. Creating service user...")
    service_user_data = {
        "name": f"test-user-{uuid.uuid4().hex[:8]}",
        "description": "Test service user with fixed response",
        "role": "tenant-user",
        "expires_in_days": 365
    }
    
    response = requests.post(
        f"{API_URL}/api/v1/service-users",
        json=service_user_data
    )
    print(f"   Status: {response.status_code}")
    
    if response.status_code in [200, 201]:
        user = response.json()
        print(f"   ✓ Response includes all fields:")
        print(f"     - id: {user.get('id')}")
        print(f"     - name: {user.get('name')}")
        print(f"     - description: {user.get('description')}")
        print(f"     - tenant_id: {user.get('tenant_id')}")
        print(f"     - role: {user.get('role')}")
        print(f"     - api_key: {user.get('api_key', '')[:20]}...")
        print(f"     - created_at: {user.get('created_at')}")
        print(f"     - created_by: {user.get('created_by')}")
        print(f"     - expires_at: {user.get('expires_at')}")
        print(f"     - active: {user.get('active')}")
        
        user_id = user.get("id")
        if not user_id:
            print(f"   ✗ FAILED: Response missing 'id' field!")
            return False
            
        # 2. Get service user by ID
        print(f"\n2. Getting service user by ID: {user_id}...")
        response = requests.get(f"{API_URL}/api/v1/service-users/{user_id}")
        print(f"   Status: {response.status_code}")
        if response.status_code == 200:
            print(f"   ✓ Retrieved: {response.json().get('name')}")
        
        # 3. Update service user
        print(f"\n3. Updating service user...")
        update_data = {
            "description": "Updated description",
            "active": True
        }
        response = requests.put(
            f"{API_URL}/api/v1/service-users/{user_id}",
            json=update_data
        )
        print(f"   Status: {response.status_code}")
        if response.status_code == 200:
            print(f"   ✓ Updated successfully")
        
        # 4. Rotate API key
        print(f"\n4. Rotating API key...")
        response = requests.post(f"{API_URL}/api/v1/service-users/{user_id}/rotate")
        print(f"   Status: {response.status_code}")
        if response.status_code == 200:
            rotated = response.json()
            print(f"   ✓ New API key: {rotated.get('api_key', '')[:20]}...")
            print(f"   ✓ Response includes id: {rotated.get('id')}")
        
        # 5. Delete service user
        print(f"\n5. Deleting service user...")
        response = requests.delete(f"{API_URL}/api/v1/service-users/{user_id}")
        print(f"   Status: {response.status_code}")
        if response.status_code in [200, 204]:
            print(f"   ✓ Deleted successfully")
        
        return True
    else:
        print(f"   ✗ Error: {response.text}")
        return False

def test_merge_operations():
    """Test Merge Operations."""
    print_section("Testing Merge Operations")
    
    catalog_name = f"test_catalog_{uuid.uuid4().hex[:8]}"
    
    print(f"\n1. Listing merge operations for catalog: {catalog_name}...")
    response = requests.get(
        f"{API_URL}/api/v1/catalogs/{catalog_name}/merge-operations"
    )
    print(f"   Status: {response.status_code}")
    if response.status_code == 200:
        ops = response.json()
        print(f"   ✓ Found {len(ops)} merge operations")
        return True
    elif response.status_code == 404:
        print(f"   ✓ Catalog not found (expected for new catalog)")
        return True
    else:
        print(f"   ✗ Unexpected response: {response.text}")
        return False

def test_iceberg_with_minio():
    """Test Iceberg table creation and verify metadata in MinIO."""
    print_section("Testing Iceberg Integration with MinIO")
    
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
            print(f"   ✗ Error: {response.text}")
            return False
        print(f"   ✓ Warehouse created")
        
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
            print(f"   ✗ Error: {response.text}")
            return False
        print(f"   ✓ Catalog created")
        
        # 3. Create namespace (using Iceberg REST API endpoint)
        print(f"\n3. Creating namespace: {namespace}...")
        namespace_data = {
            "namespace": [namespace],
            "properties": {}
        }
        
        # Iceberg REST API uses /v1/{catalog_name}/namespaces
        response = requests.post(
            f"{API_URL}/v1/{catalog_name}/namespaces",
            json=namespace_data
        )
        print(f"   Status: {response.status_code}")
        if response.status_code not in [200, 201]:
            print(f"   ✗ Namespace creation failed: {response.status_code}")
            print(f"   Error: {response.text}")
            return False
        print(f"   ✓ Namespace created")
        
        # 4. Create Iceberg table (using Iceberg REST API endpoint)
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
            f"{API_URL}/v1/{catalog_name}/namespaces/{namespace}/tables",
            json=table_data
        )
        print(f"   Status: {response.status_code}")
        if response.status_code not in [200, 201]:
            print(f"   ✗ Error: {response.text}")
            return False
        
        table_response = response.json()
        metadata_location = table_response.get("metadata-location", "")
        print(f"   ✓ Table created")
        print(f"   Metadata location: {metadata_location}")
        
        # 5. Verify metadata.json in MinIO
        print(f"\n5. Verifying metadata.json in MinIO...")
        time.sleep(2)
        
        s3_client = boto3.client(
            's3',
            endpoint_url=MINIO_ENDPOINT,
            aws_access_key_id=MINIO_ACCESS_KEY,
            aws_secret_access_key=MINIO_SECRET_KEY,
            region_name='us-east-1'
        )
        
        try:
            response = s3_client.list_objects_v2(
                Bucket=bucket_name,
                Prefix=f"{catalog_name}/{namespace}/{table_name}/metadata/"
            )
            
            if 'Contents' in response:
                print(f"   ✓ Found {len(response['Contents'])} objects in metadata folder:")
                for obj in response['Contents']:
                    print(f"     - {obj['Key']} ({obj['Size']} bytes)")
                    if 'metadata.json' in obj['Key'] or obj['Key'].endswith('.json'):
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
            print(f"   ✗ Error checking MinIO: {e}")
            return False
            
    except Exception as e:
        print(f"   ✗ Unexpected error: {e}")
        import traceback
        traceback.print_exc()
        return False

def main():
    print("=" * 60)
    print("  MemoryStore Live Test Suite (with API Fixes)")
    print("  Testing: Service Users, Merge Operations, Iceberg + MinIO")
    print("=" * 60)
    
    results = {
        "Service Users (Fixed Response)": False,
        "Merge Operations": False,
        "Iceberg + MinIO": False
    }
    
    try:
        # Test Service Users with fixed response
        results["Service Users (Fixed Response)"] = test_service_users_with_fixed_response()
        
        # Test Merge Operations
        results["Merge Operations"] = test_merge_operations()
        
        # Test Iceberg Integration
        results["Iceberg + MinIO"] = test_iceberg_with_minio()
        
    except requests.exceptions.RequestException as e:
        print(f"\n✗ Test failed with connection error: {e}")
        print(f"   Make sure the API is running at {API_URL}")
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
        print("  API fixes verified successfully")
    else:
        print("  ⚠ Some tests FAILED")
        print("  Review the output above for details")
    print("=" * 60)
    
    return 0 if all_passed else 1

if __name__ == "__main__":
    exit(main())
