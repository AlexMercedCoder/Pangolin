import requests
import json
import uuid
import time

BASE_URL = "http://localhost:8081"

def wait_for_api():
    print("Waiting for API to be ready...")
    for _ in range(30):
        try:
            requests.get(f"{BASE_URL}/health")
            print("API is ready!")
            return
        except requests.exceptions.ConnectionError:
            time.sleep(2)
    raise Exception("API failed to start")

def test_federated_catalog():
    print("Testing Federated Catalog Creation...")
    
    # 1. Use Default Tenant
    tenant_id = "00000000-0000-0000-0000-000000000000"
    print(f"Using default tenant {tenant_id}")
    
    # Skip tenant creation in NO_AUTH mode
        
    # 2. Create Federated Catalog
    catalog_name = "my_federated_catalog"
    print(f"Creating federated catalog {catalog_name}")
    
    payload = {
        "name": catalog_name,
        "config": {
            "properties": {
                "uri": "http://external-catalog:8181",
                "warehouse": "test-warehouse",
                "token": "test-token-123",
                "custom_property": "custom_value"
            }
        }
    }
    
    # We need to simulate being a Tenant Admin.
    # In No-Auth mode (default for simple local run?), we might need to bypass or set headers.
    # If PANGOLIN_NO_AUTH is set, we are treated as Root? Or we need to pretend?
    # Let's assume No-Auth defaults to Root or we can pass a header.
    # Actually, the handlers check generic permissions.
    # Let's try creating a "Root" session if auth is enabled, or just see if it works.
    # The default local run likely enables Auth unless PANGOLIN_NO_AUTH=true.
    # I should run the server with PANGOLIN_NO_AUTH=true for simplicity.
    
    headers = {"X-Pangolin-Tenant": tenant_id} # If needed, but extension extracts it?
    # Actually, `pangolin_api` extracts tenant from header `X-Pangolin-Tenant` or path?
    # Handlers use `Extension(tenant)`. The auth middleware sets this.
    
    # In "no auth" mode, we usually need to send `X-Pangolin-Tenant`.
    
    res = requests.post(
        f"{BASE_URL}/api/v1/federated-catalogs", 
        json=payload,
        headers={
            "X-Pangolin-Tenant": tenant_id,
            "Authorization": "Bearer root_token" # Mock if no-auth?
        }
    )
    
    print(f"Create Catalog Status: {res.status_code}")
    print(res.text)
    
    if res.status_code == 201:
        data = res.json()
        props = data.get("properties", {})
        if props.get("uri") == "http://external-catalog:8181" and props.get("custom_property") == "custom_value":
            print("✅ SUCCESS: Federated catalog created with correct properties.")
        else:
            print("❌ FAILURE: Properties mismatch.")
            print(data)
    else:
        print("❌ FAILURE: Could not create catalog.")

if __name__ == "__main__":
    wait_for_api()
    test_federated_catalog()
