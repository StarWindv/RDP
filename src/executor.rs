//! Legacy Executor (kept for compatibility)
//! The new SSA executor is in ssa_executor.rs

use crate::builtins::Builtins;
use crate::ssa_ir::Function;
use crate::env::ShellEnv;

/// Legacy Executor (deprecated)
pub struct Executor {
    builtins: Builtins,
    env: ShellEnv,
}

impl Executor {
    /// Create a new executor
    pub fn new() -> Self {
        Self {
            builtins: Builtins,
            env: ShellEnv::new(),
        }
    }
    
    /// Execute a legacy IR program (deprecated)
    pub fn execute(&mut self, _program: &Function) -> i32 {
        // For compatibility, just return success
        0
    }
    
    /// Get current environment
    pub fn get_env(&self) -> &ShellEnv {
        &self.env
    }
    
    /// Get mutable environment
    pub fn get_env_mut(&mut self) -> &mut ShellEnv {
        &mut self.env
    }
}