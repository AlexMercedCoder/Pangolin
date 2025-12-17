from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, IntegerType, StringType
import pyarrow as pa

# Configuration
# Assuming the catalog 'demo_catalog' already exists and supports credential vending
CATALOG_NAME = "catalog"
API_URL = "http://localhost:8080"

def run_demo():
    print(f"--- Connecting to {CATALOG_NAME} via PyIceberg ---")
    
    # We load the catalog. 
    # Since credential vending is enabled, we don't pass AWS keys here.
    # We DO pass s3.endpoint because we are running on 'localhost' but the server 
    # might know itself as 'minio' or another internal DNS name.
    catalog = load_catalog(
        "local",
        **{
            "type": "rest",
            "uri": f"{API_URL}/v1/{CATALOG_NAME}",
            "header.X-Iceberg-Access-Delegation": "vended-credentials",
        }
    )

    print("Connected. Listing Namespaces...")
    try:
        print(catalog.list_namespaces())
    except Exception as e:
        print(f"Error listing namespaces: {e}")
        return

    # Create/Load a table
    ns = "demo_ns"
    try:
        catalog.create_namespace(ns)
        print(f"Created namespace: {ns}")
    except:
        pass

    table_name = f"{ns}.vending_demo"
    print(f"Creating/Loading table '{table_name}'...")
    
    schema = Schema(
        NestedField(1, "id", IntegerType(), required=True),
        NestedField(2, "data", StringType(), required=False),
    )

    try:
        table = catalog.create_table(table_name, schema=schema)
        print(f"Table created: {table}")
    except Exception:
        table = catalog.load_table(table_name)
        print("Table loaded.")

    # Write Data
    print("Writing data...")
    df = pa.Table.from_pylist([
        {"id": 100, "data": "Vended Credentials Work!"},
        {"id": 200, "data": "Pangolin Rocks"}
    ])
    try:
        table.append(df)
        print("Data written successfully.")
    except Exception as e:
        print(f"Write failed (Check credential vending): {e}")
        return

    # Read Data
    print("Reading data back...")
    result = table.scan().to_arrow()
    print(result.to_pandas())

if __name__ == "__main__":
    run_demo()
