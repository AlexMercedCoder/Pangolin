import pyarrow as pa
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, StringType, IntegerType

# Configuration
CATALOG_NAME = "my-catalog"
TOKEN = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIwMDAwMDAwMC0wMDAwLTAwMDAtMDAwMC0wMDAwMDAwMDAwMDAiLCJqdGkiOiI4OTRiY2I4ZC0wZWZhLTRmM2EtYWJjOS1mNWE2NWQ5ZDJlNjQiLCJ1c2VybmFtZSI6ImFkbWluIiwidGVuYW50X2lkIjoiMDAwMDAwMDAtMDAwMC0wMDAwLTAwMDAtMDAwMDAwMDAwMDAwIiwicm9sZSI6InRlbmFudC1hZG1pbiIsImV4cCI6MTc2NjU4MjMyMywiaWF0IjoxNzY2NDk1OTIzfQ.gxJi5OSFjXqe2W7dACCTOODf8hYSvEMhX7tTpleGj8Y"
URI = "http://localhost:8080/v1/my-catalog"
TENANT_ID = "00000000-0000-0000-0000-000000000000"

def run_test():
    print(f"Connecting to catalog: {CATALOG_NAME}...")
    
    # Load catalog with vended credentials enabled
    catalog = load_catalog(
        CATALOG_NAME,
        **{
            "type": "rest",
            "uri": URI,
            "token": TOKEN,
            "header.X-Iceberg-Access-Delegation": "vended-credentials",
            "header.X-Pangolin-Tenant": TENANT_ID,
            "s3.endpoint": "http://localhost:9000", # Fallback if not vended correctly
            "s3.access-key-id": "minioadmin",
            "s3.secret-access-key": "minioadmin",
        }
    )

    namespace = "qa"
    table_name = f"{namespace}.test_table"

    try:
        # 1. Create Namespace
        print(f"Creating namespace '{namespace}'...")
        catalog.create_namespace(namespace)
    except Exception as e:
        print(f"Namespace might already exist: {e}")

    # 2. Define Schema
    schema = Schema(
        NestedField(field_id=1, name="id", field_type=IntegerType(), required=True),
        NestedField(field_id=2, name="data", field_type=StringType(), required=True),
    )

    try:
        # 3. Create Table
        print(f"Creating table '{table_name}'...")
        if table_name in [t[0]+"."+t[1] for t in catalog.list_tables(namespace)]:
            catalog.drop_table(table_name)
        
        table = catalog.create_table(
            table_name,
            schema=schema
        )
        print("Table created successfully.")

        # 4. Write Data
        print("Writing data to table...")
        df = pa.Table.from_pydict({
            "id": [1, 2, 3],
            "data": ["A", "B", "C"]
        })
        table.append(df)
        print("Data written successfully.")

        # 5. Read Data
        print("Reading data back...")
        # PyIceberg 0.7.0+ uses scan()
        scan = table.scan()
        result_df = scan.to_arrow()
        print("Data read successfully:")
        print(result_df)

        # 6. Clean up
        # print(f"Dropping table '{table_name}'...")
        # catalog.drop_table(table_name)
        
    except Exception as e:
        print(f"Error during test: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    run_test()
