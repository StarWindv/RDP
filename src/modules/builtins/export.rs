//! export builtin command - set export attribute for variables

use crate::env::ShellEnv;

use super::BuiltinCommand;

/// export builtin command
pub struct Export;

impl BuiltinCommand for Export {
    fn name(&self) -> &'static str {
        "export"
    }
    
    fn execute(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            // Print all exported variables
            for (key, value) in &env.vars {
                if key.starts_with("__exported_") {
                    let var_name = &key[11..]; // Remove "__exported_" prefix
                    println!("export {}='{}'", var_name, value);
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
                // Mark as exported
                env.set_var(format!("__exported_{}", var), "1".to_string());
            } else {
                // Just variable name, mark as exported
                env.set_var(format!("__exported_{}", arg), "1".to_string());
            }
        }
        
        0
    }
}