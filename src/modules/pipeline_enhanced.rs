//! Enhanced pipeline execution using the new ProcessManager
//! 
//! This module provides proper pipeline execution with cross-platform support.

use crate::modules::process_manager::{ProcessManager, ProcessBuilder};
use std::process::Stdio;
use std::io::{Read, Write};
use tempfile::NamedTempFile;

/// Pipeline executor for POSIX Shell
pub struct PipelineExecutor {
    process_manager: ProcessManager,
}

impl PipelineExecutor {
    /// Create a new pipeline executor
    pub fn new() -> Self {
        Self {
            process_manager: ProcessManager::new(),
        }
    }

    /// Execute a pipeline: cmd1 | cmd2 | cmd3
    pub fn execute_pipeline(&self, commands: &[(String, Vec<String>)]) -> std::io::Result<i32> {
        if commands.is_empty() {
            return Ok(0);
        }

        if commands.len() == 1 {
            // Single command, no pipeline needed
            let (cmd, args) = &commands[0];
            return self.process_manager.execute(cmd, args);
        }

        // For multiple commands, create a pipeline
        let mut handles = self.process_manager.create_pipeline(commands)?;
        
        // Wait for all processes to complete
        // In a pipeline, we wait for the last process
        if let Some(mut last_handle) = handles.pop() {
            last_handle.wait()
        } else {
            Ok(0)
        }
    }

    /// Execute a command with input redirection: cmd < file
    pub fn execute_with_input_redirect(
        &self,
        cmd: &str,
        args: &[String],
        input_file: &str,
    ) -> std::io::Result<i32> {
        let input = Stdio::from(std::fs::File::open(input_file)?);
        ProcessBuilder::new(cmd)
            .args(args)
            .stdin(input)
            .execute()
    }

    /// Execute a command with output redirection: cmd > file
    pub fn execute_with_output_redirect(
        &self,
        cmd: &str,
        args: &[String],
        output_file: &str,
        append: bool,
    ) -> std::io::Result<i32> {
        let file = if append {
            std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open(output_file)?
        } else {
            std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(output_file)?
        };
        
        let output = Stdio::from(file);
        ProcessBuilder::new(cmd)
            .args(args)
            .stdout(output)
            .execute()
    }

    /// Execute a command with error redirection: cmd 2> file
    pub fn execute_with_error_redirect(
        &self,
        cmd: &str,
        args: &[String],
        error_file: &str,
        append: bool,
    ) -> std::io::Result<i32> {
        let file = if append {
            std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open(error_file)?
        } else {
            std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(error_file)?
        };
        
        let stderr = Stdio::from(file);
        ProcessBuilder::new(cmd)
            .args(args)
            .stderr(stderr)
            .execute()
    }

    /// Execute a command with here-document: cmd << EOF
    pub fn execute_with_here_doc(
        &self,
        cmd: &str,
        args: &[String],
        content: &str,
    ) -> std::io::Result<i32> {
        // Create a temporary file with the here-document content
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(content.as_bytes())?;
        
        // Seek to beginning
        temp_file.as_file_mut().sync_all()?;
        temp_file.as_file_mut().rewind()?;
        
        let input = Stdio::from(temp_file.reopen()?);
        ProcessBuilder::new(cmd)
            .args(args)
            .stdin(input)
            .execute()
    }

    /// Execute a command with combined stdout/stderr redirection: cmd >& file or cmd &> file
    pub fn execute_with_combined_redirect(
        &self,
        cmd: &str,
        args: &[String],
        output_file: &str,
        append: bool,
    ) -> std::io::Result<i32> {
        let file = if append {
            std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open(output_file)?
        } else {
            std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(output_file)?
        };
        
        let stdio = Stdio::from(file);
        ProcessBuilder::new(cmd)
            .args(args)
            .stdout(stdio)
            .stderr(Stdio::piped()) // For combined, we need to handle differently
            .execute()
    }

    /// Execute a command with file descriptor duplication: cmd n>&m or cmd n<&m
    pub fn execute_with_fd_duplication(
        &self,
        cmd: &str,
        args: &[String],
        fd_from: i32,
        fd_to: i32,
    ) -> std::io::Result<i32> {
        // This is complex and platform-specific
        // For now, we'll implement a simplified version
        // that only handles common cases like 2>&1
        
        if fd_from == 2 && fd_to == 1 {
            // stderr to stdout
            ProcessBuilder::new(cmd)
                .args(args)
                .stderr(Stdio::inherit()) // Inherit from stdout
                .execute()
        } else if fd_from == 1 && fd_to == 2 {
            // stdout to stderr
            ProcessBuilder::new(cmd)
                .args(args)
                .stdout(Stdio::inherit()) // Inherit from stderr
                .execute()
        } else {
            // Unsupported fd duplication
            eprintln!("Warning: FD duplication {fd_from}>&{fd_to} not fully supported");
            self.process_manager.execute(cmd, args)
        }
    }

    /// Execute a command in background: cmd &
    pub fn execute_background(
        &self,
        cmd: &str,
        args: &[String],
    ) -> std::io::Result<u32> {
        let handle = ProcessBuilder::new(cmd)
            .args(args)
            .spawn()?;
        Ok(handle.id())
    }
}

/// Helper function to parse redirection specifications
pub fn parse_redirection(redirect_str: &str) -> Option<(i32, String, bool, bool)> {
    // Parse redirections like: > file, >> file, 2> file, 2>> file, &> file, &>> file
    let mut chars = redirect_str.chars().peekable();
    let mut fd = 1; // Default to stdout
    let mut append = false;
    let mut is_stderr = false;
    let mut is_combined = false;
    
    // Parse file descriptor
    while let Some(&c) = chars.peek() {
        if c.is_digit(10) {
            fd = c.to_digit(10).unwrap() as i32;
            chars.next();
        } else {
            break;
        }
    }
    
    // Parse operator
    while let Some(&c) = chars.peek() {
        match c {
            '>' => {
                chars.next();
                if let Some(&'&') = chars.peek() {
                    // >& (duplicate or combined)
                    chars.next();
                    is_combined = true;
                } else if let Some(&'>') = chars.peek() {
                    // >> (append)
                    chars.next();
                    append = true;
                }
                break;
            }
            '<' => {
                chars.next();
                if let Some(&'&') = chars.peek() {
                    // <& (duplicate input)
                    chars.next();
                }
                break;
            }
            '&' => {
                chars.next();
                if let Some(&'>') = chars.peek() {
                    // &> (combined output)
                    chars.next();
                    is_combined = true;
                    if let Some(&'>') = chars.peek() {
                        // &>> (combined append)
                        chars.next();
                        append = true;
                    }
                }
                break;
            }
            '2' => {
                chars.next();
                if let Some(&'>') = chars.peek() {
                    // 2> (stderr)
                    chars.next();
                    is_stderr = true;
                    if let Some(&'>') = chars.peek() {
                        // 2>> (stderr append)
                        chars.next();
                        append = true;
                    }
                }
                break;
            }
            _ => {
                // Unknown operator
                break;
            }
        }
    }
    
    // Parse filename
    let filename: String = chars.collect();
    if filename.is_empty() {
        None
    } else {
        Some((fd, filename.trim().to_string(), append, is_stderr || is_combined))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_redirection() {
        // Test various redirection formats
        assert_eq!(
            parse_redirection("> output.txt"),
            Some((1, "output.txt".to_string(), false, false))
        );
        
        assert_eq!(
            parse_redirection(">> output.txt"),
            Some((1, "output.txt".to_string(), true, false))
        );
        
        assert_eq!(
            parse_redirection("2> error.txt"),
            Some((1, "error.txt".to_string(), false, true))
        );
        
        assert_eq!(
            parse_redirection("2>> error.txt"),
            Some((1, "error.txt".to_string(), true, true))
        );
        
        assert_eq!(
            parse_redirection("&> both.txt"),
            Some((1, "both.txt".to_string(), false, true))
        );
        
        assert_eq!(
            parse_redirection("&>> both.txt"),
            Some((1, "both.txt".to_string(), true, true))
        );
    }

    #[test]
    fn test_simple_pipeline() -> std::io::Result<()> {
        let executor = PipelineExecutor::new();
        
        // Test: echo hello | cat
        let commands = vec![
            ("echo".to_string(), vec!["hello".to_string()]),
            ("cat".to_string(), vec![]),
        ];
        
        let result = executor.execute_pipeline(&commands)?;
        assert_eq!(result, 0);
        Ok(())
    }

    #[test]
    fn test_input_redirect() -> std::io::Result<()> {
        let executor = PipelineExecutor::new();
        
        // Create a temporary file
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(b"test content\n")?;
        let temp_path = temp_file.path().to_str().unwrap().to_string();
        
        // Test: cat < file
        let result = executor.execute_with_input_redirect(
            "cat",
            &[],
            &temp_path,
        )?;
        
        assert_eq!(result, 0);
        Ok(())
    }

    #[test]
    fn test_output_redirect() -> std::io::Result<()> {
        let executor = PipelineExecutor::new();
        
        // Create a temporary file path
        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path().to_str().unwrap().to_string();
        
        // Test: echo test > file
        let result = executor.execute_with_output_redirect(
            "echo",
            &["test".to_string()],
            &temp_path,
            false,
        )?;
        
        assert_eq!(result, 0);
        
        // Verify file content
        let content = std::fs::read_to_string(&temp_path)?;
        assert!(content.contains("test"));
        Ok(())
    }
}