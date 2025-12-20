from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, IntegerType

def main():
    print("Connecting to Pangolin Catalog 'api_demo'...")
    
    # Connect to Pangolin
    # Note: Requires 'api_demo' catalog to be created in UI or via API first
    catalog = load_catalog(
        "pangolin",
        **{
            "type": "rest",
            "uri": "http://pangolin-api:8080/v1/api_demo",
            "header.X-Iceberg-Access-Delegation": "vended-credentials",
        }
    )

    # List namespaces
    print(f"Namespaces: {catalog.list_namespaces()}")

    # Create namespace
    try:
        catalog.create_namespace("demo_ns")
        print("Created namespace 'demo_ns'")
    except Exception:
        print("Namespace 'demo_ns' already exists")

    # Define Schema
    schema = Schema(
        NestedField(1, "id", IntegerType(), required=True),
        NestedField(2, "name", StringType(), required=False),
    )

    # Create table
    try:
        table = catalog.create_table("demo_ns.users", schema=schema)
        print(f"Created table: {table}")
    except Exception as e:
        print(f"Table creation error or table exists: {e}")
        try:
            table = catalog.load_table("demo_ns.users")
            print(f"Loaded existing table: {table}")
        except Exception as load_e:
            print(f"Failed to load table: {load_e}")

if __name__ == "__main__":
    main()
