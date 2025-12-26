#!/usr/bin/env python3
"""
Script to automatically split MemoryStore into modular structure.
Extracts methods from memory.rs.bak and creates individual module files.
"""

import re
import os
from pathlib import Path

# Define method groupings
METHOD_GROUPS = {
    'tenants.rs': [
        'create_tenant', 'get_tenant', 'list_tenants', 'update_tenant', 'delete_tenant'
    ],
    'warehouses.rs': [
        'create_warehouse', 'get_warehouse', 'list_warehouses', 'update_warehouse', 'delete_warehouse'
    ],
    'catalogs.rs': [
        'create_catalog', 'get_catalog', 'list_catalogs', 'update_catalog', 'delete_catalog'
    ],
    'namespaces.rs': [
        'create_namespace', 'list_namespaces', 'get_namespace', 'delete_namespace', 'update_namespace_properties'
    ],
    'assets.rs': [
        'create_asset', 'get_asset', 'get_asset_by_id', 'list_assets', 'delete_asset', 
        'rename_asset', 'count_assets'
    ],
    'branches.rs': [
        'create_branch', 'get_branch', 'list_branches', 'delete_branch', 'merge_branch'
    ],
    'tags.rs': [
        'create_tag', 'get_tag', 'list_tags', 'delete_tag'
    ],
    'commits.rs': [
        'create_commit', 'get_commit'
    ],
    'io.rs': [
        'get_metadata_location', 'update_metadata_location', 'read_file', 'write_file',
        'expire_snapshots', 'remove_orphan_files', 'read_file_uncached',
        'get_warehouse_for_location', 'get_object_store_cache_key', 'extract_object_store_path'
    ],
    'audit.rs': [
        'log_audit_event', 'list_audit_events', 'get_audit_event', 'count_audit_events'
    ],
    'users.rs': [
        'create_user', 'get_user', 'get_user_by_username', 'list_users', 'update_user', 'delete_user'
    ],
    'service_users.rs': [
        'create_service_user', 'get_service_user', 'list_service_users', 'update_service_user', 'delete_service_user'
    ],
    'roles.rs': [
        'create_role', 'get_role', 'list_roles', 'update_role', 'delete_role',
        'assign_role_to_user', 'revoke_role_from_user', 'get_user_roles'
    ],
    'permissions.rs': [
        'grant_permission', 'revoke_permission', 'list_permissions', 'check_permission'
    ],
    'tokens.rs': [
        'store_token', 'get_token', 'list_active_tokens', 'revoke_token', 'validate_token'
    ],
    'business_metadata.rs': [
        'upsert_business_metadata', 'get_business_metadata', 'delete_business_metadata',
        'search_assets', 'search_catalogs', 'search_namespaces', 'search_branches'
    ],
    'access_requests.rs': [
        'create_access_request', 'get_access_request', 'list_access_requests',
        'approve_access_request', 'deny_access_request'
    ],
    'merge.rs': [
        'create_merge_operation', 'get_merge_operation', 'list_merge_operations',
        'add_merge_conflict', 'get_merge_conflicts', 'resolve_conflict',
        'complete_merge', 'abort_merge'
    ],
    'federated.rs': [
        'get_federated_catalog_stats', 'update_federated_catalog_stats'
    ],
    'settings.rs': [
        'get_system_settings', 'update_system_settings'
    ],
    'signer.rs': [
        'get_table_credentials', 'presign_get'
    ],
}

def extract_method(content, method_name):
    """Extract a complete method from the source content."""
    # Pattern to match async fn method_name
    pattern = rf'(async fn {re.escape(method_name)}\([^{{]*\{{)'
    
    match = re.search(pattern, content, re.MULTILINE | re.DOTALL)
    if not match:
        return None
    
    start_pos = match.start()
    
    # Find the matching closing brace
    brace_count = 0
    in_method = False
    end_pos = start_pos
    
    for i in range(start_pos, len(content)):
        char = content[i]
        if char == '{':
            brace_count += 1
            in_method = True
        elif char == '}':
            brace_count -= 1
            if in_method and brace_count == 0:
                end_pos = i + 1
                break
    
    if end_pos > start_pos:
        return content[start_pos:end_pos]
    
    return None

def create_module_file(module_name, methods, source_content):
    """Create a module file with extracted methods."""
    output_dir = Path('src/memory')
    output_file = output_dir / module_name
    
    # Module header
    header = f'''use super::MemoryStore;
use anyhow::Result;
use uuid::Uuid;
use async_trait::async_trait;

impl MemoryStore {{
'''
    
    # Extract all methods
    method_bodies = []
    for method in methods:
        method_code = extract_method(source_content, method)
        if method_code:
            # Add proper indentation
            indented = '\n'.join('    ' + line if line.strip() else line 
                                for line in method_code.split('\n'))
            method_bodies.append(indented)
        else:
            print(f"Warning: Could not find method '{method}' for {module_name}")
    
    # Module footer
    footer = '}\n'
    
    # Write file
    content = header + '\n'.join(method_bodies) + '\n' + footer
    
    with open(output_file, 'w') as f:
        f.write(content)
    
    print(f"Created {module_name} with {len(method_bodies)} methods")

def main():
    # Read source file
    source_file = Path('src/memory.rs.bak')
    with open(source_file, 'r') as f:
        source_content = f.read()
    
    # Create each module
    for module_name, methods in METHOD_GROUPS.items():
        create_module_file(module_name, methods, source_content)
    
    print("\nModule creation complete!")

if __name__ == '__main__':
    main()
