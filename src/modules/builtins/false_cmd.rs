//! false builtin command - return failure

use crate::env::ShellEnv;

use super::BuiltinCommand;

/// false builtin command
pub struct False;

impl BuiltinCommand for False {
    fn name(&self) -> &'static str {
        "false"
    }
    
    fn execute(&self, _args: &[String], _env: &mut ShellEnv) -> i32 {
        1
    }
}