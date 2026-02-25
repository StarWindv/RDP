//! readonly builtin command - make variables read-only

use crate::modules::env::ShellEnv;

use crate::modules::builtins::registry::BuiltinCommand;

/// readonly builtin command
pub struct Readonly;

impl BuiltinCommand for Readonly {
    fn name(&self) -> &'static str {
        "readonly"
    }
    
    fn execute(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            // Print all read-only variables
            for (key, value) in &env.vars {
                if key.starts_with("__readonly_") {
                    let var_name = &key[11..]; // Remove "__readonly_" prefix
                    println!("readonly {}='{}'", var_name, value);
                }
            }
            return 0;
        }
        
        for arg in args {
            if arg.contains('=') {
                // VAR=value format
                let parts: Vec<&str> = arg.splitn(2, '=').collect();
                let var = parts[0];
                let val = if parts.len() > 1 { parts[1] } else { "" };
                env.set_var(var.to_string(), val.to_string());
                // Mark as read-only
                env.set_var(format!("__readonly_{}", var), "1".to_string());
            } else {
                // Just variable name, mark as read-only
                env.set_var(format!("__readonly_{}", arg), "1".to_string());
            }
        }
        
        0
    }
}