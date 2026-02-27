//! export builtin command - set export attribute for variables

use crate::modules::env::ShellEnv;
use crate::modules::variables::get_variable_system;

use crate::modules::builtins::registry::BuiltinCommand;

/// export builtin command
pub struct Export;

impl BuiltinCommand for Export {
    fn name(&self) -> &'static str {
        "export"
    }

    fn execute(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        let mut vs = get_variable_system();

        if args.is_empty() {
            // Print all exported variables
            for (name, var) in vs.get_all_vars() {
                if var.is_exported() {
                    println!("export {}='{}'", name, var.value);
                }
            }
            return 0;
        }

        for arg in args {
            if arg.contains('=') {
                // VAR=value format
                let parts: Vec<&str> = arg.splitn(2, '=').collect();
                let var_name = parts[0].to_string();
                let value = if parts.len() > 1 { parts[1] } else { "" }.to_string();

                // Set variable in environment (for backward compatibility)
                env.set_var(var_name.clone(), value.clone());

                // Set in variable system with export attribute
                if let Err(e) = vs.set(var_name.clone(), value) {
                    eprintln!("export: {}", e);
                    return 1;
                }

                if let Err(e) = vs.export(&var_name) {
                    eprintln!("export: {}", e);
                    return 1;
                }
            } else {
                // Just variable name, mark as exported
                let var_name = arg.to_string();

                if let Err(e) = vs.export(&var_name) {
                    eprintln!("export: {}", e);
                    return 1;
                }
            }
        }

        0
    }
}
