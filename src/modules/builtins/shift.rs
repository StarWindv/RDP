//! shift builtin command - shift positional parameters

use crate::modules::env::ShellEnv;
use crate::modules::options::get_options;

use crate::modules::builtins::registry::BuiltinCommand;

/// shift builtin command
pub struct Shift;

impl BuiltinCommand for Shift {
    fn name(&self) -> &'static str {
        "shift"
    }
    
    fn execute(&self, args: &[String], _env: &mut ShellEnv) -> i32 {
        let n = if args.is_empty() {
            1
        } else {
            match args[0].parse() {
                Ok(num) if num > 0 => num,
                _ => {
                    eprintln!("shift: invalid shift count: {}", args[0]);
                    return 1;
                }
            }
        };
        
        let mut options = get_options();
        let shifted = options.shift_positional_params(n);
        
        if shifted.is_empty() && n > 0 {
            eprintln!("shift: can't shift that many");
            return 1;
        }
        
        0
    }
}