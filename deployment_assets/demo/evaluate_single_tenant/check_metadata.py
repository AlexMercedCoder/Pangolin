from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, IntegerType
import time
import sys

def main():
    print("Checking for metadata.json persistence...")
    
    catalog = load_catalog(
        "pangolin",
        **{
            "type": "rest",
            "uri": "http://pangolin-api:8080/v1/demo",
            "header.X-Iceberg-Access-Delegation": "vended-credentials",
            "header.X-Pangolin-Tenant": "00000000-0000-0000-0000-000000000000",
        }
    )

    ns_name = f"meta_check_{int(time.time())}"
    try:
        catalog.create_namespace(ns_name)
    except:
        pass

    schema = Schema(
        NestedField(1, "id", IntegerType(), required=True),
    )

    table_name = f"{ns_name}.meta_table"
    print(f"Creating table '{table_name}'...")
    table = catalog.create_table(table_name, schema=schema)
    print(f"Table Location: {table.location()}")
    print(f"Metadata Location: {table.metadata_location}")

if __name__ == "__main__":
    main()
