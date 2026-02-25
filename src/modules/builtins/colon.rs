//! : (colon) builtin command - null command

use crate::modules::env::ShellEnv;

use crate::modules::builtins::registry::BuiltinCommand;

/// : (colon) builtin command
pub struct Colon;

impl BuiltinCommand for Colon {
    fn name(&self) -> &'static str {
        ":"
    }
    
    fn execute(&self, _args: &[String], _env: &mut ShellEnv) -> i32 {
        0
    }
}