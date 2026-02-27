//! local builtin command - create local variables in functions

use crate::modules::env::ShellEnv;
use crate::modules::variables::get_variable_system;

use crate::modules::builtins::registry::BuiltinCommand;

/// local builtin command
pub struct Local;

impl BuiltinCommand for Local {
    fn name(&self) -> &'static str {
        "local"
    }

    fn execute(&self, args: &[String], _env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            // In POSIX shell, local without arguments does nothing
            return 0;
        }

        let mut vs = get_variable_system();

        for arg in args {
            if arg.contains('=') {
                // VAR=value format
                let parts: Vec<&str> = arg.splitn(2, '=').collect();
                let var_name = parts[0].to_string();
                let value = if parts.len() > 1 { parts[1] } else { "" }.to_string();

                if let Err(e) = vs.local(var_name, Some(value)) {
                    eprintln!("local: {}", e);
                    return 1;
                }
            } else {
                // Just variable name, create with empty value
                let var_name = arg.to_string();

                if let Err(e) = vs.local(var_name, None) {
                    eprintln!("local: {}", e);
                    return 1;
                }
            }
        }

        0
    }
}
