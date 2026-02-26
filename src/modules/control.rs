//! Control structures for POSIX Shell
//! Handles if, while, until, for, case, etc.

use crate::modules::ast::{AstNode, CaseClause};
use crate::modules::env::ShellEnv;
use crate::modules::builtins::Builtins;
use crate::modules::variables::get_variable_system;

/// Control structure executor
pub struct ControlExecutor {
    env: ShellEnv,
    builtins: Builtins,
}

impl ControlExecutor {
    /// Create a new control executor
    pub fn new(env: ShellEnv, builtins: Builtins) -> Self {
        Self { env, builtins }
    }
    
    /// Execute an if statement
    pub fn execute_if(
        &mut self,
        condition: &AstNode,
        then_branch: &[AstNode],
        else_branch: Option<&[AstNode]>,
        elif_branches: &[(AstNode, Vec<AstNode>)],
    ) -> Result<i32, String> {
        // Execute condition
        let condition_status = self.execute_condition(condition)?;
        
        if condition_status == 0 {
            // Condition true, execute then branch
            self.execute_commands(then_branch)
        } else {
            // Condition false, check elif branches
            let mut status = condition_status;
            for (elif_condition, elif_body) in elif_branches {
                status = self.execute_condition(elif_condition)?;
                if status == 0 {
                    return self.execute_commands(elif_body);
                }
            }
            
            // All conditions false, execute else branch if present
            if let Some(else_body) = else_branch {
                self.execute_commands(else_body)
            } else {
                Ok(0) // No else branch, return success
            }
        }
    }
    
    /// Execute a while loop
    pub fn execute_while(
        &mut self,
        condition: &AstNode,
        body: &[AstNode],
    ) -> Result<i32, String> {
        let mut last_status = 0;
        
        loop {
            // Execute condition
            let condition_status = self.execute_condition(condition)?;
            
            // If condition is false (non-zero), break
            if condition_status != 0 {
                break;
            }
            
            // Execute body
            last_status = self.execute_commands(body)?;
            
            // Check for break/continue (would be handled by special control flow)
            // For now, just continue loop
        }
        
        Ok(last_status)
    }
    
    /// Execute an until loop
    pub fn execute_until(
        &mut self,
        condition: &AstNode,
        body: &[AstNode],
    ) -> Result<i32, String> {
        let mut last_status = 0;
        
        loop {
            // Execute condition
            let condition_status = self.execute_condition(condition)?;
            
            // If condition is true (zero), break
            if condition_status == 0 {
                break;
            }
            
            // Execute body
            last_status = self.execute_commands(body)?;
            
            // Check for break/continue (would be handled by special control flow)
            // For now, just continue loop
        }
        
        Ok(last_status)
    }
    
    /// Execute a for loop
    pub fn execute_for(
        &mut self,
        variable: &str,
        items: &[String],
        body: &[AstNode],
    ) -> Result<i32, String> {
        let mut last_status = 0;
        let mut vs = get_variable_system();
        
        for item in items {
            // Set loop variable
            if let Err(e) = vs.set(variable.to_string(), item.clone()) {
                return Err(format!("Failed to set loop variable {}: {}", variable, e));
            }
            
            // Execute body
            last_status = self.execute_commands(body)?;
            
            // Check for break/continue (would be handled by special control flow)
            // For now, just continue loop
        }
        
        Ok(last_status)
    }
    
    /// Execute a case statement
    pub fn execute_case(
        &mut self,
        word: &str,
        cases: &[CaseClause],
    ) -> Result<i32, String> {
        // Find matching case
        for case in cases {
            for pattern in &case.patterns {
                if self.pattern_matches(word, pattern) {
                    return self.execute_commands(&case.body.iter().map(|c| c.clone()).collect::<Vec<_>>());
                }
            }
        }
        
        // No matching case
        Ok(0)
    }
    
    /// Execute a condition (used in if, while, until)
    fn execute_condition(&mut self, condition: &AstNode) -> Result<i32, String> {
        // For now, just execute the command and return its exit status
        // In a full implementation, we would handle test expressions ([ ... ])
        self.execute_command(condition)
    }
    
    /// Execute a single command
    fn execute_command(&mut self, command: &AstNode) -> Result<i32, String> {
        // TODO: Implement command execution
        // For now, just return success
        Ok(0)
    }
    
    /// Execute multiple commands
    fn execute_commands(&mut self, commands: &[AstNode]) -> Result<i32, String> {
        let mut last_status = 0;
        
        for command in commands {
            last_status = self.execute_command(command)?;
        }
        
        Ok(last_status)
    }
    
    /// Check if a word matches a pattern (simple globbing)
    fn pattern_matches(&self, word: &str, pattern: &str) -> bool {
        // Simple pattern matching for now
        // Supports * and ? wildcards
        let word_chars: Vec<char> = word.chars().collect();
        let pattern_chars: Vec<char> = pattern.chars().collect();
        
        let mut i = 0; // word index
        let mut j = 0; // pattern index
        
        while i < word_chars.len() && j < pattern_chars.len() {
            match pattern_chars[j] {
                '*' => {
                    // Try to match zero or more characters
                    if j + 1 == pattern_chars.len() {
                        return true; // * at end matches everything
                    }
                    
                    // Try to match the rest of the pattern
                    for k in i..=word_chars.len() {
                        if self.pattern_matches(&word[k..], &pattern[j+1..]) {
                            return true;
                        }
                    }
                    return false;
                }
                '?' => {
                    // Match any single character
                    i += 1;
                    j += 1;
                }
                c => {
                    // Match exact character
                    if i < word_chars.len() && word_chars[i] == c {
                        i += 1;
                        j += 1;
                    } else {
                        return false;
                    }
                }
            }
        }
        
        // Check if we've consumed all of both strings
        i == word_chars.len() && j == pattern_chars.len()
    }
    
    /// Evaluate a test expression (for [ ... ] or test command)
    pub fn evaluate_test(&self, args: &[String]) -> Result<bool, String> {
        // Simple test expression evaluation
        // For now, just check if first argument is non-empty
        Ok(!args.is_empty())
    }
    
    /// Handle break statement
    pub fn handle_break(&self, levels: Option<i32>) -> Result<(), String> {
        // TODO: Implement break handling
        // This would need to be integrated with loop execution
        Ok(())
    }
    
    /// Handle continue statement
    pub fn handle_continue(&self, levels: Option<i32>) -> Result<(), String> {
        // TODO: Implement continue handling
        // This would need to be integrated with loop execution
        Ok(())
    }
}

/// Control flow result
pub enum ControlFlow {
    Continue,
    Break(Option<i32>), // Optional number of levels to break
    Return(i32),        // Return with exit status
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_control_executor_creation() {
        let env = ShellEnv::new();
        let builtins = Builtins::new();
        let executor = ControlExecutor::new(env, builtins);
        
        // Just test that it can be created
        assert!(true);
    }
    
    #[test]
    fn test_pattern_matching() {
        let env = ShellEnv::new();
        let builtins = Builtins::new();
        let executor = ControlExecutor::new(env, builtins);
        
        // Test exact match
        assert!(executor.pattern_matches("hello", "hello"));
        
        // Test * wildcard
        assert!(executor.pattern_matches("hello", "h*"));
        assert!(executor.pattern_matches("hello", "*o"));
        assert!(executor.pattern_matches("hello", "h*o"));
        assert!(executor.pattern_matches("hello", "*"));
        
        // Test ? wildcard
        assert!(executor.pattern_matches("hello", "h?llo"));
        assert!(executor.pattern_matches("hello", "?????"));
        
        // Test no match
        assert!(!executor.pattern_matches("hello", "world"));
        assert!(!executor.pattern_matches("hello", "h?ll"));
        assert!(!executor.pattern_matches("hello", "h*ll"));
    }
}