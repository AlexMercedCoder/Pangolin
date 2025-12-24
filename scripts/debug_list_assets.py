import requests
import json

BASE_URL = "http://localhost:8080/api/v1"

def debug():
    # 1. Login
    resp = requests.post(f"{BASE_URL}/users/login", json={"username": "admin", "password": "password"})
    if not resp.ok:
        print(f"Login failed: {resp.text}")
        return
    
    data = resp.json()
    token = data["token"]
    # Handle both tenant_id locations
    tenant_id = data.get("user", {}).get("tenant_id") or data.get("user", {}).get("tenant-id")
    
    headers = {
        "Authorization": f"Bearer {token}",
        "X-Pangolin-Tenant": tenant_id
    }
    
    print(f"Token: {token[:10]}...")
    print(f"Tenant: {tenant_id}")

    # 2. List Assets
    catalog = "test-catalog"
    namespace = "default"
    url = f"{BASE_URL}/catalogs/{catalog}/namespaces/{namespace}/assets"
    
    print(f"Fetching: {url}")
    resp = requests.get(url, headers=headers)
    
    if resp.ok:
        assets = resp.json()
        print(f"Status: {resp.status_code}")
        print(f"Count: {len(assets)}")
        print(json.dumps(assets, indent=2))
    else:
        print(f"Failed: {resp.status_code} {resp.text}")

if __name__ == "__main__":
    debug()
