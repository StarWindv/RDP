/// Read a word in arithmetic expansion (variable name, function name, etc.)
    fn read_arithmetic_word(&mut self) -> String {
        let mut word = String::new();
        
        while let Some(&c) = self.chars.peek() {
            // In arithmetic, words can contain alphanumerics and underscores
            if c.is_alphanumeric() || c == '_' {
                word.push(c);
                self.consume_char();
            } else {
                break;
            }
        }
        
        word
    }
    
    /// Check if character is a special shell character
    fn is_special_character(&self, c: char) -> bool {
        match c {
            ';' | '&' | '|' | '<' | '>' | '(' | ')' | '{' | '}' | 
            '`' | '$' | '\'' | '"' | '#' | '\n' | '=' |
            '*' | '?' | '[' | ']' | '!' | '@' | '+' => true,
            _ => false,
        }
    }
    
    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.consume_char();
            } else {
                break;
            }
        }
    }
    
    /// Consume the current character
    fn consume_char(&mut self) -> Option<char> {
        let c = self.chars.next();
        if let Some(_) = c {
            self.column += 1;
        }
        c
    }
    
    /// Check and consume a character if it matches
    fn match_char(&mut self, expected: char) -> bool {
        if let Some(&c) = self.chars.peek() {
            if c == expected {
                self.consume_char();
                return true;
            }
        }
        false
    }
    
    /// Get current position in input
    fn current_position(&self) -> usize {
        // Calculate position by subtracting remaining chars length from total length
        let remaining_str: String = self.chars.clone().collect();
        self.input.len() - remaining_str.len()
    }
}

impl<'a> Iterator for EnhancedLexer<'a> {
    type Item = Token;
    
    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        match token.token_type {
            TokenType::Eof => None,
            _ => Some(token),
        }
    }
}