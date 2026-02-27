//! unset builtin command - unset variables

use crate::modules::env::ShellEnv;
use crate::modules::variables::get_variable_system;

use crate::modules::builtins::registry::BuiltinCommand;

/// unset builtin command
pub struct Unset;

impl BuiltinCommand for Unset {
    fn name(&self) -> &'static str {
        "unset"
    }

    fn execute(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            eprintln!("unset: variable name required");
            return 1;
        }

        let mut vs = get_variable_system();

        for arg in args {
            // Also unset from environment (for backward compatibility)
            env.unset_var(arg);

            if let Err(e) = vs.unset(arg) {
                eprintln!("unset: {}", e);
                return 1;
            }
        }

        0
    }
}
