import requests
import time
import os
import json

# Configuration
BASE_URL = "http://localhost:8080/api/v1"
TENANT_ID = os.environ.get("TENANT_ID", "00000000-0000-0000-0000-000000000000")
HEADERS = {
    "X-Pangolin-Tenant": TENANT_ID,
    "Content-Type": "application/json"
}

def log(msg):
    print(f"[VERIFY] {msg}")

def test_caching_flow():
    # 1. Create a Warehouse
    wh_name = f"cache_test_wh_{int(time.time())}"
    log(f"Creating warehouse: {wh_name}")
    resp = requests.post(f"{BASE_URL}/warehouses", headers=HEADERS, json={
        "name": wh_name,
        "storage_config": {"type": "s3", "bucket": "test-bucket", "region": "us-east-1"},
        "use_sts": False
    })
    resp.raise_for_status()

    # 2. First Lookup (Cache Miss)
    log("First lookup (Should be Cache Miss - internal)")
    t0 = time.time()
    resp = requests.get(f"{BASE_URL}/warehouses/{wh_name}", headers=HEADERS)
    resp.raise_for_status()
    t1 = time.time()
    log(f"First lookup took: {(t1-t0)*1000:.2f}ms")

    # 3. Second Lookup (Should be Cache Hit)
    # Note: We can't easily verify "Hit" from outside without timing or logs, 
    # but we can verify it returns correct data.
    log("Second lookup (Should be Cache Hit - internal)")
    t0 = time.time()
    resp = requests.get(f"{BASE_URL}/warehouses/{wh_name}", headers=HEADERS)
    resp.raise_for_status()
    t1 = time.time()
    log(f"Second lookup took: {(t1-t0)*1000:.2f}ms")
    assert resp.json()["name"] == wh_name

    # 4. Update Warehouse (Trigger Invalidation)
    log("Updating warehouse (Should Invalidate Cache)")
    resp = requests.put(f"{BASE_URL}/warehouses/{wh_name}", headers=HEADERS, json={
         "storage_config": {"type": "s3", "bucket": "UPDATED-bucket", "region": "us-east-1"}
    })
    resp.raise_for_status()
    
    # 5. Lookup After Update (Should be Cache Miss + New Data)
    log("Lookup after update (Should obtain NEW data)")
    resp = requests.get(f"{BASE_URL}/warehouses/{wh_name}", headers=HEADERS)
    resp.raise_for_status()
    data = resp.json()
    
    # Verify we got the updated bucket
    current_bucket = data["storage_config"]["bucket"]
    log(f"Current bucket: {current_bucket}")
    assert current_bucket == "UPDATED-bucket", f"Cache invalidation failed! Expected 'UPDATED-bucket', got '{current_bucket}'"

    log("SUCCESS: Cache invalidation verified correctly.")

    # Cleanup
    requests.delete(f"{BASE_URL}/warehouses/{wh_name}", headers=HEADERS)

if __name__ == "__main__":
    try:
        test_caching_flow()
    except Exception as e:
        log(f"FAILURE: {e}")
        exit(1)
