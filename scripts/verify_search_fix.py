import requests
import json
import sys

BASE_URL = "http://localhost:8080"

def get_token():
    resp = requests.post(f"{BASE_URL}/api/v1/users/login", json={"username": "tenant_admin", "password": "password"})
    if not resp.ok:
        print(f"Login failed: {resp.text}")
        sys.exit(1)
    return resp.json()["token"]

def verify_search():
    token = get_token()
    headers = {"Authorization": f"Bearer {token}"}
    
    print("Calling Search API...")
    resp = requests.get(f"{BASE_URL}/api/v1/assets/search?query=anything", headers=headers)
    
    if not resp.ok:
        print(f"Search failed: {resp.status_code} {resp.text}")
        sys.exit(1)
        
    data = resp.json()
    print(f"Response type: {type(data)}")
    
    if isinstance(data, list):
        print("PASS: Response is an Array.")
    elif isinstance(data, dict) and "results" in data:
         print("WARN: Response is Object with results (Old behavior? or Wrapper added?).")
         # If I changed the UI to accept array, and backend returns array, this is good.
         # But wait, I DID NOT change the backend search handler to return array?
         # I checked `pangolin_ui` search page, it was receiving array.
         # The backend `pangolin_api/src/business_metadata_handlers.rs` `search_assets` returns `Json(mapped_results)`.
         # `mapped_results` is a `Vec<_>`. So it returns Array.
         # So checking for list is correct.
    else:
        print(f"FAIL: Unexpected response format: {data}")
        sys.exit(1)

if __name__ == "__main__":
    verify_search()
