#!/usr/bin/env python3
"""
Script to fix all generated modules by adding imports and renaming methods to _internal suffix.
"""

import re
import os
from pathlib import Path

def fix_module_file(filepath):
    """Fix a single module file by adding imports and renaming methods."""
    with open(filepath, 'r') as f:
        content = f.read()
    
    # Skip if already has proper header
    if 'use super::MemoryStore;' in content:
        print(f"Skipping {filepath.name} - already has proper imports")
        return
    
    # Replace the header
    old_header = '''use super::MemoryStore;
use anyhow::Result;
use uuid::Uuid;
use async_trait::async_trait;

impl MemoryStore {
'''
    
    # Determine what imports are needed based on file name
    module_name = filepath.stem
    
    # Build new header with appropriate imports
    imports = [
        "use super::MemoryStore;",
        "use anyhow::Result;",
        "use uuid::Uuid;",
    ]
    
    # Add specific imports based on module
    if module_name in ['tenants', 'warehouses', 'catalogs', 'namespaces', 'assets', 'branches', 'tags', 'commits']:
        imports.append(f"use pangolin_core::model::*;")
    if module_name in ['users', 'service_users']:
        imports.append("use pangolin_core::user::*;")
    if module_name in ['roles', 'permissions']:
        imports.append("use pangolin_core::permission::*;")
    if module_name == 'audit':
        imports.append("use pangolin_core::audit::*;")
    if module_name == 'tokens':
        imports.append("use pangolin_core::token::*;")
    if module_name in ['business_metadata', 'access_requests']:
        imports.append("use pangolin_core::business_metadata::*;")
    if module_name == 'io':
        imports.append("use std::sync::Arc;")
    if module_name == 'signer':
        imports.append("use crate::signer::{Signer, Credentials};")
        imports.append("use async_trait::async_trait;")
    
    new_header = '\n'.join(imports) + '\n\nimpl MemoryStore {\n'
    
    # Replace header
    content = content.replace(old_header, new_header)
    
    # Rename all async fn methods to add _internal suffix
    # Pattern: async fn method_name(
    def rename_method(match):
        method_name = match.group(1)
        # Don't rename if already has _internal
        if '_internal' in method_name:
            return match.group(0)
        return f'    pub(crate) async fn {method_name}_internal('
    
    content = re.sub(r'    async fn (\w+)\(', rename_method, content)
    
    # Write back
    with open(filepath, 'w') as f:
        f.write(content)
    
    print(f"Fixed {filepath.name}")

def main():
    memory_dir = Path('src/memory')
    
    # Fix all .rs files except main.rs and mod.rs
    for rs_file in memory_dir.glob('*.rs'):
        if rs_file.name not in ['main.rs', 'mod.rs']:
            fix_module_file(rs_file)
    
    print("\nAll modules fixed!")

if __name__ == '__main__':
    main()
