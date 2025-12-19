import requests
import json

BASE_URL = "http://localhost:8080"
AUTH_URL = f"{BASE_URL}/api/v1/users/login"

def run():
    print("--- Seeding Generic Assets ---")
    
    # 1. Login
    session = requests.Session()
    try:
        resp = session.post(AUTH_URL, json={"username": "tenant_admin", "password": "password"})
        if resp.status_code != 200:
            print(f"Login failed: {resp.status_code} {resp.text}")
            return
        
        data = resp.json()
        token = data['token']
        tenant_id = data['user']['tenant-id']
        headers = {
            "Authorization": f"Bearer {token}",
            "X-Pangolin-Tenant": tenant_id
        }
        print(f"Logged in. Tenant ID: {tenant_id}")
    except Exception as e:
        print(f"Login exception: {e}")
        return

    catalog_name = "test-catalog"
    namespace = "default"

    # 2. Create Generic File (CSV)
    print("Creating 'data_export.csv' (GenericFile)...")
    payload_csv = {
        "name": "data_export.csv",
        "kind": "GenericFile",
        "location": "s3://warehouse/test-catalog/default/data_export.csv",
        "properties": {
            "format": "csv",
            "size_bytes": "1024",
            "row_count": "50",
            "description": "Weekly data export"
        }
    }
    resp = session.post(f"{BASE_URL}/v1/{catalog_name}/namespaces/{namespace}/assets", headers=headers, json=payload_csv)
    if resp.status_code == 201:
        print("Success.")
    else:
        print(f"Failed: {resp.status_code} {resp.text}")

    # 3. Create Custom Asset (Report)
    print("Creating 'Q4_Report' (Report)...")
    payload_report = {
        "name": "Q4_Report",
        "kind": "Report", # Custom Type
        "location": "s3://warehouse/reports/q4_2025.pdf",
        "properties": {
            "author": "Alice",
            "status": "DRAFT",
            "tags": "finance, internal"
        }
    }
    resp = session.post(f"{BASE_URL}/v1/{catalog_name}/namespaces/{namespace}/assets", headers=headers, json=payload_report)
    if resp.status_code == 201:
        print("Success.")
    else:
        print(f"Failed: {resp.status_code} {resp.text}")

if __name__ == "__main__":
    run()
