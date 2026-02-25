#!/usr/bin/env python3
import os
import re

def fix_crate_imports(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Fix common import patterns
    patterns = [
        (r'use crate::tokens::', r'use crate::modules::tokens::'),
        (r'use crate::ast::', r'use crate::modules::ast::'),
        (r'use crate::env::', r'use crate::modules::env::'),
        (r'use crate::ssa_ir::', r'use crate::modules::ssa_ir::'),
        (r'use crate::builtins::', r'use crate::modules::builtins::'),
        (r'crate::tokens::', r'crate::modules::tokens::'),
        (r'crate::ast::', r'crate::modules::ast::'),
        (r'crate::env::', r'crate::modules::env::'),
        (r'crate::ssa_ir::', r'crate::modules::ssa_ir::'),
        (r'crate::builtins::', r'crate::modules::builtins::'),
    ]
    
    for pattern, replacement in patterns:
        content = re.sub(pattern, replacement, content)
    
    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(content)
    
    print(f"Fixed imports in {filepath}")

def main():
    # Fix all .rs files in src directory
    for root, dirs, files in os.walk("src"):
        for filename in files:
            if filename.endswith('.rs'):
                filepath = os.path.join(root, filename)
                fix_crate_imports(filepath)
    
    print("All crate imports have been fixed.")

if __name__ == "__main__":
    main()