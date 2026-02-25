//! echo builtin command - display arguments

use crate::modules::env::ShellEnv;

use crate::modules::builtins::registry::BuiltinCommand;

/// echo builtin command
pub struct Echo;

impl BuiltinCommand for Echo {
    fn name(&self) -> &'static str {
        "echo"
    }
    
    fn execute(&self, args: &[String], _env: &ShellEnv) -> i32 {
        let mut first = true;
        for arg in args {
            if !first {
                print!(" ");
            }
            // Arguments are already expanded by the executor
            print!("{}", arg);
            first = false;
        }
        println!();
        0
    }
}