//! Complete POSIX Shell Lexer
//! Based on POSIX.1-2017 Shell Command Language

use std::iter::Peekable;
use std::str::Chars;

use super::tokens::{
    is_reserved_word, is_valid_var_name, reserved_word_token_type, Token, TokenType,
};

/// Lexer for tokenizing POSIX Shell scripts
pub struct Lexer<'a> {
    input: &'a str,
    chars: Peekable<Chars<'a>>,
    line: usize,
    column: usize,
    current_line_start: usize,
    here_doc_delimiters: Vec<String>, // Stack of here-document delimiters
    in_here_doc: bool,                // Currently reading a here-document
    here_doc_content: Option<String>, // Accumulated here-doc content
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
                    Token::new(
                        TokenType::Newline,
                        "\n".to_string(),
                        start_line,
                        start_column,
                    )
                }

                ';' => {
                    self.consume_char();
                    // Check for ;;
                    if self.match_char(';') {
                        return Token::new(
                            TokenType::DSemi,
                            ";;".to_string(),
                            start_line,
                            start_column,
                        );
                    }
                    Token::new(
                        TokenType::Semicolon,
                        ";".to_string(),
                        start_line,
                        start_column,
                    )
                }

                '&' => {
                    self.consume_char();
                    // Check for &&
                    if self.match_char('&') {
                        return Token::new(
                            TokenType::AndIf,
                            "&&".to_string(),
                            start_line,
                            start_column,
                        );
                    }
                    Token::new(
                        TokenType::Ampersand,
                        "&".to_string(),
                        start_line,
                        start_column,
                    )
                }

                '|' => {
                    self.consume_char();
                    // Check for ||
                    if self.match_char('|') {
                        return Token::new(
                            TokenType::OrIf,
                            "||".to_string(),
                            start_line,
                            start_column,
                        );
                    }
                    // Check for >|
                    if self.match_char('>') {
                        return Token::new(
                            TokenType::Clobber,
                            ">|".to_string(),
                            start_line,
                            start_column,
                        );
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
                            // This is a here-document with tab stripping
                            // The next token will be the delimiter
                            let delimiter = self.read_here_doc_delimiter();
                            self.here_doc_delimiters.push(delimiter.clone());
                            return Token::new(
                                TokenType::DLessDash,
                                "<<-".to_string(),
                                start_line,
                                start_column,
                            );
                        } else {
                            // Regular here-document
                            let delimiter = self.read_here_doc_delimiter();
                            self.here_doc_delimiters.push(delimiter.clone());
                            return Token::new(
                                TokenType::DLess,
                                "<<".to_string(),
                                start_line,
                                start_column,
                            );
                        }
                    } else if self.match_char('&') {
                        return Token::new(
                            TokenType::LessAnd,
                            "<&".to_string(),
                            start_line,
                            start_column,
                        );
                    } else if self.match_char('>') {
                        return Token::new(
                            TokenType::LessGreat,
                            "<>".to_string(),
                            start_line,
                            start_column,
                        );
                    }
                    Token::new(TokenType::Less, "<".to_string(), start_line, start_column)
                }

                '>' => {
                    self.consume_char();
                    // Check for >>, >&
                    if self.match_char('>') {
                        return Token::new(
                            TokenType::DGreat,
                            ">>".to_string(),
                            start_line,
                            start_column,
                        );
                    } else if self.match_char('&') {
                        return Token::new(
                            TokenType::GreatAnd,
                            ">&".to_string(),
                            start_line,
                            start_column,
                        );
                    }
                    Token::new(TokenType::Great, ">".to_string(), start_line, start_column)
                }

                // ============================================
                // Brackets
                // ============================================
                '(' => {
                    self.consume_char();
                    Token::new(
                        TokenType::LeftParen,
                        "(".to_string(),
                        start_line,
                        start_column,
                    )
                }

                ')' => {
                    self.consume_char();
                    Token::new(
                        TokenType::RightParen,
                        ")".to_string(),
                        start_line,
                        start_column,
                    )
                }

                '{' => {
                    self.consume_char();
                    Token::new(
                        TokenType::LeftBrace,
                        "{".to_string(),
                        start_line,
                        start_column,
                    )
                }

                '}' => {
                    self.consume_char();
                    Token::new(
                        TokenType::RightBrace,
                        "}".to_string(),
                        start_line,
                        start_column,
                    )
                }

                // ============================================
                // Parameter expansion and command substitution
                // ============================================
                '$' => {
                    self.consume_char();
                    // Check for $( or ${
                    if let Some(&next) = self.chars.peek() {
                        if next == '(' {
                            self.consume_char();
                            return Token::new(
                                TokenType::DollarLeftParen,
                                "$(".to_string(),
                                start_line,
                                start_column,
                            );
                        } else if next == '{' {
                            self.consume_char();
                            return Token::new(
                                TokenType::DollarLeftBrace,
                                "${".to_string(),
                                start_line,
                                start_column,
                            );
                        }
                    }
                    // Otherwise, it's just a $
                    Token::new(TokenType::Dollar, "$".to_string(), start_line, start_column)
                }

                '`' => {
                    self.consume_char();
                    Token::new(
                        TokenType::Backtick,
                        "`".to_string(),
                        start_line,
                        start_column,
                    )
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
                    // Check if this is a file descriptor for redirection
                    // In POSIX shell, numbers are only special when followed by redirection operators
                    let number = self.read_number();

                    // Look ahead to see if next token is a redirection operator
                    let mut lookahead = self.chars.clone();
                    let mut is_redirect_fd = false;

                    // Skip whitespace
                    while let Some(&c) = lookahead.peek() {
                        if c == ' ' || c == '\t' || c == '\r' {
                            lookahead.next();
                        } else {
                            break;
                        }
                    }

                    // Check if next non-whitespace character is a redirection operator
                    if let Some(&c) = lookahead.peek() {
                        if c == '<' || c == '>' {
                            is_redirect_fd = true;
                        }
                    }

                    if is_redirect_fd {
                        // This is a file descriptor for redirection
                        Token::new(
                            TokenType::Number(number),
                            number.to_string(),
                            start_line,
                            start_column,
                        )
                    } else {
                        // This is just a regular number argument, treat it as a word
                        Token::new(
                            TokenType::Word(number.to_string()),
                            number.to_string(),
                            start_line,
                            start_column,
                        )
                    }
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
                        Token::new(
                            TokenType::Name(word.clone()),
                            word,
                            start_line,
                            start_column,
                        )
                    } else {
                        Token::new(
                            TokenType::Word(word.clone()),
                            word,
                            start_line,
                            start_column,
                        )
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
        // Don't include the opening quote - just the content
        self.consume_char(); // Consume opening quote

        while let Some(&c) = self.chars.peek() {
            if c == '\'' {
                self.consume_char();  // Consume closing quote
                break;
            }
            content.push(c);
            self.consume_char();
        }

        // Use SingleQuotedString to preserve semantic info
        Token::new(
            TokenType::SingleQuotedString(content.clone()),
            format!("'{}'", content),  // lexeme for logging
            start_line,
            start_column,
        )
    }

    /// Read a double-quoted string
    fn read_double_quoted_string(&mut self, start_line: usize, start_column: usize) -> Token {
        let mut content = String::new();
        // Don't include the opening quote - just the content
        self.consume_char(); // Consume opening quote

        let mut escape_next = false;

        while let Some(&c) = self.chars.peek() {
            if escape_next {
                // In double quotes, only certain escapes are special
                match c {
                    '\\' | '"' | '$' | '`' | '\n' => {
                        // These are properly escaped - store just the character
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
                // Found closing quote - don't include it in content
                self.consume_char();
                break;
            } else {
                content.push(c);
                self.consume_char();
            }
        }

        // Use QuotedString to preserve semantic info
        Token::new(
            TokenType::QuotedString(content.clone()),
            format!("\"{}\"", content),  // lexeme for logging
            start_line,
            start_column,
        )
    }

    /// Read a word (unquoted characters)
    fn read_word(&mut self) -> String {
        let mut word = String::new();
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

    /// Read an assignment word (VAR=value)
    fn read_assignment_word(&mut self) -> String {
        let mut word = String::new();
        let mut found_equals = false;

        while let Some(&c) = self.chars.peek() {
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
        let trimmed_line = line.trim_end(); // Remove trailing spaces
        if let Some(delimiter) = self.here_doc_delimiters.last() {
            if trimmed_line == *delimiter {
                // End of here-document
                self.here_doc_delimiters.pop();
                self.in_here_doc = false;

                // Return the accumulated content as a word token
                let content = self.here_doc_content.take().unwrap();
                return Token::new(
                    TokenType::Word(content.clone()),
                    content,
                    start_line,
                    start_column,
                );
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

        while let Some(&c) = chars.peek() {
            if c == '=' {
                has_equals = true;
                break;
            }
            if c.is_whitespace() || self.is_special_character(c) {
                break;
            }
            before_equals.push(c);
            chars.next();
        }

        has_equals && is_valid_var_name(&before_equals)
    }

    /// Check if character is a special shell character
    fn is_special_character(&self, c: char) -> bool {
        match c {
            ';' | '&' | '|' | '<' | '>' | '(' | ')' | '{' | '}' | '`' | '$' | '\'' | '"' | '#'
            | '\n' | '=' => true,
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

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        match token.token_type {
            TokenType::Eof => None,
            _ => Some(token),
        }
    }
}

#[cfg(test)]
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
    fn test_lexer_control_operators() {
        let input = "cmd1; cmd2 &\ncmd3";
        let mut lexer = Lexer::new(input);

        let tokens: Vec<_> = lexer.collect();
        assert_eq!(tokens.len(), 6);

        assert!(matches!(tokens[1].token_type, TokenType::Semicolon));
        assert!(matches!(tokens[3].token_type, TokenType::Ampersand));
        assert!(matches!(tokens[4].token_type, TokenType::Newline));
    }

    #[test]
    fn test_lexer_and_or_operators() {
        let input = "cmd1 && cmd2 || cmd3";
        let mut lexer = Lexer::new(input);

        let tokens: Vec<_> = lexer.collect();
        assert_eq!(tokens.len(), 5);

        assert!(matches!(tokens[1].token_type, TokenType::AndIf));
        assert!(matches!(tokens[3].token_type, TokenType::OrIf));
    }

    #[test]
    fn test_lexer_redirections() {
        let input = "cmd >out.txt <in.txt >>append.txt 2>&1";
        let mut lexer = Lexer::new(input);

        let tokens: Vec<_> = lexer.collect();
        assert_eq!(tokens.len(), 9);

        assert!(matches!(tokens[1].token_type, TokenType::Great));
        assert!(matches!(tokens[3].token_type, TokenType::Less));
        assert!(matches!(tokens[5].token_type, TokenType::DGreat));
        assert!(matches!(tokens[6].token_type, TokenType::Number(2)));
        assert!(matches!(tokens[7].token_type, TokenType::GreatAnd));
    }

    #[test]
    fn test_lexer_quotes() {
        let input = "echo 'single quoted' \"double quoted\"";
        let mut lexer = Lexer::new(input);

        let tokens: Vec<_> = lexer.collect();
        assert_eq!(tokens.len(), 3);

        if let TokenType::Word(word) = &tokens[1].token_type {
            assert_eq!(word, "'single quoted'");
        } else {
            panic!("Second token should be quoted word");
        }

        if let TokenType::Word(word) = &tokens[2].token_type {
            assert_eq!(word, "\"double quoted\"");
        } else {
            panic!("Third token should be quoted word");
        }
    }

    #[test]
    fn test_lexer_variables() {
        let input = "VAR=value echo $HOME ${PATH}";
        let mut lexer = Lexer::new(input);

        let tokens: Vec<_> = lexer.collect();
        assert_eq!(tokens.len(), 6);

        if let TokenType::AssignmentWord(word) = &tokens[0].token_type {
            assert_eq!(word, "VAR=value");
        } else {
            panic!("First token should be AssignmentWord");
        }

        assert!(matches!(tokens[2].token_type, TokenType::Dollar));
        if let TokenType::Name(name) = &tokens[3].token_type {
            assert_eq!(name, "HOME");
        } else {
            panic!("Fourth token should be Name");
        }

        assert!(matches!(tokens[4].token_type, TokenType::DollarLeftBrace));
        if let TokenType::Name(name) = &tokens[5].token_type {
            assert_eq!(name, "PATH");
        } else {
            panic!("Sixth token should be Name");
        }
    }

    #[test]
    fn test_lexer_reserved_words() {
        let input = "if then else fi while do done";
        let mut lexer = Lexer::new(input);

        let tokens: Vec<_> = lexer.collect();
        assert_eq!(tokens.len(), 7);

        assert!(matches!(tokens[0].token_type, TokenType::If));
        assert!(matches!(tokens[1].token_type, TokenType::Then));
        assert!(matches!(tokens[2].token_type, TokenType::Else));
        assert!(matches!(tokens[3].token_type, TokenType::Fi));
        assert!(matches!(tokens[4].token_type, TokenType::While));
        assert!(matches!(tokens[5].token_type, TokenType::Do));
        assert!(matches!(tokens[6].token_type, TokenType::Done));
    }

    #[test]
    fn test_lexer_command_substitution() {
        let input = "echo $(date) `whoami`";
        let mut lexer = Lexer::new(input);

        let tokens: Vec<_> = lexer.collect();
        assert_eq!(tokens.len(), 6);

        assert!(matches!(tokens[1].token_type, TokenType::DollarLeftParen));
        if let TokenType::Word(word) = &tokens[2].token_type {
            assert_eq!(word, "date");
        } else {
            panic!("Third token should be Word");
        }
        assert!(matches!(tokens[3].token_type, TokenType::RightParen));
        assert!(matches!(tokens[4].token_type, TokenType::Backtick));
        if let TokenType::Word(word) = &tokens[5].token_type {
            assert_eq!(word, "whoami");
        } else {
            panic!("Sixth token should be Word");
        }
    }
}
