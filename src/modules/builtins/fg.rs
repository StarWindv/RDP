//! fg builtin command - bring job to foreground

use crate::modules::env::ShellEnv;
use crate::modules::job_control::get_job_control;

use crate::modules::builtins::registry::BuiltinCommand;

/// fg builtin command
pub struct Fg;

impl BuiltinCommand for Fg {
    fn name(&self) -> &'static str {
        "fg"
    }
    
    fn execute(&self, args: &[String], _env: &mut ShellEnv) -> i32 {
        let job_control = get_job_control();
        let mut jc = job_control.lock().unwrap();
        
        let job_id = if args.is_empty() {
            // Default to current job
            jc.current_foreground_job()
        } else {
            // Parse job ID from argument
            match args[0].parse::<usize>() {
                Ok(id) => Some(id),
                Err(_) => {
                    eprintln!("fg: invalid job ID: {}", args[0]);
                    return 1;
                }
            }
        };
        
        match job_id {
            Some(id) => {
                match jc.foreground_job(id) {
                    Ok(_) => {
                        println!("Brought job {} to foreground", id);
                        0
                    }
                    Err(e) => {
                        eprintln!("fg: {}", e);
                        1
                    }
                }
            }
            None => {
                eprintln!("fg: no current job");
                1
            }
        }
    }
}