
import requests
import json

BASE_URL = "http://localhost:8080"
TENANT_A_ID = "b5da4d24-4603-4922-85a6-494fba3cfd9b"
CATALOG_NAME = "catalog_a"

def get_token(username, password):
    resp = requests.post(f"{BASE_URL}/api/v1/users/login", json={"username": username, "password": password})
    resp.raise_for_status()
    return resp.json()["token"]

def main():
    print("Logging in as admin_a...")
    token = get_token("admin_a", "password")
    headers = {"Authorization": f"Bearer {token}"}
    
    # 1. Create Namespace 'data'
    # REST Catalog URL: /v1/{prefix}/{catalog}/namespaces
    # Prefix is catalog_name, tenant context comes from token
    rest_base = f"{BASE_URL}/v1/{CATALOG_NAME}"
    
    print("Creating namespace 'data'...")
    try:
        requests.post(f"{rest_base}/namespaces", json={"namespace": ["data"]}, headers=headers).raise_for_status()
    except requests.exceptions.HTTPError as e:
        if e.response.status_code == 409:
            print("Namespace already exists.")
        else:
            raise

    # 2. Create Table 'table_allow'
    schema = {
        "type": "struct",
        "fields": [{"id": 1, "name": "id", "required": True, "type": "int"}]
    }
    
    print("Creating 'table_allow'...")
    try:
        requests.post(f"{rest_base}/namespaces/data/tables", json={
            "name": "table_allow",
            "schema": schema
        }, headers=headers).raise_for_status()
    except requests.exceptions.HTTPError as e:
        if e.response.status_code == 409:
             print("Table already exists.")
        else:
             raise

    # 3. Create Table 'table_discoverable'
    print("Creating 'table_discoverable'...")
    try:
        requests.post(f"{rest_base}/namespaces/data/tables", json={
            "name": "table_discoverable",
            "schema": schema
        }, headers=headers).raise_for_status()
    except requests.exceptions.HTTPError as e:
        if e.response.status_code == 409:
             print("Table already exists.")
        else:
             raise
             
    print("Tables created successfully.")

if __name__ == "__main__":
    main()
