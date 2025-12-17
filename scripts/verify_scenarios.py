import argparse
import sys
import os
import boto3
from pyiceberg.catalog import load_catalog
from pyiceberg.schema import Schema
from pyiceberg.types import NestedField, IntegerType, StringType
import pyarrow as pa
import requests

def get_args():
    parser = argparse.ArgumentParser(description="Verify Pangolin PyIceberg Scenarios")
    parser.add_argument("--mode", choices=["no_auth", "auth"], required=True)
    parser.add_argument("--vending", action="store_true", help="Enable credential vending")
    parser.add_argument("--api-url", default="http://localhost:8080", help="Pangolin API URL")
    # For local testing, we typically need to override the endpoint to reach MinIO from host
    parser.add_argument("--s3-endpoint", default="http://localhost:9000", help="Local S3 Endpoint override")
    parser.add_argument("--catalog", default="my_catalog", help="Name of the catalog to use")
    parser.add_argument("--username", default="admin", help="Username for Auth")
    parser.add_argument("--password", default="password", help="Password for Auth")
    return parser.parse_args()

def authenticate(api_url, username, password):
    print(f"Authenticating as {username}...")
    try:
        resp = requests.post(f"{api_url}/api/v1/users/login", json={
            "username": username,
            "password": password
        })
        resp.raise_for_status()
        return resp.json()["token"]
    except Exception as e:
        print(f"Authentication failed: {e}")
        sys.exit(1)

def run_test(args):
    print(f"--- Running Test: Mode={args.mode}, Vending={args.vending} ---")
    
    properties = {
        "type": "rest",
        "uri": f"{args.api_url}/v1/{args.catalog}",
        # In a real production scenario with vending, we wouldn't specify this.
        # But for local docker-compose, we need to tell PyIceberg to talk to localhost:9000
        "s3.endpoint": args.s3_endpoint, 
    }

    # 1. Auth Setup
    if args.mode == "auth":
        token = authenticate(args.api_url, args.username, args.password)
        properties["token"] = token
        # Get Tenant ID if needed? The Login response usually gives it.
        # But verify_scenarios implies we might need it. 
        # For simplicity, assuming the user is admin or context is inferred.
        # If needed, we'd extract tenant_id from auth response and add header X-Pangolin-Tenant.

    # 2. Credential Setup
    if args.vending:
        print("Using Credential Vending...")
        properties["header.X-Iceberg-Access-Delegation"] = "vended-credentials"
    else:
        print("Using Client Credentials...")
        # Hardcoded for the standard MinIO demo setup
        properties["s3.access-key-id"] = "minioadmin"
        properties["s3.secret-access-key"] = "minioadmin"
        properties["s3.path-style-access"] = "true"
        properties["s3.region"] = "us-east-1"


    # 3. Connect
    print(f"Connecting to Catalog '{args.catalog}'...")
    try:
        catalog = load_catalog("local", **properties)
        print("Connected.")
    except Exception as e:
        print(f"Failed to load catalog: {e}")
        sys.exit(1)

    # 4. Operations
    ns = "test_ns"
    try:
        print("Listing namespaces...")
        print(catalog.list_namespaces())
        
        try: 
            catalog.create_namespace(ns) 
        except: 
            pass

        tbl_name = f"{ns}.test_table_{args.mode}_{'vended' if args.vending else 'client'}"
        print(f"Creating table '{tbl_name}'...")
        
        schema = Schema(
            NestedField(1, "id", IntegerType()),
            NestedField(2, "data", StringType())
        )
        
        try:
            table = catalog.create_table(tbl_name, schema=schema)
        except Exception as e:
            print(f"Create failed ({e}), trying load...")
            table = catalog.load_table(tbl_name)
            
        try:
            print("Writing data...")
            tbl = table
            # Check if it's a VAMS table (REST table) or filesystem table
            # PyIceberg create_table returns the Table instance. 
            # We can append directly.
            
            # Simple Arrow Table
            df = pa.Table.from_pylist([{"id": 1, "data": "test"}])
            
            tbl.append(df)
            print("Write success.")
            
        except Exception as e:
            print(f"Write failed: {e}")
            # Debug: List S3 objects
            try:
                print("--- DEBUG: Listing S3 Objects in 'warehouse' ---")
                s3 = boto3.client('s3', 
                    endpoint_url=args.s3_endpoint,
                    aws_access_key_id=os.environ.get('AWS_ACCESS_KEY_ID', 'minioadmin'),
                    aws_secret_access_key=os.environ.get('AWS_SECRET_ACCESS_KEY', 'minioadmin'),
                    region_name='us-east-1'
                )
                response = s3.list_objects_v2(Bucket='warehouse')
                if 'Contents' in response:
                    for obj in response['Contents']:
                        print(f" - {obj['Key']} ({obj['Size']} bytes)")
                else:
                    print("Bucket 'warehouse' is empty.")
            except Exception as s3e:
                print(f"Failed to list S3 objects: {s3e}")
                
            sys.exit(1)
        
        print("Reading data...")
        res = table.scan().to_arrow()
        print(f"Read success. Rows: {len(res)}")
        
    except Exception as e:
        print(f"Operation failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    args = get_args()
    run_test(args)
