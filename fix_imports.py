#!/usr/bin/env python3
import os
import re

def fix_imports_in_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Replace imports
    # Pattern 1: use crate::env::ShellEnv;
    # Replace with: use crate::modules::env::ShellEnv;
    content = re.sub(r'use crate::env::ShellEnv;', 
                    r'use crate::modules::env::ShellEnv;', 
                    content)
    
    # Pattern 2: use super::BuiltinCommand;
    # Replace with: use crate::modules::builtins::registry::BuiltinCommand;
    content = re.sub(r'use super::BuiltinCommand;', 
                    r'use crate::modules::builtins::registry::BuiltinCommand;', 
                    content)
    
    # Also handle the case where both are on the same line
    content = re.sub(r'use crate::env::ShellEnv;\s*\n\s*use super::BuiltinCommand;', 
                    r'use crate::modules::env::ShellEnv;\nuse crate::modules::builtins::registry::BuiltinCommand;', 
                    content)
    
    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(content)
    
    print(f"Fixed imports in {filepath}")

def main():
    builtins_dir = os.path.join("src", "modules", "builtins")
    
    if not os.path.exists(builtins_dir):
        print(f"Directory not found: {builtins_dir}")
        return
    
    # List all .rs files in builtins directory
    for filename in os.listdir(builtins_dir):
        if filename.endswith('.rs') and filename != 'mod.rs' and filename != 'registry.rs':
            filepath = os.path.join(builtins_dir, filename)
            fix_imports_in_file(filepath)
    
    print("All builtin command imports have been fixed.")

if __name__ == "__main__":
    main()