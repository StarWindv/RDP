//! readonly builtin command - make variables read-only

use crate::modules::env::ShellEnv;
use crate::modules::variables::get_variable_system;

use crate::modules::builtins::registry::BuiltinCommand;

/// readonly builtin command
pub struct Readonly;

impl BuiltinCommand for Readonly {
    fn name(&self) -> &'static str {
        "readonly"
    }
    
    fn execute(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        let mut vs = get_variable_system();
        
        if args.is_empty() {
            // Print all read-only variables
            for (name, var) in vs.get_all_vars() {
                if var.is_readonly() {
                    println!("readonly {}='{}'", name, var.value);
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
                
                // Set in variable system as read-only
                if let Err(e) = vs.readonly_with_value(var_name, value) {
                    eprintln!("readonly: {}", e);
                    return 1;
                }
            } else {
                // Just variable name, mark as read-only
                let var_name = arg.to_string();
                
                if let Err(e) = vs.readonly(&var_name) {
                    eprintln!("readonly: {}", e);
                    return 1;
                }
            }
        }
        
        0
    }
}