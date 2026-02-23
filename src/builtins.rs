//! Built-in shell commands

use std::io;

use crate::env::ShellEnv;

/// Built-in command handler
pub struct Builtins;

impl Builtins {
    /// Execute a built-in command
    pub fn execute(&self, name: &str, args: &[String], env: &mut ShellEnv) -> i32 {
        match name {
            "cd" => self.cd(args, env),
            "pwd" => self.pwd(env),
            "echo" => self.echo(args, env),
            "exit" => self.exit(args),
            "true" => 0,
            "false" => 1,
            "help" => self.help(),
            _ => {
                eprintln!("{}: command not found", name);
                127
            }
        }
    }
    
    /// Change directory
    fn cd(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        let target = if args.is_empty() {
            // Go to home directory
            env.get_var("HOME").cloned().unwrap_or_else(|| "/".to_string())
        } else if args[0] == "-" {
            // Go to previous directory (OLDPWD)
            env.get_var("OLDPWD").cloned().unwrap_or_else(|| env.current_dir.clone())
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
    
    /// Print working directory
    fn pwd(&self, env: &ShellEnv) -> i32 {
        println!("{}", env.current_dir);
        0
    }
    
    /// Echo arguments
    fn echo(&self, args: &[String], _env: &ShellEnv) -> i32 {
        let mut first = true;
        for arg in args {
            if !first {
                print!(" ");
            }
            // Arguments are already expanded by the executor
            print!("{}", arg);
            first = false;
        }
        println!();
        0
    }
    
    /// Exit shell
    fn exit(&self, args: &[String]) -> i32 {
        let exit_code = if args.is_empty() {
            0
        } else {
            args[0].parse().unwrap_or(1)
        };
        
        std::process::exit(exit_code);
    }
    
    /// Show help
    fn help(&self) -> i32 {
        println!("rs-dash-pro - A POSIX-compatible shell");
        println!();
        println!("Built-in commands:");
        println!("  cd [dir]      Change directory");
        println!("  pwd           Print working directory");
        println!("  echo [args]   Print arguments");
        println!("  exit [n]      Exit shell with status n");
        println!("  true          Return success");
        println!("  false         Return failure");
        println!("  help          Show this help");
        println!();
        println!("External commands are also supported via PATH.");
        0
    }
    
    /// Check if a command is a built-in
    pub fn is_builtin(&self, name: &str) -> bool {
        matches!(name, "cd" | "pwd" | "echo" | "exit" | "true" | "false" | "help")
    }
}