import requests
import pyarrow as pa
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType
import logging
logging.basicConfig(level=logging.DEBUG)

def run_test():
    print("1. Logging in as testuser2...")
    try:
        auth_response = requests.post("http://localhost:8080/api/v1/users/login", json={"username": "testuser2", "password": "Password123"})
        auth_response.raise_for_status()
        token = auth_response.json()["token"]
        print(f"   Login successful. Token: {token[:10]}...")
    except Exception as e:
        print(f"   Login failed: {e}")
        if hasattr(e, 'response') and e.response:
            print(f"   Response: {e.response.text}")
        return

    print("\n2. Loading Catalog 'test-catalog'...")
    try:
        catalog = load_catalog(
            "test_catalog",
            **{
                "type": "rest",
                "uri": "http://localhost:8080/v1/test-catalog",

                "token": token,
                "s3.endpoint": "http://192.168.16.2:9000",
                "s3.path-style-access": "true",
                # Relying on vended credentials from Pangolin
            }
        )
        print("   Catalog loaded.")
    except Exception as e:
        print(f"   Failed to load catalog: {e}")
        return

    print("\n3. Creating Namespace 'default'...")
    try:
        catalog.create_namespace("default")
        print("   Namespace created.")
    except Exception as e:
        if "already exists" in str(e).lower():
             print("   Namespace already exists.")
        else:
             print(f"   Failed to create namespace: {e}")

    print("\n4. Creating Table 'default.test_table'...")
    schema = Schema(NestedField(1, "data", StringType(), required=False))
    try:
        try:
            catalog.drop_table("default.test_table")
            print("   Dropped existing table.")
        except:
            pass
            
        table = catalog.create_table("default.test_table", schema=schema)
        print("   Table created.")
    except Exception as e:
        print(f"   Failed to create table: {e}")
        return

    print("\n5. Appending Data...")
    try:
        df = pa.Table.from_pylist([{"data": "hello world"}])
        table.append(df)
        print("   Data appended.")
    except Exception as e:
        print(f"   Failed to append data: {e}")
        return

    print("\n6. Reading Data...")
    try:
        read_df = table.scan().to_arrow()
        print(f"   Read {len(read_df)} rows.")
        print(read_df)
        assert len(read_df) == 1
        print("   Verification SUCCESS!")
    except Exception as e:
        print(f"   Failed to read data: {e}")

if __name__ == "__main__":
    run_test()
