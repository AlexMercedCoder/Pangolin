import requests
import json

BASE_URL = "http://localhost:8080/api/v1"
AUTH_URL = f"{BASE_URL}/users/login"

def run():
    print("--- Verifying RBAC API ---")
    
    # Login as viewer
    resp = requests.post(AUTH_URL, json={"username": "viewer", "password": "password123"})
    if not resp.ok:
        print(f"Login failed: {resp.text}")
        return
    token = resp.json()['token']
    headers = {"Authorization": f"Bearer {token}"}
    
    print("Logged in as viewer.")
    
    # Search Customers
    print("\nSearching 'customers'...")
    resp = requests.get(f"{BASE_URL}/assets/search?query=customers", headers=headers)
    if resp.ok:
        results = resp.json()
        cust = next((r for r in results if r['name'] == 'customers'), None)
        if cust:
            print(f"Found customers: catalog={cust.get('catalog')}, namespace={cust.get('namespace')}, has_access={cust['has_access']}, discoverable={cust['discoverable']}")
            if cust['has_access'] == True:
                print("PASS: Viewer has access to customers.")
            else:
                print("FAIL: Viewer should have access to customers.")
        else:
            print("FAIL: Customers not found in search")
    else:
        print(f"Search failed: {resp.text}")

    # Search Orders Secure
    print("\nSearching 'orders_secure'...")
    resp = requests.get(f"{BASE_URL}/assets/search?query=orders", headers=headers)
    if resp.ok:
        results = resp.json()
        orders = next((r for r in results if r['name'] == 'orders_secure'), None)
        if orders:
            print(f"Found orders_secure: has_access={orders['has_access']}, discoverable={orders['discoverable']}")
            if orders['has_access'] == False:
                print("PASS: Viewer has NO access to orders_secure.")
            else:
                print("FAIL: Viewer should NOT have access to orders_secure.")
            
            if orders['discoverable'] == True:
                 print("PASS: orders_secure is discoverable.")
            else:
                 print("FAIL: orders_secure should be discoverable.")
        else:
            print("FAIL: orders_secure not found in search")
    else:
        print(f"Search failed: {resp.text}")

    # Search by Tag
    print("\nSearching '#public'...")
    # Using # syntax in query which backend now supports
    resp = requests.get(f"{BASE_URL}/assets/search?query=%23public", headers=headers)
    if resp.ok:
        results = resp.json()
        cust = next((r for r in results if r['name'] == 'customers'), None)
        if cust:
            print(f"Found customers via #public: has_access={cust['has_access']}")
            if cust['has_access'] == True:
                print("PASS: Found public asset.")
            else:
                 print("FAIL: Access check on public asset.")
        else:
            print("FAIL: Did not find 'customers' when searching #public")
    else:
        print(f"Search failed: {resp.text}")

if __name__ == "__main__":
    run()
