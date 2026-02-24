//! times builtin command - print process times

use crate::env::ShellEnv;

use super::BuiltinCommand;

/// times builtin command
pub struct Times;

impl BuiltinCommand for Times {
    fn name(&self) -> &'static str {
        "times"
    }
    
    fn execute(&self, _args: &[String], _env: &mut ShellEnv) -> i32 {
        // TODO: Get actual process times
        // For now, print dummy values
        println!("0m0.000s 0m0.000s");
        println!("0m0.000s 0m0.000s");
        0
    }
}