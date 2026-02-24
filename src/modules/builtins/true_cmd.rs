//! true builtin command - return success

use crate::env::ShellEnv;

use super::BuiltinCommand;

/// true builtin command
pub struct True;

impl BuiltinCommand for True {
    fn name(&self) -> &'static str {
        "true"
    }
    
    fn execute(&self, _args: &[String], _env: &mut ShellEnv) -> i32 {
        0
    }
}