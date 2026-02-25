//! Function system for POSIX Shell
//! Handles function definition, execution, and management

use std::collections::HashMap;

use crate::modules::ast::AstNode;
use crate::modules::env::ShellEnv;

/// Shell function definition
#[derive(Debug, Clone)]
pub struct ShellFunction {
    /// Function name
    pub name: String,
    /// Function body (AST nodes)
    pub body: Vec<AstNode>,
    /// Local variables (when function is executing)
    pub locals: HashMap<String, String>,
}

impl ShellFunction {
    /// Create a new shell function
    pub fn new(name: &str, body: Vec<AstNode>) -> Self {
        Self {
            name: name.to_string(),
            body,
            locals: HashMap::new(),
        }
    }
    
    /// Execute the function with given arguments
    pub fn execute(&mut self, args: &[String], env: &mut ShellEnv) -> Result<i32, String> {
        // TODO: Implement function execution
        // For now, just return success
        Ok(0)
    }
    
    /// Set a local variable
    pub fn set_local(&mut self, name: &str, value: &str) {
        self.locals.insert(name.to_string(), value.to_string());
    }
    
    /// Get a local variable
    pub fn get_local(&self, name: &str) -> Option<&String> {
        self.locals.get(name)
    }
    
    /// Clear all local variables
    pub fn clear_locals(&mut self) {
        self.locals.clear();
    }
}

/// Function manager for shell
pub struct FunctionManager {
    functions: HashMap<String, ShellFunction>,
    env: ShellEnv,
}

impl FunctionManager {
    /// Create a new function manager
    pub fn new(env: ShellEnv) -> Self {
        Self {
            functions: HashMap::new(),
            env,
        }
    }
    
    /// Define a new function
    pub fn define(&mut self, name: &str, body: Vec<AstNode>) -> Result<(), String> {
        if !Self::is_valid_function_name(name) {
            return Err(format!("Invalid function name: {}", name));
        }
        
        let function = ShellFunction::new(name, body);
        self.functions.insert(name.to_string(), function);
        
        Ok(())
    }
    
    /// Execute a function by name
    pub fn execute(&mut self, name: &str, args: &[String]) -> Result<i32, String> {
        match self.functions.get_mut(name) {
            Some(function) => function.execute(args, &mut self.env),
            None => Err(format!("Function not found: {}", name)),
        }
    }
    
    /// Check if a function exists
    pub fn exists(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
    
    /// List all defined functions
    pub fn list(&self) -> Vec<&str> {
        self.functions.keys().map(|k| k.as_str()).collect()
    }
    
    /// Undefine (remove) a function
    pub fn undefine(&mut self, name: &str) -> bool {
        self.functions.remove(name).is_some()
    }
    
    /// Get a function definition
    pub fn get(&self, name: &str) -> Option<&ShellFunction> {
        self.functions.get(name)
    }
    
    /// Get a mutable reference to a function
    pub fn get_mut(&mut self, name: &str) -> Option<&mut ShellFunction> {
        self.functions.get_mut(name)
    }
    
    /// Check if a name is valid for a function
    fn is_valid_function_name(name: &str) -> bool {
        // Function names follow the same rules as variable names
        if name.is_empty() {
            return false;
        }
        
        // First character must be letter or underscore
        let first_char = name.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() && first_char != '_' {
            return false;
        }
        
        // All characters must be alphanumeric or underscore
        name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    }
    
    /// Handle return statement from function
    pub fn handle_return(&self, exit_status: i32) -> Result<(), String> {
        // TODO: Implement return handling
        Ok(())
    }
    
    /// Handle local variable declaration
    pub fn handle_local(&mut self, name: &str, value: Option<&str>) -> Result<(), String> {
        // TODO: Implement local variable handling
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function_manager_creation() {
        let env = ShellEnv::new();
        let manager = FunctionManager::new(env);
        
        // Just test that it can be created
        assert!(true);
    }
    
    #[test]
    fn test_valid_function_names() {
        assert!(FunctionManager::is_valid_function_name("foo"));
        assert!(FunctionManager::is_valid_function_name("foo_bar"));
        assert!(FunctionManager::is_valid_function_name("foo123"));
        assert!(FunctionManager::is_valid_function_name("_foo"));
        
        assert!(!FunctionManager::is_valid_function_name(""));
        assert!(!FunctionManager::is_valid_function_name("123foo"));
        assert!(!FunctionManager::is_valid_function_name("foo-bar"));
        assert!(!FunctionManager::is_valid_function_name("foo bar"));
    }
    
    #[test]
    fn test_function_definition() {
        let env = ShellEnv::new();
        let mut manager = FunctionManager::new(env);
        
        // Define a function
        let body = vec![];
        let result = manager.define("myfunc", body);
        assert!(result.is_ok());
        
        // Check that function exists
        assert!(manager.exists("myfunc"));
        
        // List functions
        let functions = manager.list();
        assert_eq!(functions, vec!["myfunc"]);
        
        // Undefine function
        assert!(manager.undefine("myfunc"));
        assert!(!manager.exists("myfunc"));
    }
}