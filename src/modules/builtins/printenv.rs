//! printenv builtin command - print environment variables

use crate::modules::env::ShellEnv;
use crate::modules::variables::get_variable_system;
use crate::modules::builtins::registry::BuiltinCommand;

/// printenv builtin command
pub struct PrintEnv;

impl BuiltinCommand for PrintEnv {
    fn name(&self) -> &'static str {
        "printenv"
    }

    fn execute(&self, args: &[String], _env: &mut ShellEnv) -> i32 {
        let vs = get_variable_system();

        if args.is_empty() {
            // Print all environment variables
            let vars = vs.get_all_vars();
            for (name, var) in vars {
                println!("{}={}", name, var.value);
            }
            0
        } else {
            // Print specific variables
            let mut not_found = false;
            for var_name in args {
                if let Some(var) = vs.get(var_name) {
                    println!("{}", var.value);
                } else {
                    eprintln!("printenv: {}: not found", var_name);
                    not_found = true;
                }
            }
            if not_found { 1 } else { 0 }
        }
    }
}
