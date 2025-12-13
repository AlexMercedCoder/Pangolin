from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, IntegerType, StringType
import pyarrow as pa

# Configure catalog
catalog = load_catalog(
    "pangolin",
    **{
        "type": "rest",
        "uri": "http://localhost:8080/v1/warehouse",
        "s3.endpoint": "http://localhost:9000",
        "s3.access-key-id": "minioadmin",
        "s3.secret-access-key": "minioadmin",
    }
)

# Define schema
schema = Schema(
    NestedField(1, "id", IntegerType(), required=True),
    NestedField(2, "data", StringType(), required=False),
    NestedField(3, "category", StringType(), required=False),
)

# Create Sales Namespace and Tables
try:
    catalog.create_namespace("sales")
except:
    pass

try:
    table = catalog.create_table(
        "sales.customers",
        schema=schema,
    )
    table.append(pa.Table.from_pylist([
        {"id": 1, "data": "Alice", "category": "Premium"},
        {"id": 2, "data": "Bob", "category": "Standard"},
    ]))
    print("Created sales.customers")
except Exception as e:
    print(f"sales.customers might exist: {e}")

try:
    catalog.create_table(
        "sales.orders",
        schema=schema,
    )
    print("Created sales.orders")
except Exception as e:
    print(f"sales.orders might exist: {e}")

# Create Marketing Namespace and Tables
try:
    catalog.create_namespace("marketing")
except:
    pass

try:
    catalog.create_table(
        "marketing.campaigns",
        schema=schema,
    )
    print("Created marketing.campaigns")
except Exception as e:
    print(f"marketing.campaigns might exist: {e}")

print("Seeding complete!")
