
import requests
import json
import time
import pyarrow as pa
from pyiceberg.catalog import load_catalog

API_URL = "http://localhost:8080"
MANAGEMENT_URL = f"{API_URL}/api/v1"
TENANT_ID = "00000000-0000-0000-0000-000000000000"
CATALOG_NAME = "sqlite_catalog"
WAREHOUSE_NAME = "sqlite_warehouse"

# Headers for management API
headers = {
    "X-Pangolin-Tenant": TENANT_ID,
    "Content-Type": "application/json"
}

def wait_for_api():
    print(f"Waiting for API at {API_URL}...")
    for i in range(30):
        try:
            requests.get(f"{API_URL}/health")
            print("API is up!")
            return
        except requests.exceptions.ConnectionError:
            print(f"Waiting... {i}")
            time.sleep(1)
    raise Exception("API failed to start")

def create_warehouse():
    print(f"Creating Warehouse '{WAREHOUSE_NAME}'...")
    payload = {
        "name": WAREHOUSE_NAME,
        "storage_type": "s3",
        "storage_config": {
            "endpoint": "http://localhost:9000",
            "bucket": "warehouse",
            "region": "us-east-1",
            "access_key_id": "minioadmin",
            "secret_access_key": "minioadmin",
            "allow_http": "true" 
        }
    }
    resp = requests.post(f"{MANAGEMENT_URL}/warehouses", headers=headers, json=payload)
    if resp.status_code in [200, 201]:
        print("Warehouse created.")
    elif resp.status_code == 409:
        print("Warehouse already exists.")
    else:
        print(f"Warehouse creation failed: {resp.status_code} {resp.text}")
        exit(1)

def create_catalog():
    print(f"Creating Catalog '{CATALOG_NAME}'...")
    payload = {
        "name": CATALOG_NAME,
        "catalog_type": "managed",
        "warehouse_name": WAREHOUSE_NAME
    }
    resp = requests.post(f"{MANAGEMENT_URL}/catalogs", headers=headers, json=payload)
    if resp.status_code in [200, 201]:
        print("Catalog created.")
    elif resp.status_code == 409:
        print("Catalog already exists.")
    else:
        print(f"Catalog creation failed: {resp.status_code} {resp.text}")
        exit(1)

def run_iceberg_tests():
    print("\n--- Starting PyIceberg Tests ---")
    
    # Load Catalog
    catalog = load_catalog(
        "local",
        **{
            "type": "rest",
            "uri": f"{API_URL}/v1/{CATALOG_NAME}",
            "header.X-Pangolin-Tenant": TENANT_ID,
            "header.X-Iceberg-Access-Delegation": "vended-credentials",
            "s3.endpoint": "http://localhost:9000",
            "s3.region": "us-east-1",
            "s3.path-style-access": "true"
        }
    )

    # 1. Create Namespace
    ns_name = "test_ns"
    try:
        catalog.create_namespace(ns_name)
        print(f"Namespace '{ns_name}' created.")
    except Exception as e:
        print(f"Namespace creation note: {e}")

    # 2. Create Table
    table_name = f"{ns_name}.test_table"
    schema = pa.schema([
        ("id", pa.int64()),
        ("name", pa.string()),
        ("ts", pa.timestamp('us'))
    ])

    try:
        catalog.drop_table(table_name)
    except:
        pass

    print(f"Creating table '{table_name}'...")
    table = catalog.create_table(table_name, schema=schema)
    print("Table created successfully.")

    # 3. Insert Data
    print("Inserting data...")
    df = pa.Table.from_pylist([
        {"id": 1, "name": "Alice", "ts": 1000},
        {"id": 2, "name": "Bob", "ts": 2000},
    ], schema=schema)
    
    table.append(df)
    print("Data inserted.")

    # 4. Read Data
    print("Reading data...")
    # Refresh table to see changes
    table.refresh()
    read_df = table.scan().to_pandas()
    print("Data Read:")
    print(read_df)
    
    if len(read_df) != 2:
        raise Exception("Row count mismatch!")

    # 5. Delete Table
    print("Deleting table...")
    catalog.drop_table(table_name)
    print("Table deleted.")
    
    # 6. Delete Namespace
    print("Deleting namespace...")
    catalog.drop_namespace(ns_name)
    print("Namespace deleted.")

    print("\nSUCCESS: All SQLite + MinIO tests passed!")

if __name__ == "__main__":
    wait_for_api()
    create_warehouse()
    create_catalog()
    run_iceberg_tests()
