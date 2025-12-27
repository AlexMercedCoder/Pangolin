from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, IntegerType
import pyarrow as pa
import time

def main():
    print("Connecting to Pangolin Catalog 'demo'...")
    print("NOTE: This script should be run inside the 'datanotebook' container to access MinIO.")
    
    # Connect to Pangolin
    # Note: Requires 'demo' catalog to be created in UI first
    catalog = load_catalog(
        "pangolin",
        **{
            "type": "rest",
            "uri": "http://pangolin-api:8080/v1/demo",
            "header.X-Iceberg-Access-Delegation": "vended-credentials",
            "header.X-Pangolin-Tenant": "00000000-0000-0000-0000-000000000000", # Explicitly set default tenant
            
            # We rely on Pangolin's Vended Credentials for S3 access.
            # These point to 'http://minio:9000' which is accessible within the Docker network.
            # Do NOT override s3.endpoint here unless you are configuring /etc/hosts on your machine.
            # Ensure your Warehouse creation step included "s3.path-style-access": "true" for MinIO.
        }
    )

    # List namespaces
    print(f"Namespaces: {catalog.list_namespaces()}")

    # Create namespace
    ns_name = "demo_ns"
    try:
        catalog.create_namespace(ns_name)
        print(f"Created namespace '{ns_name}'")
    except Exception:
        print(f"Namespace '{ns_name}' already exists")

    # Define Schema
    schema = Schema(
        NestedField(1, "id", IntegerType(), required=True),
        NestedField(2, "name", StringType(), required=False),
    )

    # Create table
    table_name = f"{ns_name}.users"
    try:
        table = catalog.create_table(table_name, schema=schema)
        print(f"Created table: {table}")
    except Exception:
        table = catalog.load_table(table_name)
        print(f"Loaded existing table: {table}")

    print(f"Table Location: {table.location()}")

    # Write data to verify persistence in MinIO
    print("Appending verification data...")
    try:
        df = pa.Table.from_pylist(
            [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}], 
            schema=schema.as_arrow()
        )
        table.append(df)
        print("✅ Data appended successfully! Check MinIO bucket 'warehouse' for files.")
    except Exception as e:
        print(f"❌ Failed to append data: {e}")
        print("Ensure you are running this script inside the Docker container (datanotebook).")

if __name__ == "__main__":
    main()
