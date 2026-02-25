//! unset builtin command - unset variables

use crate::modules::env::ShellEnv;

use crate::modules::builtins::registry::BuiltinCommand;

/// unset builtin command
pub struct Unset;

impl BuiltinCommand for Unset {
    fn name(&self) -> &'static str {
        "unset"
    }
    
    fn execute(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            eprintln!("unset: variable name required");
            return 1;
        }
        
        for arg in args {
            env.unset_var(arg);
            // Also unset special attributes
            env.unset_var(&format!("__exported_{}", arg));
            env.unset_var(&format!("__readonly_{}", arg));
        }
        
        0
    }
}