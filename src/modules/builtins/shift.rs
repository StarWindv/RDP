//! shift builtin command - shift positional parameters

use crate::env::ShellEnv;

use super::BuiltinCommand;

/// shift builtin command
pub struct Shift;

impl BuiltinCommand for Shift {
    fn name(&self) -> &'static str {
        "shift"
    }
    
    fn execute(&self, args: &[String], _env: &mut ShellEnv) -> i32 {
        let n = if args.is_empty() {
            1
        } else {
            args[0].parse().unwrap_or(1)
        };
        
        // TODO: Actually shift positional parameters
        // For now, just log the operation
        println!("shift: shifting {} positional parameters", n);
        0
    }
}