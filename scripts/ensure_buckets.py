import boto3
from botocore.config import Config

def create_buckets():
    s3 = boto3.client('s3',
                      endpoint_url='http://localhost:9000',
                      aws_access_key_id='minioadmin',
                      aws_secret_access_key='minioadmin',
                      config=Config(signature_version='s3v4'),
                      region_name='us-east-1')

    buckets = ['warehouse', 'test-bucket']
    for b in buckets:
        try:
            s3.create_bucket(Bucket=b)
            print(f"Created bucket: {b}")
        except Exception as e:
            if "BucketAlreadyOwnedByYou" in str(e) or "BucketAlreadyExists" in str(e):
                print(f"Bucket already exists: {b}")
            else:
                print(f"Error creating bucket {b}: {e}")

if __name__ == "__main__":
    create_buckets()
