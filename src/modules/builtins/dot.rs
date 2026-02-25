//! . (dot) builtin command - source a file

use std::fs;

use crate::modules::env::ShellEnv;

use crate::modules::builtins::registry::BuiltinCommand;

/// . (dot) builtin command
pub struct Dot;

impl BuiltinCommand for Dot {
    fn name(&self) -> &'static str {
        "."
    }
    
    fn execute(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            eprintln!(".: filename argument required");
            return 1;
        }
        
        let filename = &args[0];
        match fs::read_to_string(filename) {
            Ok(content) => {
                // TODO: Actually parse and execute the file content
                // For now, just split into lines and handle simple cases
                let last_status = 0;
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    
                    // Simple handling of export commands
                    if line.starts_with("export ") {
                        let export_line = &line[7..]; // Remove "export "
                        let parts: Vec<&str> = export_line.splitn(2, '=').collect();
                        if parts.len() == 2 {
                            let var_name = parts[0].trim();
                            let var_value = parts[1].trim().trim_matches('"');
                            env.set_var(var_name.to_string(), var_value.to_string());
                            // Mark as exported
                            env.set_var(format!("__exported_{}", var_name), "1".to_string());
                        }
                    } else if line.starts_with("echo ") {
                        let echo_line = &line[5..]; // Remove "echo "
                        let expanded = env.expand_variables(echo_line.trim_matches('"'));
                        println!("{}", expanded);
                    }
                }
                
                last_status
            }
            Err(e) => {
                eprintln!(".: {}: {}", filename, e);
                1
            }
        }
    }
}