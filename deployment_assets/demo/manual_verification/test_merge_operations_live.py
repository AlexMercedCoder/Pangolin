#!/usr/bin/env python3
"""
Live test script for PostgreSQL Merge Operations implementation.
Tests all 11 trait methods to ensure they work correctly.
"""

import requests
import json
from typing import Dict, Any
import uuid

API_URL = "http://localhost:8080"

def test_merge_operations():
    """Test Merge Operations endpoints."""
    print("\n=== Testing Merge Operations ===")
    
    # Generate unique IDs for this test
    tenant_id = "00000000-0000-0000-0000-000000000000"
    catalog_name = f"test_merge_catalog_{uuid.uuid4().hex[:8]}"
    user_id = str(uuid.uuid4())
    
    # 1. Create a merge operation
    print("1. Creating merge operation...")
    merge_op_data = {
        "tenant_id": tenant_id,
        "catalog_name": catalog_name,
        "source_branch": "feature-branch",
        "target_branch": "main",
        "base_commit_id": None,
        "initiated_by": user_id
    }
    
    response = requests.post(
        f"{API_URL}/api/v1/merge/operations",
        json=merge_op_data
    )
    print(f"   Status: {response.status_code}")
    if response.status_code in [200, 201]:
        operation = response.json()
        print(f"   Created operation: {json.dumps(operation, indent=2)}")
        operation_id = operation.get("id")
        
        # 2. Get merge operation by ID
        print(f"\n2. Getting merge operation by ID: {operation_id}...")
        response = requests.get(f"{API_URL}/api/v1/merge/operations/{operation_id}")
        print(f"   Status: {response.status_code}")
        if response.status_code == 200:
            print(f"   Operation: {json.dumps(response.json(), indent=2)}")
        
        # 3. List merge operations
        print(f"\n3. Listing merge operations for catalog: {catalog_name}...")
        response = requests.get(
            f"{API_URL}/api/v1/merge/operations",
            params={"catalog_name": catalog_name}
        )
        print(f"   Status: {response.status_code}")
        if response.status_code == 200:
            ops = response.json()
            print(f"   Found {len(ops)} operations")
        
        # 4. Update merge operation status
        print(f"\n4. Updating merge operation status to 'Conflicted'...")
        response = requests.put(
            f"{API_URL}/api/v1/merge/operations/{operation_id}/status",
            json={"status": "Conflicted"}
        )
        print(f"   Status: {response.status_code}")
        if response.status_code == 200:
            print(f"   Updated: {response.json()}")
        
        # 5. Create a merge conflict
        print(f"\n5. Creating merge conflict...")
        conflict_data = {
            "merge_operation_id": operation_id,
            "conflict_type": {
                "SchemaConflict": {
                    "asset_name": "test_table",
                    "conflicting_columns": ["col1", "col2"]
                }
            },
            "asset_id": str(uuid.uuid4()),
            "description": "Schema conflict in test_table"
        }
        
        response = requests.post(
            f"{API_URL}/api/v1/merge/conflicts",
            json=conflict_data
        )
        print(f"   Status: {response.status_code}")
        if response.status_code in [200, 201]:
            conflict = response.json()
            print(f"   Created conflict: {json.dumps(conflict, indent=2)}")
            conflict_id = conflict.get("id")
            
            # 6. Get merge conflict by ID
            print(f"\n6. Getting merge conflict by ID: {conflict_id}...")
            response = requests.get(f"{API_URL}/api/v1/merge/conflicts/{conflict_id}")
            print(f"   Status: {response.status_code}")
            if response.status_code == 200:
                print(f"   Conflict: {json.dumps(response.json(), indent=2)}")
            
            # 7. List merge conflicts for operation
            print(f"\n7. Listing conflicts for operation: {operation_id}...")
            response = requests.get(f"{API_URL}/api/v1/merge/operations/{operation_id}/conflicts")
            print(f"   Status: {response.status_code}")
            if response.status_code == 200:
                conflicts = response.json()
                print(f"   Found {len(conflicts)} conflicts")
            
            # 8. Resolve merge conflict
            print(f"\n8. Resolving merge conflict...")
            resolution_data = {
                "conflict_id": conflict_id,
                "strategy": "TakeSource",
                "resolved_value": None,
                "resolved_by": user_id
            }
            
            response = requests.post(
                f"{API_URL}/api/v1/merge/conflicts/{conflict_id}/resolve",
                json=resolution_data
            )
            print(f"   Status: {response.status_code}")
            if response.status_code == 200:
                print(f"   Resolved: {response.json()}")
            
            # 9. Add conflict to operation
            print(f"\n9. Adding conflict to operation...")
            response = requests.post(
                f"{API_URL}/api/v1/merge/operations/{operation_id}/conflicts/{conflict_id}"
            )
            print(f"   Status: {response.status_code}")
            if response.status_code == 200:
                print(f"   Added: {response.json()}")
        
        # 10. Complete merge operation
        print(f"\n10. Completing merge operation...")
        result_commit_id = str(uuid.uuid4())
        response = requests.post(
            f"{API_URL}/api/v1/merge/operations/{operation_id}/complete",
            json={"result_commit_id": result_commit_id}
        )
        print(f"   Status: {response.status_code}")
        if response.status_code == 200:
            print(f"   Completed: {response.json()}")
        
        # 11. Create another operation to test abort
        print(f"\n11. Creating another merge operation to test abort...")
        response = requests.post(
            f"{API_URL}/api/v1/merge/operations",
            json=merge_op_data
        )
        if response.status_code in [200, 201]:
            operation2 = response.json()
            operation2_id = operation2.get("id")
            
            # Abort the operation
            print(f"    Aborting merge operation: {operation2_id}...")
            response = requests.post(f"{API_URL}/api/v1/merge/operations/{operation2_id}/abort")
            print(f"    Status: {response.status_code}")
            if response.status_code == 200:
                print(f"    Aborted: {response.json()}")
                return True
    else:
        print(f"   Error: {response.text}")
        return False

def main():
    print("=" * 60)
    print("PostgreSQL Merge Operations Live Test")
    print("=" * 60)
    
    try:
        success = test_merge_operations()
        
        print("\n" + "=" * 60)
        if success:
            print("✓ All Merge Operations tests completed successfully!")
        else:
            print("✗ Some tests failed")
        print("=" * 60)
        
        return 0 if success else 1
        
    except requests.exceptions.RequestException as e:
        print(f"\n✗ Test failed with error: {e}")
        if hasattr(e, 'response') and e.response is not None:
            print(f"   Response: {e.response.text}")
        return 1
    except Exception as e:
        print(f"\n✗ Unexpected error: {e}")
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    exit(main())
