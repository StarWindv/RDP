//! Safe job control system for POSIX shell
//! Implements basic job control using safe Rust abstractions where possible

use std::collections::HashMap;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use crate::modules::env::ShellEnv;

/// Job status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    Running,
    Stopped,
    Done(i32), // Exit status or signal number
}

/// Job information
#[derive(Debug)]
pub struct Job {
    pub id: usize,
    pub pid: u32,           // Process ID (not process group ID for simplicity)
    pub command: String,
    pub status: JobStatus,
    pub foreground: bool,
    pub start_time: SystemTime,
    pub child: Option<Child>, // Store child process handle
}

/// Safe job control system
#[derive(Debug)]
pub struct SafeJobControl {
    jobs: HashMap<usize, Job>,
    next_job_id: usize,
    current_foreground_job: Option<usize>,
}

impl SafeJobControl {
    /// Create a new safe job control system
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
            next_job_id: 1,
            current_foreground_job: None,
        }
    }
    
    /// Add a new job
    pub fn add_job(&mut self, pid: u32, command: String, foreground: bool, child: Child) -> usize {
        let id = self.next_job_id;
        self.next_job_id += 1;
        
        let job = Job {
            id,
            pid,
            command: command.clone(),
            status: JobStatus::Running,
            foreground,
            start_time: SystemTime::now(),
            child: Some(child),
        };
        
        self.jobs.insert(id, job);
        
        if foreground {
            self.current_foreground_job = Some(id);
        }
        
        id
    }
    
    /// Update job status
    pub fn update_job_status(&mut self, job_id: usize, status: JobStatus) -> Result<(), String> {
        if let Some(job) = self.jobs.get_mut(&job_id) {
            job.status = status;
            
            if matches!(status, JobStatus::Done(_)) && job.foreground {
                self.current_foreground_job = None;
            }
            
            Ok(())
        } else {
            Err(format!("Job {} not found", job_id))
        }
    }
    
    /// Check and update job status by polling
    pub fn poll_job_status(&mut self, job_id: usize) -> Result<Option<i32>, String> {
        if let Some(job) = self.jobs.get_mut(&job_id) {
            if let Some(ref mut child) = job.child {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        let exit_status = status.code().unwrap_or(128);
                        job.status = JobStatus::Done(exit_status);
                        job.child = None; // Child is no longer running
                        
                        if job.foreground {
                            self.current_foreground_job = None;
                        }
                        
                        Ok(Some(exit_status))
                    }
                    Ok(None) => {
                        // Child still running
                        Ok(None)
                    }
                    Err(e) => Err(format!("Failed to check job status: {}", e)),
                }
            } else {
                // Child already finished
                if let JobStatus::Done(status) = job.status {
                    Ok(Some(status))
                } else {
                    Ok(None)
                }
            }
        } else {
            Err(format!("Job {} not found", job_id))
        }
    }
    
    /// Wait for job to complete
    pub fn wait_for_job(&mut self, job_id: usize) -> Result<i32, String> {
        if let Some(job) = self.jobs.get_mut(&job_id) {
            if let Some(ref mut child) = job.child {
                match child.wait() {
                    Ok(status) => {
                        let exit_status = status.code().unwrap_or(128);
                        job.status = JobStatus::Done(exit_status);
                        job.child = None;
                        
                        if job.foreground {
                            self.current_foreground_job = None;
                        }
                        
                        Ok(exit_status)
                    }
                    Err(e) => Err(format!("Failed to wait for job: {}", e)),
                }
            } else {
                // Child already finished
                if let JobStatus::Done(status) = job.status {
                    Ok(status)
                } else {
                    Err(format!("Job {} has no child process", job_id))
                }
            }
        } else {
            Err(format!("Job {} not found", job_id))
        }
    }
    
    /// Get job by ID
    pub fn get_job(&self, job_id: usize) -> Option<&Job> {
        self.jobs.get(&job_id)
    }
    
    /// Get job by PID
    pub fn get_job_by_pid(&self, pid: u32) -> Option<&Job> {
        self.jobs.values().find(|job| job.pid == pid)
    }
    
    /// Get all jobs
    pub fn get_all_jobs(&self) -> Vec<&Job> {
        self.jobs.values().collect()
    }
    
    /// Get running jobs
    pub fn get_running_jobs(&self) -> Vec<&Job> {
        self.jobs.values()
            .filter(|job| matches!(job.status, JobStatus::Running))
            .collect()
    }
    
    /// Get stopped jobs (in our simplified model, "stopped" means not running but not done)
    pub fn get_stopped_jobs(&self) -> Vec<&Job> {
        self.jobs.values()
            .filter(|job| matches!(job.status, JobStatus::Stopped))
            .collect()
    }
    
    /// Get current foreground job ID
    pub fn current_foreground_job(&self) -> Option<usize> {
        self.current_foreground_job
    }
    
    /// Bring job to foreground (simplified - just mark as foreground)
    pub fn foreground_job(&mut self, job_id: usize) -> Result<(), String> {
        if let Some(job) = self.jobs.get_mut(&job_id) {
            job.foreground = true;
            self.current_foreground_job = Some(job_id);
            Ok(())
        } else {
            Err(format!("Job {} not found", job_id))
        }
    }
    
    /// Send job to background (simplified - just mark as background)
    pub fn background_job(&mut self, job_id: usize) -> Result<(), String> {
        if let Some(job) = self.jobs.get_mut(&job_id) {
            job.foreground = false;
            
            if self.current_foreground_job == Some(job_id) {
                self.current_foreground_job = None;
            }
            
            Ok(())
        } else {
            Err(format!("Job {} not found", job_id))
        }
    }
    
    /// Send signal to job (simplified - only supports termination)
    pub fn signal_job(&mut self, job_id: usize) -> Result<(), String> {
        if let Some(job) = self.jobs.get_mut(&job_id) {
            if let Some(ref mut child) = job.child {
                if let Err(e) = child.kill() {
                    return Err(format!("Failed to kill job: {}", e));
                }
                
                // Wait for it to die
                match child.wait() {
                    Ok(status) => {
                        let exit_status = status.code().unwrap_or(128);
                        job.status = JobStatus::Done(exit_status);
                        job.child = None;
                        
                        if job.foreground {
                            self.current_foreground_job = None;
                        }
                    }
                    Err(e) => return Err(format!("Failed to wait after kill: {}", e)),
                }
            } else {
                // Job already finished
                job.status = JobStatus::Done(130); // SIGINT = 130
            }
            
            Ok(())
        } else {
            Err(format!("Job {} not found", job_id))
        }
    }
    
    /// Clean up finished jobs
    pub fn cleanup_finished_jobs(&mut self) {
        let finished_ids: Vec<usize> = self.jobs.iter()
            .filter(|(_, job)| matches!(job.status, JobStatus::Done(_)))
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
            JobStatus::Done(status) => {
                if status == 0 {
                    "D"
                } else {
                    &format!("D({})", status)
                }
            }
        };
        
        let fg_marker = if job.foreground { " (fg)" } else { " (bg)" };
        
        format!("[{}] {} {}: {}{}", 
            job.id, status_symbol, job.pid, job.command, fg_marker)
    }
    
    /// Poll all jobs and return those that changed status
    pub fn poll_all_jobs(&mut self) -> Result<Vec<(usize, i32)>, String> {
        let mut changed = Vec::new();
        let job_ids: Vec<usize> = self.jobs.keys().copied().collect();
        
        for job_id in job_ids {
            if let Ok(Some(status)) = self.poll_job_status(job_id) {
                changed.push((job_id, status));
            }
        }
        
        Ok(changed)
    }
}

/// Global safe job control instance
lazy_static::lazy_static! {
    static ref SAFE_JOB_CONTROL: Arc<Mutex<SafeJobControl>> = 
        Arc::new(Mutex::new(SafeJobControl::new()));
}

/// Get global safe job control instance
pub fn get_safe_job_control() -> Arc<Mutex<SafeJobControl>> {
    SAFE_JOB_CONTROL.clone()
}

/// Execute a command with simplified job control
pub fn execute_with_safe_job_control(
    command: &str, 
    args: &[String], 
    env: &ShellEnv,
    foreground: bool,
) -> Result<usize, String> {
    let mut cmd = Command::new(command);
    
    for arg in args {
        cmd.arg(arg);
    }
    
    // Set up environment
    cmd.current_dir(&env.current_dir);
    for (key, value) in &env.vars {
        cmd.env(key, value);
    }
    
    // Start process
    match cmd.spawn() {
        Ok(child) => {
            let pid = child.id();
            let job_control = get_safe_job_control();
            let mut jc = job_control.lock().unwrap();
            
            let job_id = jc.add_job(pid, format!("{} {}", command, args.join(" ")), foreground, child);
            
            if foreground {
                // In foreground mode, wait for completion
                match jc.wait_for_job(job_id) {
                    Ok(status) => {
                        // Return job ID and status will be available via poll
                        Ok(job_id)
                    }
                    Err(e) => Err(e),
                }
            } else {
                // In background mode, just return job ID
                Ok(job_id)
            }
        }
        Err(e) => Err(format!("Failed to execute command: {}", e)),
    }
}

/// Built-in jobs command for safe job control
pub fn safe_jobs_builtin(_args: &[String]) -> i32 {
    let job_control = get_safe_job_control();
    let jc = job_control.lock().unwrap();
    
    // Poll all jobs first to update statuses
    drop(jc); // Release lock to allow polling
    
    let mut jc = job_control.lock().unwrap();
    let _ = jc.poll_all_jobs(); // Ignore errors
    
    let all_jobs = jc.get_all_jobs();
    
    if all_jobs.is_empty() {
        println!("No jobs");
        return 0;
    }
    
    for job in all_jobs {
        println!("{}", jc.format_job(job));
    }
    
    0
}

/// Built-in fg command for safe job control
pub fn safe_fg_builtin(args: &[String]) -> i32 {
    let job_control = get_safe_job_control();
    let mut jc = job_control.lock().unwrap();
    
    let job_id = if args.is_empty() {
        // Use current foreground job
        jc.current_foreground_job()
    } else {
        // Parse job spec
        let spec = &args[0];
        if spec.starts_with('%') {
            spec[1..].parse().ok()
        } else {
            spec.parse().ok()
        }
    };
    
    match job_id {
        Some(job_id) => {
            // Mark as foreground and wait for it
            if let Err(e) = jc.foreground_job(job_id) {
                eprintln!("fg: {}", e);
                return 1;
            }
            
            match jc.wait_for_job(job_id) {
                Ok(status) => status,
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

/// Built-in bg command for safe job control
pub fn safe_bg_builtin(args: &[String]) -> i32 {
    let job_control = get_safe_job_control();
    let mut jc = job_control.lock().unwrap();
    
    let job_id = if args.is_empty() {
        // Use current foreground job
        jc.current_foreground_job()
    } else {
        // Parse job spec
        let spec = &args[0];
        if spec.starts_with('%') {
            spec[1..].parse().ok()
        } else {
            spec.parse().ok()
        }
    };
    
    match job_id {
        Some(job_id) => {
            match jc.background_job(job_id) {
                Ok(_) => {
                    println!("[{}] continued in background", job_id);
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

/// Built-in wait command for safe job control
pub fn safe_wait_builtin(args: &[String]) -> i32 {
    let job_control = get_safe_job_control();
    let mut jc = job_control.lock().unwrap();
    
    if args.is_empty() {
        // Wait for all jobs
        let job_ids: Vec<usize> = jc.get_all_jobs().iter().map(|j| j.id).collect();
        let mut last_status = 0;
        
        for job_id in job_ids {
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
        let spec = &args[0];
        let job_id = if spec.starts_with('%') {
            spec[1..].parse().ok()
        } else {
            spec.parse().ok()
        };
        
        match job_id {
            Some(job_id) => {
                match jc.wait_for_job(job_id) {
                    Ok(status) => status,
                    Err(e) => {
                        eprintln!("wait: {}", e);
                        1
                    }
                }
            }
            None => {
                eprintln!("wait: invalid job specification");
                1
            }
        }
    }
}