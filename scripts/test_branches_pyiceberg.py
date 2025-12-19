from pyiceberg.catalog import load_catalog
import pyarrow as pa
import sys

import requests

def verify_branches():
    # 1. Login to get token
    TOKEN_URL = "http://localhost:8080/api/v1/users/login"
    resp = requests.post(TOKEN_URL, json={"username": "tenant_admin", "password": "password"})
    if not resp.ok:
        print(f"Login failed: {resp.text}")
        return
    token = resp.json()["token"]

    # Load catalog (assumes setup_test_env.py ran successfully)
    # Using client-credentials flow to bypass potential docker networking issues with vending
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

    # 1. Create/Write to Main
    print("Writing to main...")
    try:
        table = catalog.load_table("test_ns.test_table")
    except:
        catalog.create_namespace("test_ns")
        schema = pa.schema([("id", pa.int64()), ("data", pa.string())])
        table = catalog.create_table("test_ns.test_table", schema=schema)

    df = pa.Table.from_pylist([{"id": 1, "data": "main data"}])
    table.append(df)
    print("Wrote to main.")

    # 2. Verify Feature Branch (Assuming created in UI)
    # Note: PyIceberg support for branching via REST is typically done via ref updates or specific branch loading
    # For now, we'll try to load the table at the branch snapshot if possible, or assume the catalog URI handles the branch context if we were to support that
    # The Pangolin API might handle branch context via a header or URL path?
    # Based on previous docs, branches are managed via the catalog context.
    
    # Actually, for standard REST catalogs, branching is often part of the table identifier or separate APIs.
    # If Pangolin supports `X-Pangolin-Branch` header:
    
    print("Writing to feature-branch...")
    branch_catalog = load_catalog(
        "branch",
        **{
            "uri": "http://localhost:8080/v1/test-catalog",
            "type": "rest",
            "token": token,
            "header.X-Pangolin-Tenant": "b601090d-7bb5-40d0-b9db-82677c60daa1",
            "header.X-Pangolin-Branch": "feature-branch", # HYPOTHETICAL: Verify if this is how Pangolin passes branch context
            "s3.endpoint": "http://localhost:9000",
            "s3.access-key-id": "minioadmin",
            "s3.secret-access-key": "minioadmin",
            "s3.region": "us-east-1"
        }
    )
    
    try:
        branch_table = branch_catalog.load_table("test_ns.test_table")
        df_branch = pa.Table.from_pylist([{"id": 2, "data": "branch data"}])
        branch_table.append(df_branch)
        print("Wrote to feature-branch.")
    except Exception as e:
        print(f"Failed to write to feature branch: {e}")

if __name__ == "__main__":
    verify_branches()
