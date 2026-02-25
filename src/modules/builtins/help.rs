//! help builtin command - show help

use crate::modules::env::ShellEnv;

use crate::modules::builtins::registry::BuiltinCommand;

/// help builtin command
pub struct Help;

impl BuiltinCommand for Help {
    fn name(&self) -> &'static str {
        "help"
    }
    
    fn execute(&self, _args: &[String], _env: &mut ShellEnv) -> i32 {
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
}