//! Parameter expansion for POSIX Shell
//! Handles ${parameter}, ${parameter:-word}, ${parameter#pattern}, etc.

use crate::modules::env::ShellEnv;

/// Parameter expander for POSIX Shell
pub struct ParameterExpander {
    env: ShellEnv,
}

impl ParameterExpander {
    /// Create a new parameter expander
    pub fn new(env: ShellEnv) -> Self {
        Self { env }
    }

    /// Expand a parameter expression
    pub fn expand(&self, expression: &str) -> Result<String, String> {
        // Remove ${ and }
        let expr = expression.trim_start_matches("${").trim_end_matches('}');

        // Parse the expression
        self.parse_and_expand(expr)
    }

    /// Parse and expand a parameter expression
    fn parse_and_expand(&self, expr: &str) -> Result<String, String> {
        // Check for special operations
        if expr.starts_with('#') {
            // Length operation: ${#parameter}
            let param = &expr[1..];
            return self.expand_length(param);
        } else if expr.contains(":-") {
            // Use default value: ${parameter:-word}
            let parts: Vec<&str> = expr.split(":-").collect();
            if parts.len() == 2 {
                return self.expand_default(parts[0], parts[1], false);
            }
        } else if expr.contains(":=") {
            // Assign default value: ${parameter:=word}
            let parts: Vec<&str> = expr.split(":=").collect();
            if parts.len() == 2 {
                return self.expand_default(parts[0], parts[1], true);
            }
        } else if expr.contains(":?") {
            // Error if null or unset: ${parameter:?word}
            let parts: Vec<&str> = expr.split(":?").collect();
            if parts.len() == 2 {
                return self.expand_error_if_null(parts[0], parts[1]);
            }
        } else if expr.contains(":+") {
            // Use alternate value: ${parameter:+word}
            let parts: Vec<&str> = expr.split(":+").collect();
            if parts.len() == 2 {
                return self.expand_alternate(parts[0], parts[1]);
            }
        } else if expr.contains('#') {
            // Remove prefix pattern: ${parameter#pattern} or ${parameter##pattern}
            return self.expand_remove_prefix(expr);
        } else if expr.contains('%') {
            // Remove suffix pattern: ${parameter%pattern} or ${parameter%%pattern}
            return self.expand_remove_suffix(expr);
        } else if expr.contains('/') {
            // Pattern substitution: ${parameter/pattern/replacement}
            return self.expand_substitution(expr);
        } else if expr.contains(':') {
            // Substring expansion: ${parameter:offset} or ${parameter:offset:length}
            return self.expand_substring(expr);
        }

        // Simple parameter expansion: ${parameter}
        self.expand_simple(expr)
    }

    /// Expand simple parameter: ${parameter}
    fn expand_simple(&self, param: &str) -> Result<String, String> {
        match self.env.get_var(param) {
            Some(value) => Ok(value.clone()),
            None => Ok(String::new()), // Unset parameters expand to empty string
        }
    }

    /// Expand length operation: ${#parameter}
    fn expand_length(&self, param: &str) -> Result<String, String> {
        match self.env.get_var(param) {
            Some(value) => Ok(value.len().to_string()),
            None => Ok("0".to_string()), // Length of unset parameter is 0
        }
    }

    /// Expand default value: ${parameter:-word} or ${parameter:=word}
    fn expand_default(&self, param: &str, word: &str, assign: bool) -> Result<String, String> {
        match self.env.get_var(param) {
            Some(value) if !value.is_empty() => Ok(value.clone()),
            _ => {
                // Parameter is unset or null
                let expanded_word = self.expand_word(word)?;
                if assign {
                    // TODO: Assign the value to the parameter
                    // For now, just return the word
                }
                Ok(expanded_word)
            }
        }
    }

    /// Expand error if null: ${parameter:?word}
    fn expand_error_if_null(&self, param: &str, word: &str) -> Result<String, String> {
        match self.env.get_var(param) {
            Some(value) if !value.is_empty() => Ok(value.clone()),
            _ => {
                let message = if word.is_empty() {
                    format!("{}: parameter null or not set", param)
                } else {
                    self.expand_word(word)?
                };
                Err(message)
            }
        }
    }

    /// Expand alternate value: ${parameter:+word}
    fn expand_alternate(&self, param: &str, word: &str) -> Result<String, String> {
        match self.env.get_var(param) {
            Some(value) if !value.is_empty() => self.expand_word(word),
            _ => Ok(String::new()),
        }
    }

    /// Expand remove prefix: ${parameter#pattern} or ${parameter##pattern}
    fn expand_remove_prefix(&self, expr: &str) -> Result<String, String> {
        // TODO: Implement pattern matching for prefix removal
        Ok(expr.to_string())
    }

    /// Expand remove suffix: ${parameter%pattern} or ${parameter%%pattern}
    fn expand_remove_suffix(&self, expr: &str) -> Result<String, String> {
        // TODO: Implement pattern matching for suffix removal
        Ok(expr.to_string())
    }

    /// Expand pattern substitution: ${parameter/pattern/replacement}
    fn expand_substitution(&self, expr: &str) -> Result<String, String> {
        // TODO: Implement pattern substitution
        Ok(expr.to_string())
    }

    /// Expand substring: ${parameter:offset} or ${parameter:offset:length}
    fn expand_substring(&self, expr: &str) -> Result<String, String> {
        // TODO: Implement substring expansion
        Ok(expr.to_string())
    }

    /// Expand a word (might contain nested parameter expansions)
    fn expand_word(&self, word: &str) -> Result<String, String> {
        // TODO: Implement word expansion (recursive)
        Ok(word.to_string())
    }

    /// Check if a string is a valid parameter name
    pub fn is_valid_parameter_name(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        // First character must be letter or underscore
        let first_char = name.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() && first_char != '_' {
            return false;
        }

        // All characters must be alphanumeric or underscore
        name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_expander_creation() {
        let env = ShellEnv::new();
        let expander = ParameterExpander::new(env);

        // Just test that it can be created
        assert!(true);
    }

    #[test]
    fn test_valid_parameter_names() {
        assert!(ParameterExpander::is_valid_parameter_name("FOO"));
        assert!(ParameterExpander::is_valid_parameter_name("foo_bar"));
        assert!(ParameterExpander::is_valid_parameter_name("foo123"));
        assert!(ParameterExpander::is_valid_parameter_name("_foo"));

        assert!(!ParameterExpander::is_valid_parameter_name(""));
        assert!(!ParameterExpander::is_valid_parameter_name("123foo"));
        assert!(!ParameterExpander::is_valid_parameter_name("foo-bar"));
        assert!(!ParameterExpander::is_valid_parameter_name("foo bar"));
        assert!(!ParameterExpander::is_valid_parameter_name("foo.bar"));
    }
}
