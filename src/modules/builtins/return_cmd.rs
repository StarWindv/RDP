//! Return builtin command for shell functions
//!
//! The `return` command exits a shell function with a given exit status.
//! If no status is given, the exit status of the last command executed
//! is used.

use crate::modules::builtins::registry::BuiltinCommand;
use crate::modules::env::ShellEnv;

/// Return command implementation
pub struct Return;

impl Return {
    /// Create a new Return command
    pub fn new() -> Self {
        Self
    }
}

impl BuiltinCommand for Return {
    fn execute(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        // Parse exit status from arguments
        let exit_status = if args.is_empty() {
            // No argument: use last command's exit status
            env.exit_status
        } else {
            // Use first argument as exit status
            args[0].parse().unwrap_or_else(|_| {
                eprintln!("return: numeric argument required");
                1
            })
        };

        // In a real implementation, this would need to signal
        // that we're returning from a function, not just setting exit status.
        // For now, we'll just set the exit status.
        env.exit_status = exit_status;

        // We need a way to signal that we should return from function.
        // This will be handled by the execution engine.
        exit_status
    }

    fn name(&self) -> &'static str {
        "return"
    }
}
