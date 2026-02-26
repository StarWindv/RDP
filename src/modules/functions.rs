//! Function system for POSIX Shell
//! Handles function definition, execution, and management

use std::collections::HashMap;

use crate::modules::ast::AstNode;
use crate::modules::env::ShellEnv;
use crate::modules::variables::{get_variable_system, VariableSystem};
use crate::modules::builtins::Builtins;

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
    pub fn execute(&mut self, args: &[String], env: &mut ShellEnv, builtins: &Builtins) -> Result<i32, String> {
        // Enter function scope
        let mut vs = get_variable_system();
        vs.enter_scope();
        
        // Set positional parameters
        // $0 is function name, $1-$n are arguments
        vs.set("0".to_string(), self.name.clone())
            .map_err(|e| format!("Failed to set $0: {}", e))?;
        
        for (i, arg) in args.iter().enumerate() {
            let param_name = (i + 1).to_string();
            vs.set(param_name, arg.clone())
                .map_err(|e| format!("Failed to set ${}: {}", i + 1, e))?;
        }
        
        // Set special parameter $# (number of arguments)
        vs.set("#".to_string(), args.len().to_string())
            .map_err(|e| format!("Failed to set $#: {}", e))?;
        
        // Set special parameter $* and $@ (all arguments)
        let all_args = args.join(" ");
        vs.set("*".to_string(), all_args.clone())
            .map_err(|e| format!("Failed to set $*: {}", e))?;
        vs.set("@".to_string(), all_args)
            .map_err(|e| format!("Failed to set $@: {}", e))?;
        
        // Execute function body
        let mut exit_status = 0;
        
        for node in &self.body {
            // TODO: Actually execute AST nodes
            // For now, just simulate execution
            println!("Executing function {}: {}", self.name, node);
            
            // In a real implementation, we would use SSA executor
            // For now, just return success
            exit_status = 0;
        }
        
        // Exit function scope
        vs.exit_scope()
            .map_err(|e| format!("Failed to exit function scope: {}", e))?;
        
        Ok(exit_status)
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
    builtins: Builtins,
}

impl FunctionManager {
    /// Create a new function manager
    pub fn new(env: ShellEnv, builtins: Builtins) -> Self {
        Self {
            functions: HashMap::new(),
            env,
            builtins,
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
            Some(function) => function.execute(args, &mut self.env, &self.builtins),
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
        // This would need to break out of function execution
        Ok(())
    }
    
    /// Handle local variable declaration
    pub fn handle_local(&mut self, name: &str, value: Option<&str>) -> Result<(), String> {
        let mut vs = get_variable_system();
        let value = value.unwrap_or("");
        vs.local(name.to_string(), Some(value.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function_manager_creation() {
        let env = ShellEnv::new();
        let builtins = Builtins::new();
        let manager = FunctionManager::new(env, builtins);
        
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
        let builtins = Builtins::new();
        let mut manager = FunctionManager::new(env, builtins);
        
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