import requests
from pyiceberg.catalog import load_catalog
import pyarrow as pa

def verify_merge():
    # 1. Login to get token
    TOKEN_URL = "http://localhost:8080/api/v1/users/login"
    resp = requests.post(TOKEN_URL, json={"username": "tenant_admin", "password": "password"})
    token = resp.json()["token"]

    catalog = load_catalog(
        "default",
        **{
            "uri": "http://localhost:8080/v1/test-catalog",
            "type": "rest",
            "token": token,
            "header.X-Pangolin-Tenant": "b601090d-7bb5-40d0-b9db-82677c60daa1", 
            "s3.endpoint": "http://localhost:9000",
            "s3.access-key-id": "minioadmin",
            "s3.secret-access-key": "minioadmin",
            "s3.region": "us-east-1"
        }
    )

    print("Loading table from main...")
    table = catalog.load_table("test_ns.test_table")
    df = table.scan().to_arrow()
    print("Row count:", len(df))
    print(df.to_pylist())
    
    # Expect 2 rows if merged (id:1 from main, id:2 from branch)
    # Note: Iceberg standard merge/append would add the data files.
    # If branching implementation works, the merge commits from feature-branch should be applied to main.
    
    if len(df) >= 2:
        print("SUCCESS: Data merged.")
    else:
        print("FAILURE: Data not merged or count mismatch.")

if __name__ == "__main__":
    verify_merge()
