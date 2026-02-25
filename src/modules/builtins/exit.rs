//! exit builtin command - exit shell

use crate::modules::env::ShellEnv;

use crate::modules::builtins::registry::BuiltinCommand;

/// exit builtin command
pub struct Exit;

impl BuiltinCommand for Exit {
    fn name(&self) -> &'static str {
        "exit"
    }
    
    fn execute(&self, args: &[String], _env: &mut ShellEnv) -> i32 {
        let exit_code = if args.is_empty() {
            0
        } else {
            args[0].parse().unwrap_or(1)
        };
        
        std::process::exit(exit_code);
    }
}