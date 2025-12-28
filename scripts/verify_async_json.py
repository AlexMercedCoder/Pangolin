import requests
import json
import time

BASE_URL = "http://localhost:8000"
CATALOG = "default"
NAMESPACE = "test_async_ns"
TABLE = "test_async_table"

def run_test():
    print("--- Verifying Async JSON Offloading (MemoryStore) ---")
    
    # 1. Create Namespace (if not exists)
    # Generic asset creation for namespace isn't strictly required by Iceberg REST sometimes, 
    # but let's assume we might need it or just skip direct to table if namespace is implicit.
    # We'll try creating the table directly.
    
    # 2. Create Table
    print(f"Creating table {NAMESPACE}.{TABLE}...")
    url = f"{BASE_URL}/v1/{CATALOG}/namespaces/{NAMESPACE}/tables"
    payload = {
        "name": TABLE,
        "schema": {
            "type": "struct",
            "fields": [
                {"id": 1, "name": "id", "type": "int", "required": True},
                {"id": 2, "name": "data", "type": "string", "required": False}
            ]
        },
        "location": f"mem://{CATALOG}/{NAMESPACE}/{TABLE}",
        "properties": {"test": "async_json"}
    }
    
    try:
        res = requests.post(url, json=payload)
        if res.status_code == 200:
            print("✅ Create Table success")
        else:
            print(f"❌ Create Table failed: {res.status_code} - {res.text}")
            return
    except Exception as e:
        print(f"❌ Connection failed: {e}")
        return

    # 3. Load Table (Read Metadata)
    print(f"Loading table {NAMESPACE}.{TABLE}...")
    url = f"{BASE_URL}/v1/{CATALOG}/namespaces/{NAMESPACE}/tables/{TABLE}"
    res = requests.get(url)
    
    if res.status_code == 200:
        data = res.json()
        props = data.get("metadata", {}).get("properties", {})
        if props.get("test") == "async_json":
            print("✅ Load Table success (Metadata verified)")
        else:
            print("❌ Load Table mismatch")
    else:
        print(f"❌ Load Table failed: {res.status_code} - {res.text}")

if __name__ == "__main__":
    run_test()
