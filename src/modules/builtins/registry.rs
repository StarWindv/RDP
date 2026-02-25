//! Builtins registry and execution

use std::collections::HashMap;

use crate::modules::env::ShellEnv;

// Import only existing modules
use super::dot::Dot;
use super::colon::Colon;
use super::break_cmd::Break;
use super::continue_cmd::Continue;
use super::eval::Eval;
use super::exec::Exec;
use super::exit::Exit;
use super::export::Export;
use super::readonly::Readonly;
use super::set::Set;
use super::shift::Shift;
use super::times::Times;
use super::trap::Trap;
use super::unset::Unset;
use super::alias::Alias;
use super::cd::Cd;
use super::echo::Echo;
use super::false_cmd::False;
use super::pwd::Pwd;
use super::true_cmd::True;
use super::help::Help;

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
        
        // Register all builtin commands (only existing ones)
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
        registry.register(Box::new(Cd));
        registry.register(Box::new(Echo));
        registry.register(Box::new(False));
        registry.register(Box::new(Pwd));
        registry.register(Box::new(True));
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