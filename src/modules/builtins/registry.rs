//! Builtins registry and execution

use std::collections::HashMap;

use crate::env::ShellEnv;

use super::*;

/// Trait for built-in commands
pub trait BuiltinCommand {
    fn execute(&self, args: &[String], env: &mut ShellEnv) -> i32;
    fn name(&self) -> &'static str;
}

/// Registry for built-in commands
pub struct BuiltinRegistry {
    commands: HashMap<String, Box<dyn BuiltinCommand>>,
}

impl BuiltinRegistry {
    /// Create a new builtin registry with all commands registered
    pub fn new() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
        };
        
        // Register all builtin commands
        registry.register(Box::new(Dot));
        registry.register(Box::new(Colon));
        registry.register(Box::new(Break));
        registry.register(Box::new(Continue));
        registry.register(Box::new(Eval));
        registry.register(Box::new(Exec));
        registry.register(Box::new(Exit));
        registry.register(Box::new(Export));
        registry.register(Box::new(Readonly));
        registry.register(Box::new(Set));
        registry.register(Box::new(Shift));
        registry.register(Box::new(Times));
        registry.register(Box::new(Trap));
        registry.register(Box::new(Unset));
        
        registry.register(Box::new(Alias));
        registry.register(Box::new(Bg));
        registry.register(Box::new(Cd));
        registry.register(Box::new(Command));
        registry.register(Box::new(Echo));
        registry.register(Box::new(False));
        registry.register(Box::new(Fg));
        registry.register(Box::new(Getopts));
        registry.register(Box::new(Hash));
        registry.register(Box::new(Jobs));
        registry.register(Box::new(Kill));
        registry.register(Box::new(Pwd));
        registry.register(Box::new(Read));
        registry.register(Box::new(True));
        registry.register(Box::new(Type));
        registry.register(Box::new(Umask));
        registry.register(Box::new(Ulimit));
        registry.register(Box::new(Wait));
        registry.register(Box::new(Printf));
        registry.register(Box::new(Local));
        registry.register(Box::new(Help));
        
        registry
    }
    
    /// Register a builtin command
    fn register(&mut self, command: Box<dyn BuiltinCommand>) {
        self.commands.insert(command.name().to_string(), command);
    }
    
    /// Execute a builtin command
    pub fn execute(&self, name: &str, args: &[String], env: &mut ShellEnv) -> i32 {
        if let Some(command) = self.commands.get(name) {
            command.execute(args, env)
        } else {
            eprintln!("{}: command not found", name);
            127
        }
    }
    
    /// Check if a command is a builtin
    pub fn is_builtin(&self, name: &str) -> bool {
        self.commands.contains_key(name)
    }
    
    /// Get all builtin command names
    pub fn command_names(&self) -> Vec<&str> {
        self.commands.keys().map(|s| s.as_str()).collect()
    }
}

/// Main builtins struct (for backward compatibility)
pub struct Builtins {
    registry: BuiltinRegistry,
}

impl Builtins {
    /// Create a new Builtins instance
    pub fn new() -> Self {
        Self {
            registry: BuiltinRegistry::new(),
        }
    }
    
    /// Execute a builtin command
    pub fn execute(&self, name: &str, args: &[String], env: &mut ShellEnv) -> i32 {
        self.registry.execute(name, args, env)
    }
    
    /// Check if a command is a builtin
    pub fn is_builtin(&self, name: &str) -> bool {
        self.registry.is_builtin(name)
    }
}