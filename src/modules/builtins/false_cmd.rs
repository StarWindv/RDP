//! false builtin command - return failure

use crate::modules::env::ShellEnv;

use crate::modules::builtins::registry::BuiltinCommand;

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
