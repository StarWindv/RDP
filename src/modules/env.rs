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

    /// Expand variables in a string
    pub fn expand_variables(&self, input: &str) -> String {
        println!("DEBUG EXPAND: expanding '{}'", input);
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

                        println!("DEBUG EXPAND: found ${{{}}}", var_name);

                        // Check for special variables
                        match var_name.as_str() {
                            "?" => {
                                println!(
                                    "DEBUG EXPAND: special variable $?, value = {}",
                                    self.exit_status
                                );
                                result.push_str(&self.exit_status.to_string());
                            }
                            "$" => {
                                // Process ID
                                let pid = std::process::id();
                                println!("DEBUG EXPAND: special variable $$, value = {}", pid);
                                result.push_str(&pid.to_string());
                            }
                            "0" => {
                                // Shell name
                                println!("DEBUG EXPAND: special variable $0, value = rs-dash-pro");
                                result.push_str("rs-dash-pro");
                            }
                            _ => {
                                if let Some(value) = self.get_var(&var_name) {
                                    println!("DEBUG EXPAND: value = '{}'", value);
                                    result.push_str(value);
                                } else {
                                    println!("DEBUG EXPAND: variable not found");
                                }
                            }
                        }
                    } else if next.is_ascii_alphanumeric()
                        || next == '_'
                        || next == '?'
                        || next == '$'
                        || next == '0'
                    {
                        // $var syntax or special variables
                        let mut var_name = String::new();
                        while let Some(&c) = chars.peek() {
                            if c.is_ascii_alphanumeric()
                                || c == '_'
                                || c == '?'
                                || c == '$'
                                || c == '0'
                            {
                                var_name.push(c);
                                chars.next();
                            } else {
                                break;
                            }
                        }

                        println!("DEBUG EXPAND: found ${}", var_name);

                        // Check for special variables
                        match var_name.as_str() {
                            "?" => {
                                println!(
                                    "DEBUG EXPAND: special variable $?, value = {}",
                                    self.exit_status
                                );
                                result.push_str(&self.exit_status.to_string());
                            }
                            "$" => {
                                // Process ID
                                let pid = std::process::id();
                                println!("DEBUG EXPAND: special variable $$, value = {}", pid);
                                result.push_str(&pid.to_string());
                            }
                            "0" => {
                                // Shell name
                                println!("DEBUG EXPAND: special variable $0, value = rs-dash-pro");
                                result.push_str("rs-dash-pro");
                            }
                            _ => {
                                if let Some(value) = self.get_var(&var_name) {
                                    println!("DEBUG EXPAND: value = '{}'", value);
                                    result.push_str(value);
                                } else {
                                    println!("DEBUG EXPAND: variable not found");
                                }
                            }
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

        println!("DEBUG EXPAND: result = '{}'", result);
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
            self.get_var("Path")
                .or_else(|| self.get_var("PATH"))
                .cloned()
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

    /// Check if a variable is exported
    pub fn is_exported(&self, name: &str) -> bool {
        self.vars.contains_key(&format!("__exported_{}", name))
    }

    /// Check if a variable is read-only
    pub fn is_readonly(&self, name: &str) -> bool {
        self.vars.contains_key(&format!("__readonly_{}", name))
    }

    /// Check if a variable is an alias
    pub fn is_alias(&self, name: &str) -> bool {
        self.vars.contains_key(&format!("__alias_{}", name))
    }

    /// Get alias value
    pub fn get_alias(&self, name: &str) -> Option<&String> {
        self.vars.get(&format!("__alias_{}", name))
    }

    /// Check if a variable is a function
    pub fn is_function(&self, name: &str) -> bool {
        self.vars.contains_key(&format!("__function_{}", name))
    }

    /// Get function body
    pub fn get_function(&self, name: &str) -> Option<&String> {
        self.vars.get(&format!("__function_{}", name))
    }

    /// Get positional arguments
    pub fn get_positional_args(&self) -> Vec<String> {
        if let Some(args_str) = self.get_var("__positional_args") {
            args_str.split_whitespace().map(|s| s.to_string()).collect()
        } else {
            Vec::new()
        }
    }
}
