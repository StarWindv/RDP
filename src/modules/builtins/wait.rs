//! wait builtin command - wait for job completion

use crate::modules::env::ShellEnv;
use crate::modules::job_control::get_job_control;

use crate::modules::builtins::registry::BuiltinCommand;

/// wait builtin command
pub struct Wait;

impl BuiltinCommand for Wait {
    fn name(&self) -> &'static str {
        "wait"
    }

    fn execute(&self, args: &[String], _env: &mut ShellEnv) -> i32 {
        let job_control = get_job_control();
        let mut jc = job_control.lock().unwrap();

        if args.is_empty() {
            // Wait for all jobs
            let jobs: Vec<usize> = jc.get_all_jobs().iter().map(|j| j.id).collect();
            let mut last_status = 0;

            for job_id in jobs {
                match jc.wait_for_job(job_id) {
                    Ok(status) => last_status = status,
                    Err(e) => {
                        eprintln!("wait: {}", e);
                        return 1;
                    }
                }
            }

            last_status
        } else {
            // Wait for specific job
            match args[0].parse::<usize>() {
                Ok(job_id) => match jc.wait_for_job(job_id) {
                    Ok(status) => status,
                    Err(e) => {
                        eprintln!("wait: {}", e);
                        1
                    }
                },
                Err(_) => {
                    eprintln!("wait: invalid job ID: {}", args[0]);
                    1
                }
            }
        }
    }
}
