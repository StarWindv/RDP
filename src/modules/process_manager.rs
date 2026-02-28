//! Cross-platform process management abstraction
//! 
//! This module provides a unified interface for process creation, management,
//! and inter-process communication that works on both Unix and Windows platforms.

use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

/// Process handle for cross-platform process management
#[derive(Debug)]
pub struct ProcessHandle {
    #[cfg(unix)]
    inner: Arc<Mutex<Child>>,
    #[cfg(windows)]
    inner: Arc<Mutex<Child>>,
    pub pid: u32,
}

impl ProcessHandle {
    /// Create a new process handle
    pub fn new(child: Child) -> Self {
        let pid = child.id();
        Self {
            inner: Arc::new(Mutex::new(child)),
            pid,
        }
    }

    /// Wait for the process to complete
    pub fn wait(&mut self) -> std::io::Result<i32> {
        let mut child = self.inner.lock().unwrap();
        let status = child.wait()?;
        Ok(status.code().unwrap_or(128))
    }

    /// Kill the process
    pub fn kill(&mut self) -> std::io::Result<()> {
        let mut child = self.inner.lock().unwrap();
        child.kill()
    }

    /// Get the process ID
    pub fn id(&self) -> u32 {
        self.pid
    }
}

/// Pipe for inter-process communication
pub struct Pipe {
    #[cfg(unix)]
    pub read_end: Option<os_pipe::PipeReader>,
    #[cfg(unix)]
    pub write_end: Option<os_pipe::PipeWriter>,
    #[cfg(windows)]
    pub read_end: Option<os_pipe::PipeReader>,
    #[cfg(windows)]
    pub write_end: Option<os_pipe::PipeWriter>,
}

impl Pipe {
    /// Create a new pipe
    pub fn new() -> std::io::Result<Self> {
        #[cfg(unix)]
        {
            use std::os::unix::io::{AsRawFd, FromRawFd};
            
            // Use os_pipe crate for Unix pipes
            let (read_end, write_end) = os_pipe::pipe()?;
            Ok(Self {
                read_end: Some(read_end),
                write_end: Some(write_end),
            })
        }
        
        #[cfg(windows)]
        {
            // Use os_pipe crate for Windows pipes
            let (read_end, write_end) = os_pipe::pipe()?;
            Ok(Self {
                read_end: Some(read_end),
                write_end: Some(write_end),
            })
        }
    }

    /// Get the read end as a std::process::Stdio
    pub fn read_end_as_stdio(&mut self) -> Stdio {
        #[cfg(unix)]
        {
            if let Some(read_end) = &mut self.read_end {
                // Convert to raw file descriptor
                use std::os::unix::io::{AsRawFd, FromRawFd};
                let fd = read_end.as_raw_fd();
                unsafe { Stdio::from_raw_fd(fd) }
            } else {
                Stdio::null()
            }
        }
        
        #[cfg(windows)]
        {
            if let Some(read_end) = &mut self.read_end {
                // Convert to raw handle
                use std::os::windows::io::{AsRawHandle, FromRawHandle};
                let handle = read_end.as_raw_handle();
                unsafe { Stdio::from_raw_handle(handle) }
            } else {
                Stdio::null()
            }
        }
    }

    /// Get the write end as a std::process::Stdio
    pub fn write_end_as_stdio(&mut self) -> Stdio {
        #[cfg(unix)]
        {
            if let Some(write_end) = &mut self.write_end {
                // Convert to raw file descriptor
                use std::os::unix::io::{AsRawFd, FromRawFd};
                let fd = write_end.as_raw_fd();
                unsafe { Stdio::from_raw_fd(fd) }
            } else {
                Stdio::null()
            }
        }
        
        #[cfg(windows)]
        {
            if let Some(write_end) = &mut self.write_end {
                // Convert to raw handle
                use std::os::windows::io::{AsRawHandle, FromRawHandle};
                let handle = write_end.as_raw_handle();
                unsafe { Stdio::from_raw_handle(handle) }
            } else {
                Stdio::null()
            }
        }
    }

    /// Close the read end
    pub fn close_read(&mut self) {
        self.read_end = None;
    }

    /// Close the write end
    pub fn close_write(&mut self) {
        self.write_end = None;
    }
}

/// Process manager for cross-platform process operations
pub struct ProcessManager;

impl ProcessManager {
    /// Create a new process manager
    pub fn new() -> Self {
        Self
    }

    /// Create a pipe for inter-process communication
    pub fn create_pipe(&self) -> std::io::Result<Pipe> {
        Pipe::new()
    }

    /// Spawn a new process with the given command and arguments
    pub fn spawn(&self, cmd: &str, args: &[String]) -> std::io::Result<ProcessHandle> {
        let mut command = Command::new(cmd);
        command.args(args);
        
        // Set up environment
        // TODO: Pass environment variables from shell
        
        let child = command.spawn()?;
        Ok(ProcessHandle::new(child))
    }

    /// Spawn a new process with stdin/stdout/stderr redirection
    pub fn spawn_with_io(
        &self,
        cmd: &str,
        args: &[String],
        stdin: Option<Stdio>,
        stdout: Option<Stdio>,
        stderr: Option<Stdio>,
    ) -> std::io::Result<ProcessHandle> {
        let mut command = Command::new(cmd);
        command.args(args);
        
        if let Some(stdin_io) = stdin {
            command.stdin(stdin_io);
        }
        
        if let Some(stdout_io) = stdout {
            command.stdout(stdout_io);
        }
        
        if let Some(stderr_io) = stderr {
            command.stderr(stderr_io);
        }
        
        let child = command.spawn()?;
        Ok(ProcessHandle::new(child))
    }

    /// Execute a command and wait for it to complete
    pub fn execute(&self, cmd: &str, args: &[String]) -> std::io::Result<i32> {
        let mut handle = self.spawn(cmd, args)?;
        handle.wait()
    }

    /// Fork a new process (Unix only, simulated on Windows)
    pub fn fork(&self) -> std::io::Result<u32> {
        #[cfg(unix)]
        {
            use nix::unistd::{fork, ForkResult};
            use nix::unistd::Pid;
            
            unsafe {
                match fork()? {
                    ForkResult::Parent { child } => {
                        Ok(child.as_raw() as u32)
                    }
                    ForkResult::Child => {
                        // Child process returns 0
                        std::process::exit(0);
                    }
                }
            }
        }
        
        #[cfg(windows)]
        {
            // Windows doesn't have fork, so we simulate it by creating a new process
            // that immediately exits. The parent gets a real PID.
            let handle = self.spawn("cmd", &["/c".to_string(), "exit".to_string(), "0".to_string()])?;
            Ok(handle.id())
        }
    }

    /// Duplicate a file descriptor (Unix) or handle (Windows)
    pub fn dup_fd(&self, old_fd: i32, new_fd: i32) -> std::io::Result<()> {
        #[cfg(unix)]
        {
            use nix::unistd::dup2;
            dup2(old_fd, new_fd)?;
            Ok(())
        }
        
        #[cfg(windows)]
        {
            // Windows doesn't have dup2 in the same way
            // We'll need to implement this differently
            // For now, just return success
            Ok(())
        }
    }

    /// Close a file descriptor
    pub fn close_fd(&self, fd: i32) -> std::io::Result<()> {
        #[cfg(unix)]
        {
            use nix::unistd::close;
            close(fd)?;
            Ok(())
        }
        
        #[cfg(windows)]
        {
            // Windows doesn't have close in the same way
            // For now, just return success
            Ok(())
        }
    }

    /// Create a pipeline of processes
    pub fn create_pipeline(&self, commands: &[(String, Vec<String>)]) -> std::io::Result<Vec<ProcessHandle>> {
        let mut handles = Vec::new();
        let mut previous_stdout: Option<Stdio> = None;
        
        for (i, (cmd, args)) in commands.iter().enumerate() {
            let is_first = i == 0;
            let is_last = i == commands.len() - 1;
            
            let mut stdin_io = None;
            let mut stdout_io = None;
            
            // Set up stdin from previous command's stdout
            if !is_first {
                stdin_io = previous_stdout.take();
            }
            
            // Set up stdout to pipe if not last command
            if !is_last {
                let mut pipe = self.create_pipe()?;
                stdout_io = Some(pipe.write_end_as_stdio());
                previous_stdout = Some(pipe.read_end_as_stdio());
            }
            
            // Spawn the process
            let handle = self.spawn_with_io(cmd, args, stdin_io, stdout_io, None)?;
            handles.push(handle);
        }
        
        Ok(handles)
    }
}

/// Builder for process creation with fluent API
pub struct ProcessBuilder {
    cmd: String,
    args: Vec<String>,
    stdin: Option<Stdio>,
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
    env: Vec<(String, String)>,
    current_dir: Option<String>,
}

impl ProcessBuilder {
    /// Create a new process builder
    pub fn new(cmd: &str) -> Self {
        Self {
            cmd: cmd.to_string(),
            args: Vec::new(),
            stdin: None,
            stdout: None,
            stderr: None,
            env: Vec::new(),
            current_dir: None,
        }
    }

    /// Add an argument
    pub fn arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    /// Add multiple arguments
    pub fn args(mut self, args: &[String]) -> Self {
        self.args.extend_from_slice(args);
        self
    }

    /// Set stdin
    pub fn stdin(mut self, stdin: Stdio) -> Self {
        self.stdin = Some(stdin);
        self
    }

    /// Set stdout
    pub fn stdout(mut self, stdout: Stdio) -> Self {
        self.stdout = Some(stdout);
        self
    }

    /// Set stderr
    pub fn stderr(mut self, stderr: Stdio) -> Self {
        self.stderr = Some(stderr);
        self
    }

    /// Set an environment variable
    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.env.push((key.to_string(), value.to_string()));
        self
    }

    /// Set the current directory
    pub fn current_dir(mut self, dir: &str) -> Self {
        self.current_dir = Some(dir.to_string());
        self
    }

    /// Spawn the process
    pub fn spawn(self) -> std::io::Result<ProcessHandle> {
        let mut command = Command::new(&self.cmd);
        command.args(&self.args);
        
        if let Some(stdin) = self.stdin {
            command.stdin(stdin);
        }
        
        if let Some(stdout) = self.stdout {
            command.stdout(stdout);
        }
        
        if let Some(stderr) = self.stderr {
            command.stderr(stderr);
        }
        
        for (key, value) in self.env {
            command.env(key, value);
        }
        
        if let Some(dir) = self.current_dir {
            command.current_dir(dir);
        }
        
        let child = command.spawn()?;
        Ok(ProcessHandle::new(child))
    }

    /// Execute the process and wait for completion
    pub fn execute(self) -> std::io::Result<i32> {
        let mut handle = self.spawn()?;
        handle.wait()
    }
}