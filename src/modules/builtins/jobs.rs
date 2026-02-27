//! jobs builtin command - list jobs

use crate::modules::env::ShellEnv;
use crate::modules::job_control::get_job_control;

use crate::modules::builtins::registry::BuiltinCommand;

/// jobs builtin command
pub struct Jobs;

impl BuiltinCommand for Jobs {
    fn name(&self) -> &'static str {
        "jobs"
    }

    fn execute(&self, _args: &[String], _env: &mut ShellEnv) -> i32 {
        let job_control = get_job_control();
        let jc = job_control.lock().unwrap();

        let jobs = jc.get_all_jobs();

        if jobs.is_empty() {
            println!("No jobs running");
        } else {
            for job in jobs {
                println!("{}", jc.format_job(job));
            }
        }

        0
    }
}
