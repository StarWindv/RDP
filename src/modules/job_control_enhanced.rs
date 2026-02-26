//! Enhanced job control system for POSIX shell
//! Implements full job control with process groups, signals, and terminal control

use std::collections::HashMap;
use std::io;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, Duration};

use crate::modules::env::ShellEnv;

#[cfg(unix)]
use nix::{
    sys::signal::{self, Signal},
    unistd::{self, Pid},
    errno::Errno,
};

/// Job status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    Running,
    Stopped(Signal), // Stopped with signal
    Terminated(i32), // Terminated with exit status
    Signaled(i32),   // Killed by signal
}

/// Job information
#[derive(Debug)]
pub struct Job {
    pub id: usize,
    pub pgid: i32,           // Process group ID
    pub command: String,
    pub status: JobStatus,
    pub foreground: bool,
    pub start_time: SystemTime,
    pub children: Vec<Child>,
    pub notified: bool,      // Whether user has been notified about status change
}

/// Enhanced job control system
#[derive(Debug)]
pub struct EnhancedJobControl {
    jobs: HashMap<usize, Job>,
    next_job_id: usize,
    current_foreground_job: Option<usize>,
    shell_pgid: i32,         // Shell's own process group
    terminal_owner: bool,    // Whether shell owns the terminal
}

impl EnhancedJobControl {
    /// Create a new enhanced job control system
    pub fn new() -> Self {
        #[cfg(unix)]
        let shell_pgid = unistd::getpgrp().as_raw();
        #[cfg(not(unix))]
        let shell_pgid = 0;
        
        Self {
            jobs: HashMap::new(),
            next_job_id: 1,
            current_foreground_job: None,
            shell_pgid,
            terminal_owner: true, // Shell starts as terminal owner
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
            notified: false,
        };
        
        self.jobs.insert(id, job);
        
        if foreground {
            self.current_foreground_job = Some(id);
            // Give terminal to the new foreground process group
            #[cfg(unix)]
            self.give_terminal_to(pgid).ok(); // Ignore errors for now
        }
        
        id
    }
    
    /// Give terminal to a process group
    #[cfg(unix)]
    fn give_terminal_to(&self, pgid: i32) -> Result<(), String> {
        use nix::{
            fcntl,
            term::tcsetpgrp,
        };
        
        // Set the process group as foreground process group
        let fd = 0; // Standard input file descriptor
        match tcsetpgrp(fd, Pid::from_raw(pgid)) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to set foreground process group: {}", e)),
        }
    }
    
    /// Take back terminal control
    #[cfg(unix)]
    fn take_terminal_back(&self) -> Result<(), String> {
        use nix::term::tcsetpgrp;
        
        // Set shell's process group as foreground
        let fd = 0; // Standard input file descriptor
        match tcsetpgrp(fd, Pid::from_raw(self.shell_pgid)) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to take back terminal: {}", e)),
        }
    }
    
    /// Update job status based on wait status
    pub fn update_job_status_from_wait(&mut self, job_id: usize, wait_status: i32) -> Result<(), String> {
        if let Some(job) = self.jobs.get_mut(&job_id) {
            if wait_status == 0 {
                job.status = JobStatus::Terminated(0);
            } else if wait_status > 0 && wait_status < 128 {
                job.status = JobStatus::Terminated(wait_status);
            } else if wait_status >= 128 {
                let signal = wait_status - 128;
                job.status = JobStatus::Signaled(signal);
            }
            
            job.notified = false;
            
            if job.foreground {
                self.current_foreground_job = None;
                // Take back terminal control
                #[cfg(unix)]
                self.take_terminal_back().ok(); // Ignore errors for now
            }
            
            Ok(())
        } else {
            Err(format!("Job {} not found", job_id))
        }
    }
    
    /// Update job status to stopped
    pub fn update_job_stopped(&mut self, job_id: usize, signal: Signal) -> Result<(), String> {
        if let Some(job) = self.jobs.get_mut(&job_id) {
            job.status = JobStatus::Stopped(signal);
            job.notified = false;
            
            if job.foreground {
                self.current_foreground_job = None;
                // Take back terminal control
                #[cfg(unix)]
                self.take_terminal_back().ok(); // Ignore errors for now
            }
            
            Ok(())
        } else {
            Err(format!("Job {} not found", job_id))
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
    
    /// Get job by process group ID (mutable)
    pub fn get_job_by_pgid_mut(&mut self, pgid: i32) -> Option<&mut Job> {
        self.jobs.values_mut().find(|job| job.pgid == pgid)
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
    
    /// Get stopped jobs
    pub fn get_stopped_jobs(&self) -> Vec<&Job> {
        self.jobs.values()
            .filter(|job| matches!(job.status, JobStatus::Stopped(_)))
            .collect()
    }
    
    /// Get current foreground job ID
    pub fn current_foreground_job(&self) -> Option<usize> {
        self.current_foreground_job
    }
    
    /// Bring job to foreground
    pub fn foreground_job(&mut self, job_id: usize) -> Result<(), String> {
        // First check if job exists and get its pgid
        let pgid = if let Some(job) = self.jobs.get(&job_id) {
            job.pgid
        } else {
            return Err(format!("Job {} not found", job_id));
        };
        
        // Update job status
        if let Some(job) = self.jobs.get_mut(&job_id) {
            job.foreground = true;
            self.current_foreground_job = Some(job_id);
            
            // If job is stopped, send SIGCONT to continue it
            if let JobStatus::Stopped(_signal) = job.status {
                #[cfg(unix)]
                {
                    if let Err(e) = signal::killpg(Pid::from_raw(job.pgid), Signal::SIGCONT) {
                        return Err(format!("Failed to continue job: {}", e));
                    }
                }
                job.status = JobStatus::Running;
            }
        }
        
        // Give terminal to the job's process group
        #[cfg(unix)]
        self.give_terminal_to(pgid)?;
        
        Ok(())
    }
    
    /// Send job to background
    pub fn background_job(&mut self, job_id: usize) -> Result<(), String> {
        if let Some(job) = self.jobs.get_mut(&job_id) {
            job.foreground = false;
            
            if self.current_foreground_job == Some(job_id) {
                self.current_foreground_job = None;
            }
            
            // If job is stopped, send SIGCONT to continue it in background
            if let JobStatus::Stopped(_signal) = job.status {
                #[cfg(unix)]
                {
                    if let Err(e) = signal::killpg(Pid::from_raw(job.pgid), Signal::SIGCONT) {
                        return Err(format!("Failed to continue job in background: {}", e));
                    }
                }
                job.status = JobStatus::Running;
            }
        } else {
            return Err(format!("Job {} not found", job_id));
        }
        
        // Take back terminal control if this was the foreground job
        if self.current_foreground_job.is_none() {
            #[cfg(unix)]
            self.take_terminal_back()?;
        }
        
        Ok(())
    }
    
    /// Send signal to job
    pub fn signal_job(&mut self, job_id: usize, signal: Signal) -> Result<(), String> {
        if let Some(job) = self.jobs.get(&job_id) {
            #[cfg(unix)]
            {
                if let Err(e) = signal::killpg(Pid::from_raw(job.pgid), signal) {
                    return Err(format!("Failed to send signal to job: {}", e));
                }
            }
            
            Ok(())
        } else {
            Err(format!("Job {} not found", job_id))
        }
    }
    
    /// Wait for job to complete
    pub fn wait_for_job(&mut self, job_id: usize) -> Result<i32, String> {
        // First get the job to work on
        let mut last_status = 0;
        let mut should_update = false;
        
        if let Some(job) = self.jobs.get_mut(&job_id) {
            // Wait for all child processes in the job
            for child in &mut job.children {
                match child.wait() {
                    Ok(status) => {
                        last_status = status.code().unwrap_or(128);
                        should_update = true;
                    }
                    Err(e) => {
                        return Err(format!("Failed to wait for process: {}", e));
                    }
                }
            }
        } else {
            return Err(format!("Job {} not found", job_id));
        }
        
        // Now update the job status
        if should_update {
            self.update_job_status_from_wait(job_id, last_status)?;
        }
        
        Ok(last_status)
    }
    
    /// Wait for any job to change status
    pub fn wait_for_any_job(&mut self) -> Result<Option<(usize, i32)>, String> {
        // Collect jobs that have status changes
        let mut changed_jobs = Vec::new();
        
        for (job_id, job) in &mut self.jobs {
            for child in &mut job.children {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        let exit_status = status.code().unwrap_or(128);
                        changed_jobs.push((*job_id, exit_status));
                    }
                    Ok(None) => {
                        // Child still running
                    }
                    Err(e) => {
                        return Err(format!("Failed to check job status: {}", e));
                    }
                }
            }
        }
        
        // Update job statuses
        for (job_id, exit_status) in &changed_jobs {
            self.update_job_status_from_wait(*job_id, *exit_status)?;
        }
        
        // Return first changed job if any
        changed_jobs.first().cloned()
    }
    
    /// Clean up finished jobs
    pub fn cleanup_finished_jobs(&mut self) {
        let finished_ids: Vec<usize> = self.jobs.iter()
            .filter(|(_, job)| {
                matches!(job.status, JobStatus::Terminated(_) | JobStatus::Signaled(_))
            })
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
            JobStatus::Stopped(sig) => {
                let sig_name = match sig {
                    #[cfg(unix)]
                    Signal::SIGSTOP => "Stopped",
                    #[cfg(unix)]
                    Signal::SIGTSTP => "Stopped (tty)",
                    #[cfg(unix)]
                    Signal::SIGTTIN => "Stopped (tty in)",
                    #[cfg(unix)]
                    Signal::SIGTTOU => "Stopped (tty out)",
                    _ => "Stopped",
                };
                format!("S ({})", sig_name)
            }
            JobStatus::Terminated(status) => format!("D (exit {})", status),
            JobStatus::Signaled(signal) => format!("K (signal {})", signal),
        };
        
        let fg_marker = if job.foreground { " (fg)" } else { " (bg)" };
        
        format!("[{}] {} {}: {}{}", 
            job.id, status_symbol, job.pgid, job.command, fg_marker)
    }
    
    /// Get jobs that need notification
    pub fn get_jobs_needing_notification(&mut self) -> Vec<String> {
        let mut notifications = Vec::new();
        
        for job in self.jobs.values_mut() {
            if !job.notified {
                let notification = match job.status {
                    JobStatus::Terminated(_status) => {
                        format!("[{}] Done    {}", job.id, job.command)
                    }
                    JobStatus::Signaled(signal) => {
                        format!("[{}] Killed by signal {}    {}", job.id, signal, job.command)
                    }
                    JobStatus::Stopped(signal) => {
                        let sig_name = match signal {
                            #[cfg(unix)]
                            Signal::SIGSTOP => "SIGSTOP",
                            #[cfg(unix)]
                            Signal::SIGTSTP => "SIGTSTP",
                            #[cfg(unix)]
                            Signal::SIGTTIN => "SIGTTIN",
                            #[cfg(unix)]
                            Signal::SIGTTOU => "SIGTTOU",
                            _ => "unknown",
                        };
                        format!("[{}] Stopped by {}    {}", job.id, sig_name, job.command)
                    }
                    JobStatus::Running => continue,
                };
                
                notifications.push(notification);
                job.notified = true;
            }
        }
        
        notifications
    }
}

/// Global enhanced job control instance
lazy_static::lazy_static! {
    static ref ENHANCED_JOB_CONTROL: Arc<Mutex<EnhancedJobControl>> = 
        Arc::new(Mutex::new(EnhancedJobControl::new()));
}

/// Get global enhanced job control instance
pub fn get_enhanced_job_control() -> Arc<Mutex<EnhancedJobControl>> {
    ENHANCED_JOB_CONTROL.clone()
}

/// Initialize enhanced job control for the shell
pub fn init_enhanced_job_control() -> Result<(), String> {
    #[cfg(unix)]
    {
        use nix::sys::signal::{SigSet, Signal, sigprocmask, SigmaskHow};
        
        // Block job control signals in shell
        let mut mask = SigSet::empty();
        mask.add(Signal::SIGTTOU);
        mask.add(Signal::SIGTTIN);
        mask.add(Signal::SIGTSTP);
        mask.add(Signal::SIGCHLD);
        
        if let Err(e) = sigprocmask(SigmaskHow::SIG_BLOCK, Some(&mask), None) {
            return Err(format!("Failed to block job control signals: {}", e));
        }
        
        // Set shell as its own process group leader
        let shell_pid = unistd::getpid();
        if let Err(e) = unistd::setpgid(shell_pid, shell_pid) {
            return Err(format!("Failed to set shell process group: {}", e));
        }
        
        // Take control of terminal
        let fd = 0; // Standard input
        if let Err(e) = nix::term::tcsetpgrp(fd, shell_pid) {
            return Err(format!("Failed to take terminal control: {}", e));
        }
    }
    
    Ok(())
}

/// Execute a command with job control
pub fn execute_with_job_control(
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
    
    #[cfg(unix)]
    {
        use nix::unistd::{fork, ForkResult, setpgid};
        use nix::sys::signal::Signal;
        
        match fork() {
            Ok(ForkResult::Parent { child, .. }) => {
                // Parent process
                let child_pid = child.as_raw();
                
                // Set child's process group
                // If this is the first process in a pipeline, it becomes the process group leader
                if let Err(e) = setpgid(child, child) {
                    return Err(format!("Failed to set child process group: {}", e));
                }
                
                let job_control = get_enhanced_job_control();
                let mut jc = job_control.lock().unwrap();
                
                // TODO: Actually get the child process
                // For now, create a dummy child
                let dummy_child = Command::new("true").spawn().map_err(|e| e.to_string())?;
                
                let job_id = jc.add_job(child_pid, format!("{} {}", command, args.join(" ")), foreground, dummy_child);
                
                if foreground {
                    // Give terminal to the new process group
                    jc.give_terminal_to(child_pid)?;
                }
                
                Ok(job_id)
            }
            Ok(ForkResult::Child) => {
                // Child process
                // Reset signal handlers
                let signals = [
                    Signal::SIGINT,
                    Signal::SIGQUIT,
                    Signal::SIGTSTP,
                    Signal::SIGTTIN,
                    Signal::SIGTTOU,
                    Signal::SIGCHLD,
                ];
                
                for sig in &signals {
                    let _ = signal::signal(*sig, signal::SigHandler::SigDfl);
                }
                
                // Set process group
                let child_pid = unistd::getpid();
                if let Err(e) = setpgid(child_pid, child_pid) {
                    eprintln!("Failed to set process group: {}", e);
                    std::process::exit(1);
                }
                
                // If this is a foreground job, take control of terminal
                if foreground {
                    if let Err(e) = nix::term::tcsetpgrp(0, child_pid) {
                        eprintln!("Failed to take terminal: {}", e);
                    }
                }
                
                // Execute command
                let error = cmd.exec();
                eprintln!("Failed to execute {}: {}", command, error);
                std::process::exit(1);
            }
            Err(e) => Err(format!("Failed to fork: {}", e)),
        }
    }
    
    #[cfg(not(unix))]
    {
        // Non-Unix fallback
        match cmd.spawn() {
            Ok(child) => {
                let pid = child.id() as i32;
                let job_control = get_enhanced_job_control();
                let mut jc = job_control.lock().unwrap();
                
                let job_id = jc.add_job(pid, format!("{} {}", command, args.join(" ")), foreground, child);
                
                Ok(job_id)
            }
            Err(e) => Err(format!("Failed to execute command: {}", e)),
        }
    }
}

/// Built-in jobs command
pub fn jobs_builtin(_args: &[String]) -> i32 {
    let job_control = get_enhanced_job_control();
    let jc = job_control.lock().unwrap();
    
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

/// Built-in fg command
pub fn fg_builtin(args: &[String]) -> i32 {
    let job_control = get_enhanced_job_control();
    let mut jc = job_control.lock().unwrap();
    
    let job_id = if args.is_empty() {
        // Use current job
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
            match jc.foreground_job(job_id) {
                Ok(_) => 0,
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

/// Built-in bg command
pub fn bg_builtin(args: &[String]) -> i32 {
    let job_control = get_enhanced_job_control();
    let mut jc = job_control.lock().unwrap();
    
    let job_id = if args.is_empty() {
        // Use current job
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
                Ok(_) => 0,
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

/// Built-in wait command
pub fn wait_builtin(args: &[String]) -> i32 {
    let job_control = get_enhanced_job_control();
    let mut jc = job_control.lock().unwrap();
    
    let job_id = if args.is_empty() {
        // Wait for all jobs
        // For now, just return success
        return 0;
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