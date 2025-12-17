import requests

API_URL = "http://localhost:8080/api/v1"

def bootstrap():
    # Warehouse
    print("Creating Warehouse...")
    requests.post(f"{API_URL}/warehouses", json={
        "name": "warehouse",
        "storage_config": {
            "type": "S3",
            "bucket": "warehouse",
            "region": "us-east-1",
            "access_key_id": "minioadmin",
            "secret_access_key": "minioadmin",
            "endpoint": "http://minio:9000"
        }
    })
    
    # Catalog
    print("Creating Catalog...")
    requests.post(f"{API_URL}/catalogs", json={
        "name": "demo_catalog",
        "warehouse_name": "warehouse",
        "storage_location": "s3://warehouse/demo_catalog",
        "catalog_type": "Local",
        "properties": {}
    })

if __name__ == "__main__":
    bootstrap()
