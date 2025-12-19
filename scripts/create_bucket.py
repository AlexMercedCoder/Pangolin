import boto3
from botocore.client import Config

s3 = boto3.client('s3',
    endpoint_url='http://127.0.0.1:9000',
    aws_access_key_id='minioadmin',
    aws_secret_access_key='minioadmin',
    config=Config(signature_version='s3v4', s3={'addressing_style': 'path'}),
    region_name='us-east-1'
)

try:
    s3.create_bucket(Bucket='warehouse')
    print("Created bucket: warehouse")
except Exception as e:
    print(f"Error creating bucket: {e}")
