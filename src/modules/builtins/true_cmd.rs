//! true builtin command - return success

use crate::modules::env::ShellEnv;

use crate::modules::builtins::registry::BuiltinCommand;

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