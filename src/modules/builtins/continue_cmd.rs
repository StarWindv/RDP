//! continue builtin command - continue loop iteration

use crate::modules::env::ShellEnv;
use crate::modules::builtins::registry::BuiltinCommand;

/// continue builtin command
pub struct Continue;

impl BuiltinCommand for Continue {
    fn name(&self) -> &'static str {
        "continue"
    }
    
    fn execute(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        let n = if args.is_empty() {
            1
        } else {
            args[0].parse().unwrap_or(1)
        };
        
        // Store continue count in environment
        env.set_var("CONTINUE_LEVEL".to_string(), n.to_string());
        0
    }
}