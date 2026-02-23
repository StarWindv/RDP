//! Utility functions for shell execution

use std::collections::HashMap;
use std::env;
use std::path::Path;

/// Shell environment
#[derive(Debug, Clone)]
pub struct ShellEnv {
    pub vars: HashMap<String, String>,
    pub current_dir: String,
    pub exit_status: i32,
}

impl ShellEnv {
    pub fn new() -> Self {
        let mut vars = HashMap::new();
        
        // Copy environment variables
        for (key, value) in env::vars() {
            vars.insert(key, value);
        }
        
        let current_dir = env::current_dir()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        
        Self {
            vars,
            current_dir,
            exit_status: 0,
        }
    }
    
    pub fn get_var(&self, name: &str) -> Option<&String> {
        self.vars.get(name)
    }
    
    pub fn set_var(&mut self, name: String, value: String) {
        self.vars.insert(name, value);
    }
    
    pub fn unset_var(&mut self, name: &str) {
        self.vars.remove(name);
    }
    
    pub fn expand_variables(&self, input: &str) -> String {
        let mut result = String::new();
        let mut chars = input.chars().peekable();
        
        while let Some(c) = chars.next() {
            if c == '$' {
                if let Some(&next) = chars.peek() {
                    if next == '{' {
                        // ${var} syntax
                        chars.next(); // Skip {
                        let mut var_name = String::new();
                        while let Some(&c) = chars.peek() {
                            if c == '}' {
                                chars.next(); // Skip }
                                break;
                            }
                            var_name.push(c);
                            chars.next();
                        }
                        
                        if let Some(value) = self.get_var(&var_name) {
                            result.push_str(value);
                        }
                    } else if next.is_ascii_alphanumeric() || next == '_' {
                        // $var syntax
                        let mut var_name = String::new();
                        while let Some(&c) = chars.peek() {
                            if c.is_ascii_alphanumeric() || c == '_' {
                                var_name.push(c);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        
                        if let Some(value) = self.get_var(&var_name) {
                            result.push_str(value);
                        }
                    } else {
                        // Just a literal $
                        result.push(c);
                    }
                } else {
                    // Just a literal $ at end of string
                    result.push(c);
                }
            } else {
                result.push(c);
            }
        }
        
        result
    }
    
    pub fn find_in_path(&self, cmd: &str) -> Option<String> {
        // If command contains path separator, check directly
        if cmd.contains('/') || (cfg!(windows) && cmd.contains('\\')) {
            if Path::new(cmd).exists() {
                return Some(cmd.to_string());
            }
            return None;
        }
        
        // Get PATH variable
        let path_var = if cfg!(windows) {
            self.get_var("Path").or_else(|| self.get_var("PATH")).cloned()
        } else {
            self.get_var("PATH").cloned()
        };
        
        let path_var = path_var.unwrap_or_default();
        let path_separator = if cfg!(windows) { ';' } else { ':' };
        
        for dir in path_var.split(path_separator) {
            if dir.is_empty() {
                continue;
            }
            
            let full_path = Path::new(dir).join(cmd);
            if full_path.exists() {
                return Some(full_path.to_string_lossy().to_string());
            }
            
            // On Windows, check with extensions
            #[cfg(windows)]
            {
                let extensions = [".exe", ".bat", ".cmd"];
                for ext in &extensions {
                    let full_path = Path::new(dir).join(format!("{}{}", cmd, ext));
                    if full_path.exists() {
                        return Some(full_path.to_string_lossy().to_string());
                    }
                }
            }
        }
        
        None
    }
}