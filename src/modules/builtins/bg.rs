//! bg builtin command - send job to background

use crate::modules::env::ShellEnv;
use crate::modules::job_control::get_job_control;

use crate::modules::builtins::registry::BuiltinCommand;

/// bg builtin command
pub struct Bg;

impl BuiltinCommand for Bg {
    fn name(&self) -> &'static str {
        "bg"
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
                    eprintln!("bg: invalid job ID: {}", args[0]);
                    return 1;
                }
            }
        };
        
        match job_id {
            Some(id) => {
                match jc.background_job(id) {
                    Ok(_) => {
                        println!("Sent job {} to background", id);
                        0
                    }
                    Err(e) => {
                        eprintln!("bg: {}", e);
                        1
                    }
                }
            }
            None => {
                eprintln!("bg: no current job");
                1
            }
        }
    }
}