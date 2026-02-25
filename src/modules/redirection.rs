//! Redirection handling for POSIX Shell
//! Handles >, <, >>, >&, <&, etc.

use std::fs::{File, OpenOptions};
use std::io;
use std::process::Stdio;

use crate::modules::ast::RedirectType;

/// Redirection handler for POSIX Shell
pub struct RedirectionHandler;

impl RedirectionHandler {
    /// Create a new redirection handler
    pub fn new() -> Self {
        Self
    }
    
    /// Apply a redirection to a file descriptor
    pub fn apply_redirection(
        &self,
        redirect_type: RedirectType,
        target: &str,
        fd: Option<i32>,
    ) -> Result<RedirectionResult, String> {
        let fd = fd.unwrap_or(match redirect_type {
            RedirectType::Input | RedirectType::DupInput | RedirectType::ReadWrite => 0, // stdin
            _ => 1, // stdout
        });
        
        match redirect_type {
            RedirectType::Input => self.redirect_input(target, fd),
            RedirectType::Output => self.redirect_output(target, fd, false),
            RedirectType::Append => self.redirect_output(target, fd, true),
            RedirectType::HereDoc => self.redirect_here_doc(target, fd),
            RedirectType::HereDocStrip => self.redirect_here_doc_strip(target, fd),
            RedirectType::DupInput => self.dup_input(target, fd),
            RedirectType::DupOutput => self.dup_output(target, fd),
            RedirectType::ReadWrite => self.redirect_read_write(target, fd),
            RedirectType::Clobber => self.redirect_clobber(target, fd),
        }
    }
    
    /// Redirect input from a file: < file
    fn redirect_input(&self, target: &str, fd: i32) -> Result<RedirectionResult, String> {
        let file = File::open(target)
            .map_err(|e| format!("Cannot open {} for reading: {}", target, e))?;
        
        Ok(RedirectionResult {
            fd,
            stdio: Stdio::from(file),
            close_original: true,
        })
    }
    
    /// Redirect output to a file: > file or >> file
    fn redirect_output(&self, target: &str, fd: i32, append: bool) -> Result<RedirectionResult, String> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(!append)
            .append(append)
            .open(target)
            .map_err(|e| format!("Cannot open {} for writing: {}", target, e))?;
        
        Ok(RedirectionResult {
            fd,
            stdio: Stdio::from(file),
            close_original: true,
        })
    }
    
    /// Redirect here-document: << delimiter
    fn redirect_here_doc(&self, _delimiter: &str, fd: i32) -> Result<RedirectionResult, String> {
        // TODO: Implement here-document redirection
        // For now, just return a placeholder
        Ok(RedirectionResult {
            fd,
            stdio: Stdio::null(),
            close_original: true,
        })
    }
    
    /// Redirect here-document with tab stripping: <<- delimiter
    fn redirect_here_doc_strip(&self, _delimiter: &str, fd: i32) -> Result<RedirectionResult, String> {
        // TODO: Implement here-document with tab stripping
        // For now, just return a placeholder
        Ok(RedirectionResult {
            fd,
            stdio: Stdio::null(),
            close_original: true,
        })
    }
    
    /// Duplicate input file descriptor: <& fd
    fn dup_input(&self, target: &str, fd: i32) -> Result<RedirectionResult, String> {
        self.dup_file_descriptor(target, fd, true)
    }
    
    /// Duplicate output file descriptor: >& fd
    fn dup_output(&self, target: &str, fd: i32) -> Result<RedirectionResult, String> {
        self.dup_file_descriptor(target, fd, false)
    }
    
    /// Duplicate a file descriptor
    fn dup_file_descriptor(&self, target: &str, fd: i32, is_input: bool) -> Result<RedirectionResult, String> {
        if target == "-" {
            // Close the file descriptor
            return Ok(RedirectionResult {
                fd,
                stdio: Stdio::null(),
                close_original: true,
            });
        }
        
        let target_fd: i32 = target.parse()
            .map_err(|_| format!("Invalid file descriptor: {}", target))?;
        
        // TODO: Implement actual file descriptor duplication
        // For now, just return a placeholder
        Ok(RedirectionResult {
            fd,
            stdio: if is_input { Stdio::null() } else { Stdio::inherit() },
            close_original: false,
        })
    }
    
    /// Open file for reading and writing: <> file
    fn redirect_read_write(&self, target: &str, fd: i32) -> Result<RedirectionResult, String> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(target)
            .map_err(|e| format!("Cannot open {} for reading/writing: {}", target, e))?;
        
        Ok(RedirectionResult {
            fd,
            stdio: Stdio::from(file),
            close_original: true,
        })
    }
    
    /// Force output redirection even if noclobber is set: >| file
    fn redirect_clobber(&self, target: &str, fd: i32) -> Result<RedirectionResult, String> {
        self.redirect_output(target, fd, false)
    }
    
    /// Parse a redirection target (could be file descriptor or filename)
    fn parse_target(&self, target: &str) -> Result<RedirectionTarget, String> {
        if let Ok(fd) = target.parse::<i32>() {
            Ok(RedirectionTarget::FileDescriptor(fd))
        } else {
            Ok(RedirectionTarget::FileName(target.to_string()))
        }
    }
    
    /// Validate file descriptor
    fn validate_fd(&self, fd: i32) -> Result<(), String> {
        if fd < 0 || fd > 1023 { // Reasonable limit
            return Err(format!("Invalid file descriptor: {}", fd));
        }
        Ok(())
    }
}

/// Result of applying a redirection
pub struct RedirectionResult {
    /// File descriptor to redirect
    pub fd: i32,
    /// Stdio to use for the redirection
    pub stdio: Stdio,
    /// Whether to close the original file descriptor
    pub close_original: bool,
}

/// Redirection target type
pub enum RedirectionTarget {
    FileDescriptor(i32),
    FileName(String),
}

/// Redirection error
#[derive(Debug)]
pub enum RedirectionError {
    IoError(io::Error),
    InvalidDescriptor(String),
    InvalidTarget(String),
}

impl From<io::Error> for RedirectionError {
    fn from(err: io::Error) -> Self {
        RedirectionError::IoError(err)
    }
}

impl std::fmt::Display for RedirectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RedirectionError::IoError(e) => write!(f, "IO error: {}", e),
            RedirectionError::InvalidDescriptor(msg) => write!(f, "Invalid descriptor: {}", msg),
            RedirectionError::InvalidTarget(msg) => write!(f, "Invalid target: {}", msg),
        }
    }
}

impl std::error::Error for RedirectionError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_redirection_handler_creation() {
        let handler = RedirectionHandler::new();
        
        // Just test that it can be created
        assert!(true);
    }
    
    #[test]
    fn test_parse_target() {
        let handler = RedirectionHandler::new();
        
        // Test file descriptor
        let result = handler.parse_target("2").unwrap();
        match result {
            RedirectionTarget::FileDescriptor(fd) => assert_eq!(fd, 2),
            _ => panic!("Expected FileDescriptor"),
        }
        
        // Test filename
        let result = handler.parse_target("file.txt").unwrap();
        match result {
            RedirectionTarget::FileName(name) => assert_eq!(name, "file.txt"),
            _ => panic!("Expected FileName"),
        }
        
        // Test invalid file descriptor (still parses as string)
        let result = handler.parse_target("not_a_number").unwrap();
        match result {
            RedirectionTarget::FileName(name) => assert_eq!(name, "not_a_number"),
            _ => panic!("Expected FileName"),
        }
    }
    
    #[test]
    fn test_validate_fd() {
        let handler = RedirectionHandler::new();
        
        // Valid file descriptors
        assert!(handler.validate_fd(0).is_ok());
        assert!(handler.validate_fd(1).is_ok());
        assert!(handler.validate_fd(2).is_ok());
        assert!(handler.validate_fd(10).is_ok());
        assert!(handler.validate_fd(1023).is_ok());
        
        // Invalid file descriptors
        assert!(handler.validate_fd(-1).is_err());
        assert!(handler.validate_fd(1024).is_err());
    }
}