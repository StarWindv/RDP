//! trap builtin command - handle signals

use crate::env::ShellEnv;

use super::BuiltinCommand;

/// trap builtin command
pub struct Trap;

impl BuiltinCommand for Trap {
    fn name(&self) -> &'static str {
        "trap"
    }
    
    fn execute(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            // Print current traps
            println!("trap: no traps set");
            return 0;
        }
        
        // Parse trap command
        // Format: trap [action] signal...
        let action = if args[0] == "-" {
            // Reset to default
            "".to_string()
        } else if args[0] == "--" {
            // Reset all signals
            for sig in &["INT", "TERM", "EXIT", "ERR"] {
                env.set_var(format!("__trap_{}", sig), "".to_string());
            }
            return 0;
        } else {
            args[0].clone()
        };
        
        let signals = if args[0] == "-" || args[0] == "--" {
            &args[1..]
        } else {
            &args[1..]
        };
        
        if signals.is_empty() {
            // No signals specified, use default
            env.set_var("__trap_EXIT".to_string(), action);
        } else {
            for sig in signals {
                env.set_var(format!("__trap_{}", sig), action.clone());
            }
        }
        
        0
    }
}