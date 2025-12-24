from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, IntegerType

def main():
    print("Connecting to Pangolin Catalog 'demo'...")
    
    # Connect to Pangolin
    # Note: Requires 'demo' catalog to be created in UI first
    catalog = load_catalog(
        "pangolin",
        **{
            "type": "rest",
            "uri": "http://localhost:8080/v1/demo",
            "header.X-Iceberg-Access-Delegation": "vended-credentials",
            "header.X-Pangolin-Tenant": "00000000-0000-0000-0000-000000000000", # Explicitly set default tenant
            # MinIO credentials for local access if running outside docker
            "s3.endpoint": "http://localhost:9000",
            "s3.access-key-id": "minioadmin",
            "s3.secret-access-key": "minioadmin",
            "s3.region": "us-east-1",
            "s3.path-style-access": "true",
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
    except Exception:
        table = catalog.load_table("demo_ns.users")
        print(f"Loaded existing table: {table}")

if __name__ == "__main__":
    main()
