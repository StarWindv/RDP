//! cd builtin command - change directory

use crate::modules::env::ShellEnv;

use crate::modules::builtins::registry::BuiltinCommand;

/// cd builtin command
pub struct Cd;

impl BuiltinCommand for Cd {
    fn name(&self) -> &'static str {
        "cd"
    }

    fn execute(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        let target = if args.is_empty() {
            // Go to home directory
            env.get_var("HOME")
                .cloned()
                .unwrap_or_else(|| "/".to_string())
        } else if args[0] == "-" {
            // Go to previous directory (OLDPWD)
            env.get_var("OLDPWD")
                .cloned()
                .unwrap_or_else(|| env.current_dir.clone())
        } else {
            args[0].clone()
        };

        // Save current directory as OLDPWD
        env.set_var("OLDPWD".to_string(), env.current_dir.clone());

        // Change directory
        match std::env::set_current_dir(&target) {
            Ok(_) => {
                // Update current directory in environment
                if let Ok(cwd) = std::env::current_dir() {
                    env.current_dir = cwd.to_string_lossy().to_string();
                }
                0
            }
            Err(e) => {
                eprintln!("cd: {}: {}", target, e);
                1
            }
        }
    }
}
