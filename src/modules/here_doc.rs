//! Here-document handling for POSIX Shell
//! Implements << and <<- redirections

use std::io::{self, Write, Read, Seek};
use std::process::Stdio;
use std::fs::File;
use tempfile::NamedTempFile;

/// Here-document processor
pub struct HereDocProcessor;

impl HereDocProcessor {
    /// Create a new here-document processor
    pub fn new() -> Self {
        Self
    }
    
    /// Process a here-document
    pub fn process_here_doc(
        &self,
        _delimiter: &str,
        strip_tabs: bool,
        content: &str,
    ) -> Result<Stdio, String> {
        // Create a temporary file for the here-document content
        let mut temp_file = NamedTempFile::new()
            .map_err(|e| format!("Failed to create temporary file: {}", e))?;
        
        // Process the content
        let processed_content = self.process_content(content, strip_tabs);
        
        // Write to temporary file
        temp_file.write_all(processed_content.as_bytes())
            .map_err(|e| format!("Failed to write here-document: {}", e))?;
        
        // Seek to beginning
        temp_file.as_file_mut().seek(io::SeekFrom::Start(0))
            .map_err(|e| format!("Failed to seek temporary file: {}", e))?;
        
        // Convert to Stdio
        Ok(Stdio::from(temp_file.into_file()))
    }
    
    /// Process here-document content
    fn process_content(&self, content: &str, strip_tabs: bool) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = String::new();
        
        for line in lines {
            if strip_tabs {
                // Remove leading tabs
                let stripped_line = line.trim_start_matches('\t');
                result.push_str(stripped_line);
            } else {
                result.push_str(line);
            }
            result.push('\n');
        }
        
        result
    }
    
    /// Read here-document from input
    pub fn read_here_doc(&self, _delimiter: &str, strip_tabs: bool) -> Result<String, String> {
        let mut content = String::new();
        
        println!("Reading here-document (delimiter: {}, strip tabs: {})", delimiter, strip_tabs);
        println!("Type lines. End with '{}' on a line by itself.", delimiter);
        
        loop {
            let mut line = String::new();
            io::stdin().read_line(&mut line)
                .map_err(|e| format!("Failed to read here-document line: {}", e))?;
            
            // Check for delimiter
            let trimmed_line = if strip_tabs {
                line.trim_start_matches('\t').trim_end()
            } else {
                line.trim_end()
            };
            
            if trimmed_line == delimiter {
                break;
            }
            
            content.push_str(&line);
        }
        
        Ok(content)
    }
    
    /// Create a here-document from string content
    pub fn create_here_doc(&self, content: &str, strip_tabs: bool) -> Result<Stdio, String> {
        self.process_here_doc("EOF", strip_tabs, content)
    }
}

/// Here-document redirection result
pub struct HereDocResult {
    /// File descriptor for the here-document
    pub fd: i32,
    /// Stdio for the redirection
    pub stdio: Stdio,
}

impl HereDocResult {
    /// Create a new here-document result
    pub fn new(fd: i32, stdio: Stdio) -> Self {
        Self { fd, stdio }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_here_doc_processor_creation() {
        let processor = HereDocProcessor::new();
        
        // Just test that it can be created
        assert!(true);
    }
    
    #[test]
    fn test_process_content() {
        let processor = HereDocProcessor::new();
        
        // Test without tab stripping
        let content = "line1\n\tline2\nline3";
        let result = processor.process_content(content, false);
        assert_eq!(result, "line1\n\tline2\nline3\n");
        
        // Test with tab stripping
        let result = processor.process_content(content, true);
        assert_eq!(result, "line1\nline2\nline3\n");
    }
    
    #[test]
    fn test_create_here_doc() {
        let processor = HereDocProcessor::new();
        
        // Test creating a here-document
        let content = "Hello\nWorld\n";
        let result = processor.create_here_doc(content, false);
        
        // Should succeed
        assert!(result.is_ok());
    }
}