//! set builtin command - set shell options

use crate::env::ShellEnv;

use super::BuiltinCommand;

/// set builtin command
pub struct Set;

impl BuiltinCommand for Set {
    fn name(&self) -> &'static str {
        "set"
    }
    
    fn execute(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            // Print all variables
            for (key, value) in &env.vars {
                println!("{}='{}'", key, value);
            }
            return 0;
        }
        
        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            
            if arg == "--" {
                // End of options
                i += 1;
                break;
            }
            
            if arg.starts_with('-') && !arg.starts_with("--") {
                // Handle options
                for ch in arg[1..].chars() {
                    match ch {
                        'e' => env.set_var("__option_errexit".to_string(), "1".to_string()),
                        'u' => env.set_var("__option_nounset".to_string(), "1".to_string()),
                        'x' => env.set_var("__option_xtrace".to_string(), "1".to_string()),
                        _ => {
                            eprintln!("set: invalid option -{}", ch);
                            return 1;
                        }
                    }
                }
            } else if arg.starts_with('+') {
                // Clear options
                for ch in arg[1..].chars() {
                    match ch {
                        'e' => env.unset_var("__option_errexit"),
                        'u' => env.unset_var("__option_nounset"),
                        'x' => env.unset_var("__option_xtrace"),
                        _ => {
                            eprintln!("set: invalid option +{}", ch);
                            return 1;
                        }
                    }
                }
            } else {
                // Set positional parameters
                let mut pos_args = Vec::new();
                for j in i..args.len() {
                    pos_args.push(args[j].clone());
                }
                env.set_var("__positional_args".to_string(), pos_args.join(" "));
                break;
            }
            
            i += 1;
        }
        
        0
    }
}