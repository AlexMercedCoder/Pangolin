#!/usr/bin/env python3
"""
Debug PyIceberg requests by monkey-patching the session
"""

import requests
from pyiceberg.catalog import load_catalog

# Monkey-patch requests to log all requests
original_request = requests.Session.request

def logged_request(self, method, url, **kwargs):
    print(f"\n=== REQUEST ===")
    print(f"Method: {method}")
    print(f"URL: {url}")
    print(f"Headers: {kwargs.get('headers', {})}")
    print(f"Params: {kwargs.get('params', {})}")
    print(f"================\n")
    
    response = original_request(self, method, url, **kwargs)
    
    print(f"\n=== RESPONSE ===")
    print(f"Status: {response.status_code}")
    print(f"Headers: {dict(response.headers)}")
    print(f"Body: {response.text[:200]}")
    print(f"=================\n")
    
    return response

requests.Session.request = logged_request

# Now try PyIceberg
print("Creating PyIceberg catalog...")
catalog = load_catalog(
    "test",
    **{
        "uri": "http://localhost:8080",
        "prefix": "analytics",
        "header.X-Pangolin-Tenant": "00000000-0000-0000-0000-000000000001",
    }
)

print("\nCatalog created. Attempting to list namespaces...")
try:
    namespaces = catalog.list_namespaces()
    print(f"✓ Success! Namespaces: {namespaces}")
except Exception as e:
    print(f"✗ Failed: {e}")
