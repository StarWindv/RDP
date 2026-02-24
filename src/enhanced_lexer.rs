//! Enhanced Lexer for POSIX Shell with full tokenization support
//! Based on POSIX.1-2017 Shell Command Language

use std::iter::Peekable;
use std::str::Chars;

use crate::tokens::{Token, TokenType, is_valid_var_name, is_reserved_word, reserved_word_token_type};

/// Enhanced Lexer for tokenizing POSIX Shell scripts with full support
pub struct EnhancedLexer<'a> {
    input: &'a str,
    chars: Peekable<Chars<'a>>,
    line: usize,
    column: usize,
    current_line_start: usize,
    here_doc_delimiters: Vec<String>, // Stack of here-document delimiters
    in_here_doc: bool,                // Currently reading a here-document
    here_doc_content: Option<String>, // Accumulated here-doc content
}

impl<'a> EnhancedLexer<'a> {
    /// Create a new enhanced lexer for the given input
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().peekable(),
            line: 1,
            column: 1,
            current_line_start: 0,
            here_doc_delimiters: Vec::new(),
            in_here_doc: false,
            here_doc_content: None,
        }
    }
    
    /// Get the next token from the input
    pub fn next_token(&mut self) -> Token {
        // Handle here-document content if we're in a here-doc
        if self.in_here_doc {
            return self.read_here_doc_content();
        }
        
        self.skip_whitespace();
        
        if let Some(&c) = self.chars.peek() {
            let start_line = self.line;
            let start_column = self.column;
            
            match c {
                // ============================================
                // Control operators
                // ============================================
                '\n' => {
                    self.consume_char();
                    self.line += 1;
                    self.column = 1;
                    self.current_line_start = self.current_position();
                    Token::new(TokenType::Newline, "\n".to_string(), start_line, start_column)
                }
                
                ';' => {
                    self.consume_char();
                    // Check for ;;
                    if self.match_char(';') {
                        return Token::new(TokenType::DSemi, ";;".to_string(), start_line, start_column);
                    }
                    Token::new(TokenType::Semicolon, ";".to_string(), start_line, start_column)
                }
                
                '&' => {
                    self.consume_char();
                    // Check for &&
                    if self.match_char('&') {
                        return Token::new(TokenType::AndIf, "&&".to_string(), start_line, start_column);
                    }
                    Token::new(TokenType::Ampersand, "&".to_string(), start_line, start_column)
                }
                
                '|' => {
                    self.consume_char();
                    // Check for ||
                    if self.match_char('|') {
                        return Token::new(TokenType::OrIf, "||".to_string(), start_line, start_column);
                    }
                    // Check for >|
                    if self.match_char('>') {
                        return Token::new(TokenType::Clobber, ">|".to_string(), start_line, start_column);
                    }
                    Token::new(TokenType::Pipe, "|".to_string(), start_line, start_column)
                }
                
                // ============================================
                // Redirection operators
                // ============================================
                '<' => {
                    self.consume_char();
                    // Check for <<, <<-, <&, <>
                    if self.match_char('<') {
                        // Check for <<-
                        if self.match_char('-') {
                            return Token::new(TokenType::DLessDash, "<<-".to_string(), start_line, start_column);
                        }
                        return Token::new(TokenType::DLess, "<<".to_string(), start_line, start_column);
                    } else if self.match_char('&') {
                        return Token::new(TokenType::LessAnd, "<&".to_string(), start_line, start_column);
                    } else if self.match_char('>') {
                        return Token::new(TokenType::LessGreat, "<>".to_string(), start_line, start_column);
                    }
                    Token::new(TokenType::Less, "<".to_string(), start_line, start_column)
                }
                
                '>' => {
                    self.consume_char();
                    // Check for >>, >&
                    if self.match_char('>') {
                        return Token::new(TokenType::DGreat, ">>".to_string(), start_line, start_column);
                    } else if self.match_char('&') {
                        return Token::new(TokenType::GreatAnd, ">&".to_string(), start_line, start_column);
                    }
                    Token::new(TokenType::Great, ">".to_string(), start_line, start_column)
                }
                
                // ============================================
                // Grouping operators
                // ============================================
                '(' => {
                    self.consume_char();
                    Token::new(TokenType::LeftParen, "(".to_string(), start_line, start_column)
                }
                
                ')' => {
                    self.consume_char();
                    Token::new(TokenType::RightParen, ")".to_string(), start_line, start_column)
                }
                
                '{' => {
                    self.consume_char();
                    Token::new(TokenType::LeftBrace, "{".to_string(), start_line, start_column)
                }
                
                '}' => {
                    self.consume_char();
                    Token::new(TokenType::RightBrace, "}".to_string(), start_line, start_column)
                }
                
                // ============================================
                // Parameter expansion and command substitution
                // ============================================
                '$' => {
                    self.consume_char();
                    // Check for $(, ${, $((, $[
                    if self.match_char('(') {
                        // Check for $(( (arithmetic expansion)
                        if self.match_char('(') {
                            return Token::new(TokenType::DollarDLeftParen, "$((".to_string(), start_line, start_column);
                        }
                        return Token::new(TokenType::DollarLeftParen, "$(".to_string(), start_line, start_column);
                    } else if self.match_char('{') {
                        return Token::new(TokenType::DollarLeftBrace, "${".to_string(), start_line, start_column);
                    } else if self.match_char('[') {
                        return Token::new(TokenType::Dollar, "$[".to_string(), start_line, start_column);
                    }
                    
                    Token::new(TokenType::Dollar, "$".to_string(), start_line, start_column)
                }
                
                '`' => {
                    self.consume_char();
                    Token::new(TokenType::Backtick, "`".to_string(), start_line, start_column)
                }
                
                // ============================================
                // Quotes and comments
                // ============================================
                '\'' => {
                    self.consume_char();
                    let content = self.read_quoted_string('\'');
                    Token::new(TokenType::Word(content.clone()), format!("'{}'", content), start_line, start_column)
                }
                
                '"' => {
                    self.consume_char();
                    let content = self.read_quoted_string('"');
                    Token::new(TokenType::Word(content.clone()), format!("\"{}\"", content), start_line, start_column)
                }
                
                '#' => {
                    // Comment, skip to end of line
                    let comment = self.read_comment();
                    Token::new(TokenType::Word(comment.clone()), format!("#{}", comment), start_line, start_column)
                }
                
                // ============================================
                // Assignment and pattern matching
                // ============================================
                '=' => {
                    self.consume_char();
                    Token::new(TokenType::Word("=".to_string()), "=".to_string(), start_line, start_column)
                }
                
                '*' => {
                    self.consume_char();
                    Token::new(TokenType::Star, "*".to_string(), start_line, start_column)
                }
                
                '?' => {
                    self.consume_char();
                    Token::new(TokenType::Question, "?".to_string(), start_line, start_column)
                }
                
                '[' => {
                    self.consume_char();
                    Token::new(TokenType::LeftBracket, "[".to_string(), start_line, start_column)
                }
                
                ']' => {
                    self.consume_char();
                    Token::new(TokenType::RightBracket, "]".to_string(), start_line, start_column)
                }
                
                '!' => {
                    self.consume_char();
                    Token::new(TokenType::Exclamation, "!".to_string(), start_line, start_column)
                }
                
                '@' => {
                    self.consume_char();
                    Token::new(TokenType::At, "@".to_string(), start_line, start_column)
                }
                
                '+' => {
                    self.consume_char();
                    Token::new(TokenType::Plus, "+".to_string(), start_line, start_column)
                }
                
                // ============================================
                // Words and names
                // ============================================
                _ => {
                    if c.is_alphabetic() || c == '_' {
                        // Could be a reserved word, variable name, or regular word
                        let word = self.read_word();
                        
                        // Check if it's a reserved word
                        if is_reserved_word(&word) {
                            if let Some(token_type) = reserved_word_token_type(&word) {
                                return Token::new(token_type, word, start_line, start_column);
                            }
                        }
                        
                        // Check if it's followed by = (assignment)
                        self.skip_whitespace();
                        if let Some(&next) = self.chars.peek() {
                            if next == '=' {
                                self.consume_char();
                                // Read the value
                                self.skip_whitespace();
                                let value = self.read_assignment_value();
                                let assignment = format!("{}={}", word, value);
                                return Token::new(TokenType::AssignmentWord(assignment.clone()), assignment, start_line, start_column);
                            }
                        }
                        
                        // Regular word
                        Token::new(TokenType::Word(word.clone()), word, start_line, start_column)
                    } else if c.is_digit(10) {
                        // Number
                        let number_str = self.read_number();
                        if let Ok(num) = number_str.parse::<i32>() {
                            Token::new(TokenType::Number(num), number_str, start_line, start_column)
                        } else {
                            Token::new(TokenType::Error(format!("Invalid number: {}", number_str)), number_str, start_line, start_column)
                        }
                    } else {
                        // Unknown character, consume it
                        self.consume_char();
                        Token::new(TokenType::Error(format!("Unexpected character: {}", c)), c.to_string(), start_line, start_column)
                    }
                }
            }
        } else {
            // End of input
            Token::new(TokenType::Eof, "".to_string(), self.line, self.column)
        }
    }
    
    /// Read a word (alphanumeric and underscores)
    fn read_word(&mut self) -> String {
        let mut word = String::new();
        
        while let Some(&c) = self.chars.peek() {
            if c.is_alphanumeric() || c == '_' {
                word.push(c);
                self.consume_char();
            } else {
                break;
            }
        }
        
        word
    }
    
    /// Read a variable name (after $)
    fn read_variable_name(&mut self) -> String {
        let mut name = String::new();
        
        while let Some(&c) = self.chars.peek() {
            if c.is_alphanumeric() || c == '_' {
                name.push(c);
                self.consume_char();
            } else {
                break;
            }
        }
        
        name
    }
    
    /// Read a number
    fn read_number(&mut self) -> String {
        let mut number = String::new();
        
        while let Some(&c) = self.chars.peek() {
            if c.is_digit(10) {
                number.push(c);
                self.consume_char();
            } else {
                break;
            }
        }
        
        number
    }
    
    /// Read a quoted string
    fn read_quoted_string(&mut self, quote_char: char) -> String {
        let mut content = String::new();
        
        while let Some(&c) = self.chars.peek() {
            if c == quote_char {
                self.consume_char(); // Consume closing quote
                break;
            } else if c == '\\' && quote_char == '"' {
                // Handle escapes in double quotes
                self.consume_char();
                if let Some(&next) = self.chars.peek() {
                    content.push(next);
                    self.consume_char();
                }
            } else {
                content.push(c);
                self.consume_char();
            }
        }
        
        content
    }
    
    /// Read a comment
    fn read_comment(&mut self) -> String {
        let mut comment = String::new();
        
        while let Some(&c) = self.chars.peek() {
            if c == '\n' {
                break;
            }
            comment.push(c);
            self.consume_char();
        }
        
        comment
    }
    
    /// Read assignment value
    fn read_assignment_value(&mut self) -> String {
        let mut value = String::new();
        
        while let Some(&c) = self.chars.peek() {
            if c == ' ' || c == '\t' || c == '\n' || c == ';' || c == '&' || c == '|' {
                break;
            }
            value.push(c);
            self.consume_char();
        }
        
        value
    }
    
    /// Read here-document content
    fn read_here_doc_content(&mut self) -> Token {
        // TODO: Implement here-document reading
        // For now, just return EOF
        Token::new(TokenType::Eof, "".to_string(), self.line, self.column)
    }
    
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