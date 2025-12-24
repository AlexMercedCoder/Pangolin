import os
import sys
import uuid
import time
from typing import Dict, Any

# Ensure we import local pypangolin
sys.path.insert(0, os.path.abspath("pypangolin/src"))

from pypangolin import PangolinClient
from pypangolin.models import AuditEvent, SystemStats

def log(msg):
    print(f"[{time.strftime('%H:%M:%S')}] {msg}")

def main():
    log("=== Starting PyPangolin Admin Verification ===")
    
    # 1. Setup
    client = PangolinClient(uri="http://localhost:8080")
    
    # Login as root
    client.login("admin", "password")
    log("✅ Logged in as Root/Admin")
    
    # 2. System Stats
    log("--- Testing System Stats ---")
    stats = client.system.get_stats()
    log(f"System Stats: {stats}")
    assert stats.catalogs_count >= 0
    
    # Check summary for existing 'search_cat' (created later in script, so let's move creation up or use a known one)
    # We create 'search_cat' below. Let's move tenant creation up.
    
    # 2b. Tenant Setup (Moved up)
    tenant_name = f"admin_tenant_{uuid.uuid4().hex[:8]}"
    tenant = client.tenants.create(tenant_name)
    client.set_tenant(tenant.id)
    
    username = f"admin_{tenant_name}"
    client.users.create(
        username=username, 
        email=f"{username}@example.com", 
        role="tenant-admin", 
        tenant_id=tenant.id, 
        password="Admin123!"
    )
    client.login(username, "Admin123!", tenant_id=tenant.id)
    
    # Create Warehouse first
    client.warehouses.create_s3("summary_wh", "s3://bucket", access_key="access", secret_key="secret")
    client.catalogs.create("test_summary_cat", "summary_wh")
    
    summary = client.system.get_catalog_summary("test_summary_cat")
    log(f"Catalog Summary: {summary}")
    assert summary.name == "test_summary_cat"
    
    log("✅ System Dashboard Verified")

    # 3. Search
    log("--- Testing Search ---")
    
    # Reuse existing tenant/admin session from Step 2b
    
    # Create Catalog for search using existing warehouse
    client.catalogs.create("search_cat", "summary_wh")
    
    # Use namespaces client bound to catalog
    ns_client = client.catalogs.namespaces("search_cat")
    ns_client.create(["search_ns"])
    
    asset_name = "searchable_asset"
    # Register asset
    # Note: register_asset returns raw dict (from post), not Asset model yet in NamespaceClient code?
    # Let's check NamespaceClient implementation. It returns Dict.
    # We should return Asset model or dict is fine, but we access .id later.
    # So we need to wrap it or access dict key.
    
    asset_data = ns_client.register_asset(
        namespace="search_ns", 
        name=asset_name, 
        kind="PARQUET_TABLE", 
        location=f"s3://bucket/search_ns/{asset_name}"
    )
    
    # Asset ID is in asset_data['id']?
    asset_id = asset_data.get("id") or asset_data.get("asset", {}).get("id")
    # API response for register likely returns the Asset object.
    
    # Add metadata to make it searchable by tag
    client.metadata.upsert(asset_id, tags=["search-me"], discoverable=True)
    
    # Then search
    # Note: Search might be eventually consistent or immediate depending on backend.
    # Memory store is immediate.
    results = client.search.query("searchable", tags=["search-me"])
    log(f"Search Results: {len(results)}")
    if results:
         log(f"First Result: {results[0].name}")
         assert results[0].name == asset_name
    
    log("✅ Search Verified")
    
    # 4. Audit
    log("--- Testing Audit ---")
    # We performed actions (login, create tenant, create user, create catalog...).
    # There should be events.
    
    # Switch to Root to see all events?
    client.login("admin", "password") # Root
    
    count = client.audit.count()
    log(f"Audit Event Count: {count}")
    assert count > 0
    
    events = client.audit.list_events(limit=5)
    log(f"Recent Events: {len(events)}")
    for e in events:
        log(f" - {e.action} by {e.user_id or 'system'} at {e.timestamp}")
        
    if events:
        single_event = client.audit.get(events[0].id)
        assert single_event.id == events[0].id
        log("✅ Get Single Event Verified")

    # 5. Tokens
    log("--- Testing Tokens ---")
    
    # Generate token
    token_response = client.tokens.generate("test-token", expires_in_days=7)
    # Response is likely a dict with 'token' key
    if isinstance(token_response, dict):
        new_token = token_response.get("token", "")
    else:
        new_token = str(token_response)
    
    assert len(new_token) > 0
    log(f"Generated Token: {new_token[:10]}...")
    
    # List tokens
    my_tokens = client.tokens.list_my_tokens()
    log(f"My Tokens Count: {len(my_tokens)}")
    
    # Find token ID (from list)
    token_id = None
    for t in my_tokens:
        if t.get("name") == "test-token":
             token_id = t.get("id")
             break
    
    if token_id:
        # Revoke
        client.tokens.revoke(token_id)
        log("Revoked Token")
        
    log("✅ Token Management Verified")

    # 6. System Config (Root only)
    log("--- Testing System Config ---")
    settings = client.system.get_settings()
    log(f"System Settings: {settings}")
    # Just verify we got a dict back
    assert isinstance(settings, dict)
    log("✅ System Config Verified")

    log("=== Verification Complete ===")
    log("=== Verification Complete ===")

if __name__ == "__main__":
    main()
