#!/usr/bin/env python3
"""
Check if session headers are set correctly
"""

from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "test",
    **{
        "uri": "http://localhost:8080",
        "prefix": "analytics",
        "header.X-Pangolin-Tenant": "00000000-0000-0000-0000-000000000001",
    }
)

print("Session headers:", dict(catalog._session.headers))

# Try making a manual request with the session
import requests
response = catalog._session.get("http://localhost:8080/v1/analytics/namespaces")
print(f"Manual request status: {response.status_code}")
print(f"Manual request body: {response.text}")
