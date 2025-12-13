#!/usr/bin/env python3
"""
Test PyIceberg without authentication using default tenant
"""

from pyiceberg.catalog import load_catalog

print("Testing PyIceberg with default tenant (no authentication)...")

try:
    # Connect without any authentication
    catalog = load_catalog(
        "pangolin",
        **{
            "uri": "http://localhost:8080",
            "prefix": "analytics",
            # No token - will use default tenant
        }
    )
    
    print(f"✓ Connected to catalog: {catalog}")
    
    # Try to list namespaces
    namespaces = catalog.list_namespaces()
    print(f"✓ Namespaces: {namespaces}")
    
    # Try to create a namespace
    catalog.create_namespace("test_default_tenant")
    print("✓ Created namespace: test_default_tenant")
    
    # List again
    namespaces = catalog.list_namespaces()
    print(f"✓ Updated namespaces: {namespaces}")
    
    print("\n✅ SUCCESS! PyIceberg works without authentication using default tenant")
    
except Exception as e:
    print(f"\n❌ FAILED: {e}")
    import traceback
    traceback.print_exc()
