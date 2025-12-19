import requests
import json

BASE_URL = "http://localhost:8080/api/v1"
TENANT_ID = "b601090d-7bb5-40d0-b9db-82677c60daa1"

def get_asset_id():
    # 1. Login
    resp = requests.post(f"{BASE_URL}/users/login", json={"username": "tenant_admin", "password": "password"})
    token = resp.json()["token"]
    headers = {"Authorization": f"Bearer {token}", "X-Pangolin-Tenant": TENANT_ID}

    # 2. List Catalogs
    cat_resp = requests.get(f"{BASE_URL}/catalogs", headers=headers)
    catalogs = cat_resp.json()
    if not catalogs:
        print("No catalogs found.")
        return

    catalog_name = catalogs[0]["name"]
    print(f"Using catalog: {catalog_name}")

    # 3. List Namespaces (Mocking finding namespaces... actually typically we list tables)
    # The API might verify search or list assets.
    # Let's try to search first to see if any assets exist via API (even if UI search is broken)
    # Or just guess the namespace/table from previous tests: test_ns.test_table
    
    # We need the asset ID. 
    # Let's try to get the asset by path if that endpoint exists or list assets?
    # Based on handlers, there isn't a simple "list all assets" endpoint for the whole tenant easily visible.
    # But `src/iceberg_handlers.rs` usually handles `load_table`.
    # AND `pangolin_handlers.rs` has `list_catalogs`.
    
    # Let's try to use the `search` endpoint via script to see if it works at all.
    search_resp = requests.get(f"{BASE_URL}/assets/search?query=test_table", headers=headers)
    if search_resp.ok:
        data = search_resp.json()
        if isinstance(data, list):
            results = data
        else:
            results = data.get("results", [])
        
        if results:
            print("ASSET_ID:", results[0]["id"])
            return results[0]["id"]
        else:
            print("Search returned no results.")
    else:
        print(f"Search failed: {search_resp.status_code}")

if __name__ == "__main__":
    get_asset_id()
