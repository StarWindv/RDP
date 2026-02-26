//! Job control system for POSIX shell
//! Implements background execution, process groups, and job management

use std::collections::HashMap;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime};

use crate::modules::env::ShellEnv;

/// Job status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    Running,
    Stopped,
    Done,
}

/// Job information
#[derive(Debug)]
pub struct Job {
    pub id: usize,
    pub pgid: i32,
    pub command: String,
    pub status: JobStatus,
    pub foreground: bool,
    pub start_time: SystemTime,
    pub children: Vec<Child>,
}

/// Job control system
#[derive(Debug)]
pub struct JobControl {
    jobs: HashMap<usize, Job>,
    next_job_id: usize,
    current_foreground_job: Option<usize>,
}

impl JobControl {
    /// Create a new job control system
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
            next_job_id: 1,
            current_foreground_job: None,
        }
    }
    
    /// Add a new job
    pub fn add_job(&mut self, pgid: i32, command: String, foreground: bool, child: Child) -> usize {
        let id = self.next_job_id;
        self.next_job_id += 1;
        
        let job = Job {
            id,
            pgid,
            command: command.clone(),
            status: JobStatus::Running,
            foreground,
            start_time: SystemTime::now(),
            children: vec![child],
        };
        
        self.jobs.insert(id, job);
        
        if foreground {
            self.current_foreground_job = Some(id);
        }
        
        id
    }
    
    /// Update job status
    pub fn update_job_status(&mut self, job_id: usize, status: JobStatus) {
        if let Some(job) = self.jobs.get_mut(&job_id) {
            job.status = status;
            
            if status == JobStatus::Done && job.foreground {
                self.current_foreground_job = None;
            }
        }
    }
    
    /// Get job by ID
    pub fn get_job(&self, job_id: usize) -> Option<&Job> {
        self.jobs.get(&job_id)
    }
    
    /// Get job by process group ID
    pub fn get_job_by_pgid(&self, pgid: i32) -> Option<&Job> {
        self.jobs.values().find(|job| job.pgid == pgid)
    }
    
    /// Get all jobs
    pub fn get_all_jobs(&self) -> Vec<&Job> {
        self.jobs.values().collect()
    }
    
    /// Get running jobs
    pub fn get_running_jobs(&self) -> Vec<&Job> {
        self.jobs.values()
            .filter(|job| job.status == JobStatus::Running)
            .collect()
    }
    
    /// Get stopped jobs
    pub fn get_stopped_jobs(&self) -> Vec<&Job> {
        self.jobs.values()
            .filter(|job| job.status == JobStatus::Stopped)
            .collect()
    }
    
    /// Get current foreground job ID
    pub fn current_foreground_job(&self) -> Option<usize> {
        self.current_foreground_job
    }
    
    /// Bring job to foreground
    pub fn foreground_job(&mut self, job_id: usize) -> Result<(), String> {
        if let Some(job) = self.jobs.get_mut(&job_id) {
            job.foreground = true;
            self.current_foreground_job = Some(job_id);
            
            // TODO: Actually bring process group to foreground with tcsetpgrp
            // For now, just mark it as foreground
            
            Ok(())
        } else {
            Err(format!("Job {} not found", job_id))
        }
    }
    
    /// Send job to background
    pub fn background_job(&mut self, job_id: usize) -> Result<(), String> {
        if let Some(job) = self.jobs.get_mut(&job_id) {
            job.foreground = false;
            
            if self.current_foreground_job == Some(job_id) {
                self.current_foreground_job = None;
            }
            
            // TODO: Actually send SIGCONT to process group
            // For now, just mark it as background
            
            Ok(())
        } else {
            Err(format!("Job {} not found", job_id))
        }
    }
    
    /// Send signal to job
    pub fn signal_job(&mut self, job_id: usize, signal: i32) -> Result<(), String> {
        if let Some(job) = self.jobs.get_mut(&job_id) {
            // TODO: Actually send signal to process group
            // For now, just update status based on signal
            
            match signal {
                // SIGSTOP
                19 => {
                    job.status = JobStatus::Stopped;
                }
                // SIGCONT
                18 => {
                    job.status = JobStatus::Running;
                }
                // SIGINT (2) or SIGTERM (15)
                2 | 15 => {
                    job.status = JobStatus::Done;
                }
                _ => {}
            }
            
            Ok(())
        } else {
            Err(format!("Job {} not found", job_id))
        }
    }
    
    /// Wait for job to complete
    pub fn wait_for_job(&mut self, job_id: usize) -> Result<i32, String> {
        if let Some(job) = self.jobs.get_mut(&job_id) {
            let mut last_status = 0;
            
            // Wait for all child processes in the job
            for child in &mut job.children {
                match child.wait() {
                    Ok(status) => {
                        last_status = status.code().unwrap_or(128);
                    }
                    Err(e) => {
                        return Err(format!("Failed to wait for process: {}", e));
                    }
                }
            }
            
            // Mark as done
            job.status = JobStatus::Done;
            
            if job.foreground {
                self.current_foreground_job = None;
            }
            
            Ok(last_status)
        } else {
            Err(format!("Job {} not found", job_id))
        }
    }
    
    /// Clean up finished jobs
    pub fn cleanup_finished_jobs(&mut self) {
        let finished_ids: Vec<usize> = self.jobs.iter()
            .filter(|(_, job)| job.status == JobStatus::Done)
            .map(|(id, _)| *id)
            .collect();
        
        for id in finished_ids {
            self.jobs.remove(&id);
        }
    }
    
    /// Format job for display
    pub fn format_job(&self, job: &Job) -> String {
        let status_symbol = match job.status {
            JobStatus::Running => "+",
            JobStatus::Stopped => "S",
            JobStatus::Done => "D",
        };
        
        let fg_marker = if job.foreground { " (fg)" } else { " (bg)" };
        
        format!("[{}] {} {}: {}{}", 
            job.id, status_symbol, job.pgid, job.command, fg_marker)
    }
}

/// Global job control instance
lazy_static::lazy_static! {
    static ref JOB_CONTROL: Arc<Mutex<JobControl>> = Arc::new(Mutex::new(JobControl::new()));
}

/// Get global job control instance
pub fn get_job_control() -> Arc<Mutex<JobControl>> {
    JOB_CONTROL.clone()
}

/// Initialize job control for the shell
pub fn init_job_control() -> Result<(), String> {
    // TODO: Set up signal handlers for SIGINT, SIGTSTP, SIGCONT
    // TODO: Set up terminal control
    
    Ok(())
}

/// Execute a command in background
pub fn execute_background(command: &str, args: &[String], env: &ShellEnv) -> Result<usize, String> {
    let mut cmd = Command::new(command);
    
    for arg in args {
        cmd.arg(arg);
    }
    
    // Set up environment
    cmd.current_dir(&env.current_dir);
    for (key, value) in &env.vars {
        cmd.env(key, value);
    }
    
    // Start process in background
    match cmd.spawn() {
        Ok(child) => {
            let pid = child.id() as i32;
            let job_control = get_job_control();
            let mut jc = job_control.lock().unwrap();
            
            let job_id = jc.add_job(pid, format!("{} {}", command, args.join(" ")), false, child);
            
            Ok(job_id)
        }
        Err(e) => Err(format!("Failed to execute command: {}", e)),
    }
}

/// Execute a command in foreground
pub fn execute_foreground(command: &str, args: &[String], env: &ShellEnv) -> Result<i32, String> {
    let mut cmd = Command::new(command);
    
    for arg in args {
        cmd.arg(arg);
    }
    
    // Set up environment
    cmd.current_dir(&env.current_dir);
    for (key, value) in &env.vars {
        cmd.env(key, value);
    }
    
    // Start process in foreground
    match cmd.spawn() {
        Ok(child) => {
            let pid = child.id() as i32;
            let job_control = get_job_control();
            let mut jc = job_control.lock().unwrap();
            
            let job_id = jc.add_job(pid, format!("{} {}", command, args.join(" ")), true, child);
            
            // Wait for process to complete
            match jc.wait_for_job(job_id) {
                Ok(status) => Ok(status),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(format!("Failed to execute command: {}", e)),
    }
}