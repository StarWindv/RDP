//! Lexer for shell script tokenization

use std::iter::Peekable;
use std::str::Chars;

use super::tokens::{Token, TokenType};

#[cfg(test)]
mod tests;

/// Lexer for tokenizing shell scripts
pub struct Lexer<'a> {
    input: &'a str,
    chars: Peekable<Chars<'a>>,
    line: usize,
    column: usize,
    current_line_start: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given input
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().peekable(),
            line: 1,
            column: 1,
            current_line_start: 0,
        }
    }
    
    /// Get the next token from the input
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        
        if let Some(&c) = self.chars.peek() {
            let start_line = self.line;
            let start_column = self.column;
            
            match c {
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
                '<' => {
                    self.consume_char();
                    // Check for <<, <<-, <&, <>
                    if self.match_char('<') {
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
                '(' => {
                    self.consume_char();
                    // Check for $(
                    if start_column > 1 {
                        // Look back in input to see if there was a $ before (
                        let pos = self.current_position() - 1;
                        if pos > 0 && self.input.chars().nth(pos - 1) == Some('$') {
                            return Token::new(TokenType::DollarLeftParen, "$(".to_string(), start_line, start_column - 1);
                        }
                    }
                    Token::new(TokenType::LeftParen, "(".to_string(), start_line, start_column)
                }
                ')' => {
                    self.consume_char();
                    Token::new(TokenType::RightParen, ")".to_string(), start_line, start_column)
                }
                '{' => {
                    self.consume_char();
                    // Check for ${
                    if start_column > 1 {
                        // Look back in input to see if there was a $ before {
                        let pos = self.current_position() - 1;
                        if pos > 0 && self.input.chars().nth(pos - 1) == Some('$') {
                            return Token::new(TokenType::DollarLeftBrace, "${".to_string(), start_line, start_column - 1);
                        }
                    }
                    Token::new(TokenType::LeftBrace, "{".to_string(), start_line, start_column)
                }
                '}' => {
                    self.consume_char();
                    Token::new(TokenType::RightBrace, "}".to_string(), start_line, start_column)
                }
                '`' => {
                    self.consume_char();
                    Token::new(TokenType::Backtick, "`".to_string(), start_line, start_column)
                }
                '\'' => self.read_quoted_string('\'', start_line, start_column),
                '"' => self.read_quoted_string('"', start_line, start_column),
                '$' => {
                    self.consume_char();
                    // Check for $( or ${
                    if let Some(&next) = self.chars.peek() {
                        if next == '(' {
                            self.consume_char();
                            return Token::new(TokenType::DollarLeftParen, "$(".to_string(), start_line, start_column);
                        } else if next == '{' {
                            self.consume_char();
                            return Token::new(TokenType::DollarLeftBrace, "${".to_string(), start_line, start_column);
                        }
                    }
                    // Otherwise, it's a regular word starting with $
                    let word = self.read_word(true);
                    Token::new(TokenType::Word(word), format!("${}", &word[1..]), start_line, start_column)
                }
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
                '\n' => {
                    self.consume_char();
                    self.line += 1;
                    self.column = 1;
                    self.current_line_start = self.current_position();
                    Token::new(TokenType::Newline, "\n".to_string(), start_line, start_column)
                }
                _ => {
                    // Read a word (could be regular word, assignment word, or reserved word)
                    let word = self.read_word(false);
                    self.check_reserved_word(&word, start_line, start_column)
                }
            }
        } else {
            Token::new(TokenType::Eof, "".to_string(), self.line, self.column)
        }
    }
    
    /// Read a quoted string (single or double quotes)
    fn read_quoted_string(&mut self, quote_char: char, start_line: usize, start_column: usize) -> Token {
        let mut content = String::new();
        content.push(quote_char);
        self.consume_char(); // Consume opening quote
        
        let mut escape_next = false;
        
        while let Some(&c) = self.chars.peek() {
            if escape_next {
                content.push(c);
                self.consume_char();
                escape_next = false;
                continue;
            }
            
            if c == '\\' {
                escape_next = true;
                content.push(c);
                self.consume_char();
            } else if c == quote_char {
                content.push(c);
                self.consume_char();
                break;
            } else {
                content.push(c);
                self.consume_char();
            }
        }
        
        Token::new(TokenType::Word(content), content, start_line, start_column)
    }
    
    /// Read a word from input
    fn read_word(&mut self, started_with_dollar: bool) -> String {
        let mut word = String::new();
        if started_with_dollar {
            word.push('$');
        }
        
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
                // Check if this character ends the word
                if c.is_whitespace() || self.is_special_character(c) {
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
    
    /// Check if a word is a reserved word and return appropriate token
    fn check_reserved_word(&self, word: &str, line: usize, column: usize) -> Token {
        // Check if it's an assignment word (contains = but not at start)
        if word.contains('=') && !word.starts_with('=') {
            // Check if it's a valid variable assignment
            let parts: Vec<&str> = word.splitn(2, '=').collect();
            if parts.len() == 2 && self.is_valid_var_name(parts[0]) {
                return Token::new(TokenType::AssignmentWord(word.to_string()), word.to_string(), line, column);
            }
        }
        
        // Check for reserved words
        match word {
            "if" => Token::new(TokenType::If, word.to_string(), line, column),
            "then" => Token::new(TokenType::Then, word.to_string(), line, column),
            "else" => Token::new(TokenType::Else, word.to_string(), line, column),
            "elif" => Token::new(TokenType::Elif, word.to_string(), line, column),
            "fi" => Token::new(TokenType::Fi, word.to_string(), line, column),
            "do" => Token::new(TokenType::Do, word.to_string(), line, column),
            "done" => Token::new(TokenType::Done, word.to_string(), line, column),
            "case" => Token::new(TokenType::Case, word.to_string(), line, column),
            "esac" => Token::new(TokenType::Esac, word.to_string(), line, column),
            "while" => Token::new(TokenType::While, word.to_string(), line, column),
            "until" => Token::new(TokenType::Until, word.to_string(), line, column),
            "for" => Token::new(TokenType::For, word.to_string(), line, column),
            "in" => Token::new(TokenType::In, word.to_string(), line, column),
            "select" => Token::new(TokenType::Select, word.to_string(), line, column),
            "!" => Token::new(TokenType::Bang, word.to_string(), line, column),
            "time" => Token::new(TokenType::Time, word.to_string(), line, column),
            "function" => Token::new(TokenType::Function, word.to_string(), line, column),
            _ => Token::new(TokenType::Word(word.to_string()), word.to_string(), line, column),
        }
    }
    
    /// Check if character is a special shell character
    fn is_special_character(&self, c: char) -> bool {
        match c {
            ';' | '&' | '|' | '<' | '>' | '(' | ')' | '{' | '}' | '`' | '$' | '\'' | '"' | '#' | '\n' => true,
            _ => false,
        }
    }
    
    /// Check if a string is a valid variable name
    fn is_valid_var_name(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        
        // First character must be letter or underscore
        let first_char = name.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() && first_char != '_' {
            return false;
        }
        
        // Remaining characters must be alphanumeric or underscore
        name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
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
        self.input.len() - self.chars.as_str().len()
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;
    
    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        match token.token_type {
            TokenType::Eof => None,
            _ => Some(token),
        }
    }
}#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lexer_basic() {
        let input = "echo hello world";
        let mut lexer = Lexer::new(input);
        
        let tokens: Vec<_> = lexer.collect();
        assert_eq!(tokens.len(), 3);
        
        if let TokenType::Word(word) = &tokens[0].token_type {
            assert_eq!(word, "echo");
        } else {
            panic!("First token should be Word");
        }
        
        if let TokenType::Word(word) = &tokens[1].token_type {
            assert_eq!(word, "hello");
        } else {
            panic!("Second token should be Word");
        }
        
        if let TokenType::Word(word) = &tokens[2].token_type {
            assert_eq!(word, "world");
        } else {
            panic!("Third token should be Word");
        }
    }
    
    #[test]
    fn test_lexer_operators() {
        let input = "cmd1; cmd2 && cmd3 || cmd4";
        let mut lexer = Lexer::new(input);
        
        let tokens: Vec<_> = lexer.collect();
        assert_eq!(tokens.len(), 7);
        
        assert!(matches!(tokens[1].token_type, TokenType::Semicolon));
        assert!(matches!(tokens[3].token_type, TokenType::AndIf));
        assert!(matches!(tokens[5].token_type, TokenType::OrIf));
    }
    
    #[test]
    fn test_lexer_redirections() {
        let input = "cmd > out.txt < in.txt >> append.txt";
        let mut lexer = Lexer::new(input);
        
        let tokens: Vec<_> = lexer.collect();
        assert_eq!(tokens.len(), 7);
        
        assert!(matches!(tokens[1].token_type, TokenType::Great));
        assert!(matches!(tokens[3].token_type, TokenType::Less));
        assert!(matches!(tokens[5].token_type, TokenType::DGreat));
    }
    
    #[test]
    fn test_lexer_quotes() {
        let input = "echo 'hello world' \"test string\"";
        let mut lexer = Lexer::new(input);
        
        let tokens: Vec<_> = lexer.collect();
        assert_eq!(tokens.len(), 3);
        
        if let TokenType::Word(word) = &tokens[1].token_type {
            assert_eq!(word, "'hello world'");
        } else {
            panic!("Second token should be quoted word");
        }
        
        if let TokenType::Word(word) = &tokens[2].token_type {
            assert_eq!(word, "\"test string\"");
        } else {
            panic!("Third token should be quoted word");
        }
    }
    
    #[test]
    fn test_lexer_variables() {
        let input = "VAR=value echo $HOME";
        let mut lexer = Lexer::new(input);
        
        let tokens: Vec<_> = lexer.collect();
        assert_eq!(tokens.len(), 4);
        
        if let TokenType::AssignmentWord(word) = &tokens[0].token_type {
            assert_eq!(word, "VAR=value");
        } else {
            panic!("First token should be AssignmentWord");
        }
        
        if let TokenType::Word(word) = &tokens[2].token_type {
            assert_eq!(word, "$HOME");
        } else {
            panic!("Third token should be Word with $");
        }
    }
}