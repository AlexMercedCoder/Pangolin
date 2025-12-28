import requests
import json
import os
import sys
import time

# Configuration
BASE_URL = "http://localhost:8080/api/v1"
HEALTH_URL = "http://localhost:8080/health"
TENANT_ID = "00000000-0000-0000-0000-000000000000"
HEADERS = {
    "Content-Type": "application/json", 
    "X-Pangolin-Tenant": TENANT_ID
}

def wait_for_api(timeout=60):
    print(f"Waiting for API at {HEALTH_URL}...")
    start_time = time.time()
    while time.time() - start_time < timeout:
        try:
            resp = requests.get(HEALTH_URL)
            if resp.status_code == 200:
                print("API is ready.")
                return True
        except requests.exceptions.ConnectionError:
            pass
        time.sleep(2)
    print("Timeout waiting for API.")
    return False

def setup_resources():
    print(f"--- Setting up Resources (LocalStack) ---")
    
    # 1. Create Warehouse with AWS STS Strategy pointing to LocalStack
    warehouse_payload = {
        "name": "sts-localstack-warehouse",
        "storage_config": {
            "s3.bucket": "test-bucket",
            "s3.region": "us-east-1",
            "s3.access-key-id": "test",
            "s3.secret-access-key": "test",
            "s3.endpoint": "http://localstack:4566" 
        },
        "vending_strategy": {
            "AwsSts": {
                "role_arn": "arn:aws:iam::000000000000:role/PangolinRole",
                "external_id": "test-external-id"
            }
        }
    }
    
    # Cleanup first
    try:
        requests.delete(f"{BASE_URL}/warehouses/sts-localstack-warehouse", headers=HEADERS)
    except:
        pass

    print("Creating Warehouse...")
    # Retry warehouse creation to allow for DB/Storage initialization
    for i in range(5):
        try:
            resp = requests.post(f"{BASE_URL}/warehouses", headers=HEADERS, json=warehouse_payload)
            if resp.status_code in [200, 201]:
                print("Warehouse created.")
                return
            else:
                 print(f"Attempt {i+1}: Failed to create warehouse ({resp.status_code}). Retrying...")
        except Exception as e:
            print(f"Attempt {i+1}: Failed to connect ({e}). Retrying...")
        time.sleep(2)
    
    print("Failed to create warehouse after retries.")
    sys.exit(1)

def verify_sts_flow():
    print(f"\n--- Verifying AWS STS Flow (Happy Path) ---")
    
    # We request credentials for a location in the bucket
    location = "s3://test-bucket/some/table"
    
    # Execution Attempt with Retry (LocalStack STS might take a moment)
    print(f"Requesting Table Credentials for {location}...")
    
    for i in range(10):
        try:
            resp = requests.get(f"{BASE_URL}/warehouses/sts-localstack-warehouse/credentials?location={location}", headers=HEADERS)
            
            if resp.status_code == 200:
                creds = resp.json()
                print("SUCCESS: Received credentials from LocalStack STS.")
                print(f"Access Key ID: {creds.get('access_key_id')}")
                print(f"Session Token: {creds.get('session_token')[:20]}...")
                return
            elif resp.status_code == 500:
                print(f"Attempt {i+1}: Got 500 (likely LocalStack not ready). Response: {resp.text}. Retrying...")
            else:
                 print(f"Attempt {i+1}: Unexpected status {resp.status_code}. Retrying...")
        except Exception as e:
             print(f"Attempt {i+1}: Request failed {e}. Retrying...")
        
        time.sleep(3)
        
    print(f"FAILURE: Could not get valid credentials after retries.")
    sys.exit(1)

if __name__ == "__main__":
    if wait_for_api():
        setup_resources()
        verify_sts_flow()
    else:
        sys.exit(1)
