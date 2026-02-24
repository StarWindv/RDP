//! exec builtin command - replace shell with command

use std::process::Command;

use crate::env::ShellEnv;

use super::BuiltinCommand;

/// exec builtin command
pub struct Exec;

impl BuiltinCommand for Exec {
    fn name(&self) -> &'static str {
        "exec"
    }
    
    fn execute(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            // Just return success if no arguments
            return 0;
        }
        
        let command = &args[0];
        let command_args = &args[1..];
        
        // TODO: Actually replace shell process
        // For now, just execute as external command
        match Command::new(command)
            .args(command_args)
            .current_dir(&env.current_dir)
            .status() {
            Ok(status) => {
                if let Some(code) = status.code() {
                    code
                } else {
                    128
                }
            }
            Err(e) => {
                eprintln!("exec: {}: {}", command, e);
                127
            }
        }
    }
}