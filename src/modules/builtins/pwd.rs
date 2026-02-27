//! pwd builtin command - print working directory

use crate::modules::env::ShellEnv;

use crate::modules::builtins::registry::BuiltinCommand;

/// pwd builtin command
pub struct Pwd;

impl BuiltinCommand for Pwd {
    fn name(&self) -> &'static str {
        "pwd"
    }

    fn execute(&self, _args: &[String], env: &mut ShellEnv) -> i32 {
        println!("{}", env.current_dir);
        0
    }
}
