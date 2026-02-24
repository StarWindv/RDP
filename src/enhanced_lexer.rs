//! Enhanced POSIX Shell Lexer with full POSIX support
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
    here_doc_delimiters: Vec<(String, bool)>, // (delimiter, strip_tabs flag)
    in_here_doc: bool,
    here_doc_content: Option<String>,
    pending_here_doc: Option<(String, bool)>, // Delimiter to process after command
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
            pending_here_doc: None,
        }
    }
    
    /// Get the next token from the input
    pub fn next_token(&mut self) -> Token {
        // Handle pending here-document delimiter
        if let Some((delimiter, strip_tabs)) = self.pending_here_doc.take() {
            let delimiter_clone = delimiter.clone();
            self.here_doc_delimiters.push((delimiter_clone.clone(), strip_tabs));
            return Token::new(
                TokenType::HereDocDelimiter(delimiter_clone),
                delimiter,
                self.line,
                self.column,
            );
        }
        
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
                    // Check for <<, <<-, <&, <>, <( (process substitution)
                    if self.match_char('<') {
                        // Check for <<-
                        let strip_tabs = self.match_char('-');
                        if strip_tabs {
                            // The delimiter will be read after the command
                            self.pending_here_doc = Some((self.read_here_doc_delimiter(), true));
                            return Token::new(TokenType::DLessDash, "<<-".to_string(), start_line, start_column);
                        } else if let Some(&next) = self.chars.peek() {
                            if next == '(' {
                                // Process substitution: <(
                                self.consume_char();
                                return Token::new(TokenType::LessLeftParen, "<(".to_string(), start_line, start_column);
                            }
                        }
                        // Regular here-document
                        // The delimiter will be read after the command
                        self.pending_here_doc = Some((self.read_here_doc_delimiter(), false));
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
                    // Check for >>, >&, >( (process substitution)
                    if self.match_char('>') {
                        return Token::new(TokenType::DGreat, ">>".to_string(), start_line, start_column);
                    } else if self.match_char('&') {
                        return Token::new(TokenType::GreatAnd, ">&".to_string(), start_line, start_column);
                    } else if let Some(&next) = self.chars.peek() {
                        if next == '(' {
                            // Process substitution: >(
                            self.consume_char();
                            return Token::new(TokenType::GreatLeftParen, ">(".to_string(), start_line, start_column);
                        }
                    }
                    Token::new(TokenType::Great, ">".to_string(), start_line, start_column)
                }
                
                // ============================================
                // Bang operator (history expansion) and pattern matching
                // ============================================
                '!' => {
                    self.consume_char();
                    // Check if it's a reserved word or pattern matching
                    if self.is_next_char_word_boundary() {
                        // It's the reserved word "!"
                        return Token::new(TokenType::Bang, "!".to_string(), start_line, start_column);
                    } else {
                        // Pattern matching or history expansion
                        // Check for !( pattern (extended globbing)
                        if let Some(&next) = self.chars.peek() {
                            if next == '(' {
                                self.consume_char();
                                return Token::new(TokenType::Exclamation, "!(".to_string(), start_line, start_column);
                            }
                        }
                        // History expansion, read as word
                        let mut word = String::from("!");
                        while let Some(&c) = self.chars.peek() {
                            if c.is_whitespace() || self.is_special_character(c) {
                                break;
                            }
                            word.push(c);
                            self.consume_char();
                        }
                        return Token::new(TokenType::Word(word.clone()), word, start_line, start_column);
                    }
                }
                
                // ============================================
                // Brackets
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
                // Pattern matching operators
                // ============================================
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
                
                '@' => {
                    self.consume_char();
                    Token::new(TokenType::At, "@".to_string(), start_line, start_column)
                }
                
                // ============================================
                // Parameter expansion and command substitution
                // ============================================
                '$' => {
                    self.consume_char();
                    // Check for $(, ${, $(( (arithmetic expansion)
                    if let Some(&next) = self.chars.peek() {
                        if next == '(' {
                            self.consume_char();
                            // Check for $(( (arithmetic expansion)
                            if self.match_char('(') {
                                return Token::new(TokenType::DollarDLeftParen, "$((".to_string(), start_line, start_column);
                            }
                            return Token::new(TokenType::DollarLeftParen, "$(".to_string(), start_line, start_column);
                        } else if next == '{' {
                            self.consume_char();
                            return Token::new(TokenType::DollarLeftBrace, "${".to_string(), start_line, start_column);
                        } else if next == '[' {
                            // bash/zsh extension: $[ ... ] for arithmetic
                            self.consume_char();
                            return Token::new(TokenType::Word("$[".to_string()), "$[".to_string(), start_line, start_column);
                        }
                    }
                    // Otherwise, it's just a $
                    Token::new(TokenType::Dollar, "$".to_string(), start_line, start_column)
                }
                
                '`' => {
                    self.consume_char();
                    Token::new(TokenType::Backtick, "`".to_string(), start_line, start_column)
                }
                
                // ============================================
                // Quotes
                // ============================================
                '\'' => self.read_single_quoted_string(start_line, start_column),
                '"' => self.read_double_quoted_string(start_line, start_column),
                
                // ============================================
                // Comments
                // ============================================
                '#' => {
                    // Comment, skip to end of line
                    while let Some(&c) = self.chars.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.consume_char();
                    }
                    self.next_token()
                }
                
                // ============================================
                // Numbers (for file descriptors)
                // ============================================
                '0'..='9' => {
                    let number = self.read_number();
                    Token::new(TokenType::Number(number), number.to_string(), start_line, start_column)
                }
                
                // ============================================
                // Words and other tokens
                // ============================================
                _ => {
                    // Check for assignment word (VAR=value)
                    if self.looks_like_assignment() {
                        let assignment = self.read_assignment_word();
                        if is_valid_var_name(assignment.split('=').next().unwrap()) {
                            return Token::new(
                                TokenType::AssignmentWord(assignment.clone()),
                                assignment,
                                start_line,
                                start_column,
                            );
                        }
                    }
                    
                    // Read a regular word
                    let word = self.read_word();
                    
                    // Check if it's a reserved word
                    if is_reserved_word(&word) {
                        if let Some(token_type) = reserved_word_token_type(&word) {
                            return Token::new(token_type, word, start_line, start_column);
                        }
                    }
                    
                    // Check if it's a valid name (for variable names)
                    if is_valid_var_name(&word) {
                        Token::new(TokenType::Name(word.clone()), word, start_line, start_column)
                    } else {
                        Token::new(TokenType::Word(word.clone()), word, start_line, start_column)
                    }
                }
            }
        } else {
            Token::new(TokenType::Eof, "".to_string(), self.line, self.column)
        }
    }
    
    /// Read a single-quoted string
    fn read_single_quoted_string(&mut self, start_line: usize, start_column: usize) -> Token {
        let mut content = String::new();
        content.push('\'');
        self.consume_char(); // Consume opening quote
        
        while let Some(&c) = self.chars.peek() {
            if c == '\'' {
                content.push(c);
                self.consume_char();
                break;
            }
            content.push(c);
            self.consume_char();
        }
        
        Token::new(TokenType::Word(content.clone()), content, start_line, start_column)
    }
    
    /// Read a double-quoted string
    fn read_double_quoted_string(&mut self, start_line: usize, start_column: usize) -> Token {
        let mut content = String::new();
        content.push('"');
        self.consume_char(); // Consume opening quote
        
        let mut escape_next = false;
        
        while let Some(&c) = self.chars.peek() {
            if escape_next {
                // In double quotes, only certain escapes are special
                match c {
                    '\\' | '"' | '$' | '`' | '\n' => {
                        // These are properly escaped
                        content.push(c);
                    }
                    _ => {
                        // Other characters: keep both backslash and character
                        content.push('\\');
                        content.push(c);
                    }
                }
                self.consume_char();
                escape_next = false;
                continue;
            }
            
            if c == '\\' {
                escape_next = true;
                content.push(c);
                self.consume_char();
            } else if c == '"' {
                content.push(c);
                self.consume_char();
                break;
            } else {
                content.push(c);
                self.consume_char();
            }
        }
        
        Token::new(TokenType::Word(content.clone()), content, start_line, start_column)
    }
    
    /// Read a word (unquoted characters)
    fn read_word(&mut self) -> String {
        let mut word = String::new();
        let mut in_quote = None;
        let mut escape_next = false;
        let mut brace_depth = 0; // For ${...} expansions
        let mut paren_depth = 0; // For $(...) and $((...)) expansions
        
        while let Some(&c) = self.chars.peek() {
            if escape_next {
                word.push(c);
                self.consume_char();
                escape_next = false;
                continue;
            }
            
            if c == '\\' {
                escape_next = true;
                word.push(c);
                self.consume_char();
            } else if c == '\'' || c == '"' {
                if in_quote.is_none() {
                    in_quote = Some(c);
                } else if in_quote == Some(c) {
                    in_quote = None;
                }
                word.push(c);
                self.consume_char();
            } else if in_quote.is_none() {
                // Handle nested expansions
                if c == '{' && word.ends_with('$') {
                    brace_depth += 1;
                    word.push(c);
                    self.consume_char();
                    continue;
                } else if c == '}' && brace_depth > 0 {
                    brace_depth -= 1;
                    word.push(c);
                    self.consume_char();
                    continue;
                } else if c == '(' && word.ends_with('$') {
                    paren_depth += 1;
                    word.push(c);
                    self.consume_char();
                    continue;
                } else if c == ')' && paren_depth > 0 {
                    paren_depth -= 1;
                    word.push(c);
                    self.consume_char();
                    continue;
                }
                
                // Check if this character ends the word
                if c.is_whitespace() || (self.is_special_character(c) && brace_depth == 0 && paren_depth == 0) {
                    break;
                }
                word.push(c);
                self.consume_char();
            } else {
                // Inside quotes, accept any character
                word.push(c);
                self.consume_char();
            }
        }
        
        word
    }
    
    /// Read an assignment word (VAR=value)
    fn read_assignment_word(&mut self) -> String {
        let mut word = String::new();
        let mut found_equals = false;
        let mut in_quote = None;
        let mut escape_next = false;
        
        while let Some(&c) = self.chars.peek() {
            if escape_next {
                word.push(c);
                self.consume_char();
                escape_next = false;
                continue;
            }
            
            if c == '\\' {
                escape_next = true;
                word.push(c);
                self.consume_char();
            } else if c == '\'' || c == '"' {
                if in_quote.is_none() {
                    in_quote = Some(c);
                } else if in_quote == Some(c) {
                    in_quote = None;
                }
                word.push(c);
                self.consume_char();
            } else if in_quote.is_none() {
                if c == '=' && !found_equals {
                    found_equals = true;
                    word.push(c);
                    self.consume_char();
                } else if found_equals {
                    // After =, read until whitespace or special character
                    if c.is_whitespace() || self.is_special_character(c) {
                        break;
                    }
                    word.push(c);
                    self.consume_char();
                } else {
                    // Before =, accept any non-whitespace, non-special character
                    if c.is_whitespace() || self.is_special_character(c) {
                        break;
                    }
                    word.push(c);
                    self.consume_char();
                }
            } else {
                // Inside quotes, accept any character
                word.push(c);
                self.consume_char();
            }
        }
        
        word
    }
    
    /// Read a number
    fn read_number(&mut self) -> i32 {
        let mut num_str = String::new();
        
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_digit() {
                num_str.push(c);
                self.consume_char();
            } else {
                break;
            }
        }
        
        num_str.parse().unwrap_or(0)
    }
    
    /// Read a here-document delimiter
    fn read_here_doc_delimiter(&mut self) -> String {
        self.skip_whitespace();
        self.read_word()
    }
    
    /// Read here-document content
    fn read_here_doc_content(&mut self) -> Token {
        let start_line = self.line;
        let start_column = self.column;
        
        if self.here_doc_content.is_none() {
            self.here_doc_content = Some(String::new());
        }
        
        let mut line = String::new();
        
        // Read until end of line
        while let Some(&c) = self.chars.peek() {
            if c == '\n' {
                self.consume_char();
                self.line += 1;
                self.column = 1;
                self.current_line_start = self.current_position();
                break;
            }
            line.push(c);
            self.consume_char();
        }
        
        // Check if this line matches the delimiter
        if let Some((delimiter, strip_tabs)) = self.here_doc_delimiters.last() {
            let mut trimmed_line = line.trim_end(); // Remove trailing spaces
            
            // If strip_tabs is true, also remove leading tabs
            if *strip_tabs {
                trimmed_line = trimmed_line.trim_start_matches('\t');
            }
            
            if trimmed_line == *delimiter {
                // End of here-document
                self.here_doc_delimiters.pop();
                self.in_here_doc = self.here_doc_delimiters.len() > 0;
                
                // Return the accumulated content as a word token
                let content = self.here_doc_content.take().unwrap();
                let content_clone = content.clone();
                return Token::new(TokenType::Word(content_clone), content, start_line, start_column);
            }
        }
        
        // Add line to content (with newline)
        if let Some(content) = &mut self.here_doc_content {
            content.push_str(&line);
            content.push('\n');
        }
        
        // Continue reading here-doc content
        self.next_token()
    }
    
    /// Check if the next token looks like an assignment
    fn looks_like_assignment(&mut self) -> bool {
        let mut chars = self.chars.clone();
        let mut has_equals = false;
        let mut before_equals = String::new();
        let mut in_quote = None;
        let mut escape_next = false;
        
        while let Some(&c) = chars.peek() {
            if escape_next {
                before_equals.push(c);
                chars.next();
                escape_next = false;
                continue;
            }
            
            if c == '\\' {
                escape_next = true;
                before_equals.push(c);
                chars.next();
            } else if c == '\'' || c == '"' {
                if in_quote.is_none() {
                    in_quote = Some(c);
                } else if in_quote == Some(c) {
                    in_quote = None;
                }
                before_equals.push(c);
                chars.next();
            } else if in_quote.is_none() {
                if c == '=' {
                    has_equals = true;
                    break;
                }
                if c.is_whitespace() || self.is_special_character(c) {
                    break;
                }
                before_equals.push(c);
                chars.next();
            } else {
                before_equals.push(c);
                chars.next();
            }
        }
        
        has_equals && is_valid_var_name(&before_equals)
    }
    
    /// Check if next character is a word boundary
    fn is_next_char_word_boundary(&mut self) -> bool {
        let mut chars = self.chars.clone();
        if let Some(&c) = chars.peek() {
            c.is_whitespace() || self.is_special_character(c)
        } else {
            true // EOF is also a boundary
        }
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