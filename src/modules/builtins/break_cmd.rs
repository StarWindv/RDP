//! break builtin command - exit from a loop

use crate::modules::env::ShellEnv;
use crate::modules::builtins::registry::BuiltinCommand;

/// break builtin command
pub struct Break;

impl BuiltinCommand for Break {
    fn name(&self) -> &'static str {
        "break"
    }
    
    fn execute(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        let n = if args.is_empty() {
            1
        } else {
            args[0].parse().unwrap_or(1)
        };
        
        // Store break count in environment
        env.set_var("BREAK_LEVEL".to_string(), n.to_string());
        0
    }
}