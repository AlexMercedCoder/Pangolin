import requests
import json
from setup_test_env import setup, BASE_URL

def seed_assets():
    # 1. Ensure base env is set up
    print("Running base setup...")
    setup()
    
    # 2. Login as Tenant Admin to get token
    print("\nLogging in as tenant_admin...")
    resp = requests.post(f"{BASE_URL}/users/login", json={"username": "tenant_admin", "password": "password"})
    if not resp.ok:
        print(f"Login failed: {resp.text}")
        return
        
    data = resp.json()
    token = data["token"]
    
    # Debug print to see structure
    # print(json.dumps(data, indent=2))
    
    user_info = data.get("user", {})
    tenant_id = user_info.get("tenant_id") or user_info.get("tenant-id")
    
    if not tenant_id:
        print("Could not find tenant_id in login response:", data)
        return

    headers = {
        "Authorization": f"Bearer {token}",
        "X-Pangolin-Tenant": tenant_id
    }
    
    catalog = "test-catalog"
    namespace = "default"
    
    # 3. Create namespace 'default' if it doesn't exist
    # Note: Iceberg namespaces endpoint
    print(f"\nCreating namespace '{namespace}'...")
    requests.post(f"{BASE_URL}/{catalog}/namespaces", json={"namespace": [namespace]}, headers=headers)

    # 4. Register Assets
    assets = [
        {
            "name": "daily_events",
            "kind": "CSV_TABLE",
            "location": "s3://test-bucket/data/daily_events.csv",
            "properties": {"header": "true", "delimiter": ","}
        },
        {
            "name": "churn_predictions",
            "kind": "ML_MODEL",
            "location": "s3://test-bucket/models/churn_v1.pkl",
            "properties": {"framework": "sklearn", "version": "1.0"}
        },
        {
            "name": "sales_delta",
            "kind": "DELTA_TABLE",
            "location": "s3://test-bucket/delta/sales",
            "properties": {}
        }
    ]
    
    print("\nRegistering assets...")
    for asset in assets:
        url = f"{BASE_URL}/catalogs/{catalog}/namespaces/{namespace}/assets"
        print(f"POST {url} with {asset['name']}")
        resp = requests.post(url, json=asset, headers=headers)
        if resp.status_code == 201:
            print(f"Successfully registered {asset['name']}")
        else:
            print(f"Failed to register {asset['name']}: {resp.status_code} {resp.text}")

if __name__ == "__main__":
    seed_assets()
