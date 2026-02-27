//! set builtin command - set shell options

use crate::modules::env::ShellEnv;
use crate::modules::options::{get_options, ShellOption};

use crate::modules::builtins::registry::BuiltinCommand;

/// set builtin command
pub struct Set;

impl BuiltinCommand for Set {
    fn name(&self) -> &'static str {
        "set"
    }

    fn execute(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        let mut options = get_options();

        if args.is_empty() {
            // Print all variables and options
            println!("Current shell options:");
            println!("  Short options: {}", options.format_options());
            println!();

            println!("  Long options:");
            for opt in &[
                ShellOption::AllExport,
                ShellOption::IgnoreEof,
                ShellOption::Interactive,
                ShellOption::Monitor,
                ShellOption::NoClobber,
                ShellOption::NoGlob,
                ShellOption::NoLog,
                ShellOption::Notify,
                ShellOption::Physical,
                ShellOption::Posix,
                ShellOption::Vi,
            ] {
                let status = if options.is_enabled(*opt) {
                    "on"
                } else {
                    "off"
                };
                println!(
                    "    {:20} {} ({})",
                    format!("{}", opt.long_name()),
                    status,
                    opt.description()
                );
            }
            println!();

            println!("Positional parameters:");
            for (i, param) in options.get_positional_params().iter().enumerate() {
                println!("  ${} = '{}'", i + 1, param);
            }
            println!();

            println!("Environment variables:");
            for (key, value) in &env.vars {
                println!("  {}='{}'", key, value);
            }
            return 0;
        }

        // Parse options
        match options.apply_options(&args) {
            Ok(consumed) => {
                // Handle remaining arguments as positional parameters
                if consumed < args.len() {
                    let pos_args: Vec<String> = args[consumed..].to_vec();
                    options.set_positional_params(pos_args);
                }
                0
            }
            Err(e) => {
                eprintln!("{}", e);
                1
            }
        }
    }
}
