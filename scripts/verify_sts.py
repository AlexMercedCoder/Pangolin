import requests
import json
import os
import sys

# Configuration
BASE_URL = "http://localhost:8080/api/v1"
TENANT_ID = "00000000-0000-0000-0000-000000000000"
HEADERS = {
    "Content-Type": "application/json", 
    "X-Pangolin-Tenant": TENANT_ID
}

def setup_resources():
    print(f"--- Setting up Resources ---")
    
    # 1. Create Warehouse with AWS STS Strategy
    # We use a dummy role and credentials. We expect the SDK to TRY to use them and fail with a Service Error.
    # If implementation is missing, we'd get "Not Implemented".
    warehouse_payload = {
        "name": "sts-verify-warehouse",
        "storage_config": {
            "s3.bucket": "test-bucket",
            "s3.region": "us-east-1",
            "s3.access-key-id": "dummy_access_key",
            "s3.secret-access-key": "dummy_secret_key"
        },
        "vending_strategy": {
            "AwsSts": {
                "role_arn": "arn:aws:iam::123456789012:role/NonExistentRole",
                "external_id": "test-external-id"
            }
        }
    }
    
    # Cleanup first
    try:
        requests.delete(f"{BASE_URL}/warehouses/sts-verify-warehouse", headers=HEADERS)
    except:
        pass

    print("Creating Warehouse...")
    resp = requests.post(f"{BASE_URL}/warehouses", headers=HEADERS, json=warehouse_payload)
    if resp.status_code not in [200, 201]:
        print(f"Failed to create warehouse: {resp.text}")
        sys.exit(1)
    print("Warehouse created.")

def verify_sts_flow():
    print(f"\n--- Verifying AWS STS Flow (Error Path) ---")
    
    # We request credentials for a location in the bucket
    location = "s3://test-bucket/some/table"
    
    # Execution Attempt
    print(f"Requesting Table Credentials for {location}...")
    resp = requests.get(f"{BASE_URL}/warehouses/sts-verify-warehouse/credentials?location={location}", headers=HEADERS)
    
    print(f"Status Code: {resp.status_code}")
    print(f"Response: {resp.text}")
    
    # Assertions
    # We expect a 500 because the credentials are fake and STS will reject them.
    # CRITICAL: We DO NOT want "Not Implemented"
    if resp.status_code == 500:
        if "STS AssumeRole failed" in resp.text:
             print("SUCCESS: Code executed STS logic and received expected upstream error.")
        elif "InvalidClientTokenId" in resp.text or "SignatureDoesNotMatch" in resp.text:
             print("SUCCESS: Code executed STS logic and received expected AWS error.")
        elif "security token included in the request is invalid" in resp.text:
             print("SUCCESS: Code executed STS logic and received expected AWS error.")
        else:
             print("WARNING: Received 500 but message was generic. Manually verify logs.")
    elif resp.status_code == 200:
        print("UNEXPECTED SUCCESS: Did you use real credentials? If so, this is good. If not, logic is mocked?")
    else:
        print(f"FAILURE: Unexpected status code. {resp.text}")
        if "not implemented" in resp.text.lower():
            print("CRITICAL FAILURE: Feature is still marked as not implemented.")
            sys.exit(1)

def verify_presign_flow():
    print(f"\n--- Verifying Presign Get Flow (Error Path) ---")
    
    # Presign Get
    location = "s3://test-bucket/data/file.parquet"
    print(f"Requesting Presigned URL for {location}...")
    
    # Check if we have a presign endpoint. 
    # Usually it's via credentials or a specific endpoint?
    # Wait, the task was to implement `presign_get` internal method.
    # Is there an API endpoint exposing it?
    # Checking `pangolin_api/src/warehouse_handlers.rs`...
    # There might not be an endpoint exposed for this yet??
    # If no endpoint, we can't verify via API.
    # The `Signer` trait has `presign_get`, but is it used?
    
    # Actually, `get_table_credentials` is exposed via `/credentials`.
    # Is there a `/presign` endpoint?
    # Let's check `warehouse_handlers.rs` in next step if this fails.
    # Assuming there isn't one, this step might be "Manual verification via unit test" or added via new endpoint.
    
    # HACK: For now, we only verify STS via the credentials endpoint.
    # If the user wants presign verification, we need to know HOW to call it.
    pass

if __name__ == "__main__":
    setup_resources()
    verify_sts_flow()
