//! Complete POSIX Shell token definitions
//! Based on POSIX.1-2017 Shell Command Language

use std::fmt;

/// Token types for POSIX Shell parsing
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    // ============================================
    // Words and Identifiers
    // ============================================
    Word(String),           // Any word
    AssignmentWord(String), // VAR=value or VAR= (empty assignment)
    Name(String),          // Valid identifier (for variable names)
    
    // ============================================
    // Operators (from POSIX Shell Grammar)
    // ============================================
    
    // Control operators
    Newline,               // \n
    Semicolon,             // ;
    Ampersand,             // &
    Pipe,                  // |
    DSemi,                 // ;; (case terminator)
    
    // AND-OR list operators
    AndIf,                 // &&
    OrIf,                  // ||
    
    // Redirection operators
    Less,                  // <
    Great,                 // >
    DLess,                 // <<
    DGreat,                // >>
    LessAnd,               // <&
    GreatAnd,              // >&
    LessGreat,             // <>
    DLessDash,             // <<-
    Clobber,               // >|
    
    // File descriptor numbers for redirection
    Number(i32),           // e.g., 2>file
    
    // ============================================
    // Reserved Words (POSIX Shell reserved words)
    // ============================================
    Bang,                  // !
    Case,                  // case
    Do,                    // do
    Done,                  // done
    Elif,                  // elif
    Else,                  // else
    Esac,                  // esac
    Fi,                    // fi
    For,                   // for
    If,                    // if
    In,                    // in
    Then,                  // then
    Until,                 // until
    While,                 // while
    
    // Additional reserved words (optional in POSIX)
    Function,              // function
    Select,                // select
    Time,                  // time
    
    // ============================================
    // Brackets and Special Characters
    // ============================================
    LeftParen,             // (
    RightParen,            // )
    LeftBrace,             // {
    RightBrace,            // }
    
    // ============================================
    // Parameter Expansion and Command Substitution
    // ============================================
    Dollar,                // $
    DollarLeftParen,       // $(
    DollarLeftBrace,       // ${
    Backtick,              // `
    
    // ============================================
    // Pattern Matching (for case statements)
    // ============================================
    Pattern(String),       // Pattern in case statement
    
    // ============================================
    // Here-document delimiters
    // ============================================
    HereDocDelimiter(String), // <<WORD or <<-WORD delimiter
    
    // ============================================
    // Special Tokens
    // ============================================
    Eof,
    Error(String),
}

/// A token with location information
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize, column: usize) -> Self {
        Self {
            token_type,
            lexeme,
            line,
            column,
        }
    }
    
    pub fn is_word(&self) -> bool {
        matches!(self.token_type, TokenType::Word(_))
    }
    
    pub fn is_assignment(&self) -> bool {
        matches!(self.token_type, TokenType::AssignmentWord(_))
    }
    
    pub fn is_name(&self) -> bool {
        matches!(self.token_type, TokenType::Name(_))
    }
    
    pub fn is_control_operator(&self) -> bool {
        match self.token_type {
            TokenType::Newline |
            TokenType::Semicolon |
            TokenType::Ampersand |
            TokenType::Pipe |
            TokenType::DSemi => true,
            _ => false,
        }
    }
    
    pub fn is_and_or_operator(&self) -> bool {
        matches!(self.token_type, TokenType::AndIf | TokenType::OrIf)
    }
    
    pub fn is_redirect_operator(&self) -> bool {
        match self.token_type {
            TokenType::Less |
            TokenType::Great |
            TokenType::DLess |
            TokenType::DGreat |
            TokenType::LessAnd |
            TokenType::GreatAnd |
            TokenType::LessGreat |
            TokenType::DLessDash |
            TokenType::Clobber => true,
            _ => false,
        }
    }
    
    pub fn is_reserved_word(&self) -> bool {
        match self.token_type {
            TokenType::Bang |
            TokenType::Case |
            TokenType::Do |
            TokenType::Done |
            TokenType::Elif |
            TokenType::Else |
            TokenType::Esac |
            TokenType::Fi |
            TokenType::For |
            TokenType::If |
            TokenType::In |
            TokenType::Then |
            TokenType::Until |
            TokenType::While |
            TokenType::Function |
            TokenType::Select |
            TokenType::Time => true,
            _ => false,
        }
    }
    
    pub fn is_number(&self) -> bool {
        matches!(self.token_type, TokenType::Number(_))
    }
    
    pub fn is_here_doc_delimiter(&self) -> bool {
        matches!(self.token_type, TokenType::HereDocDelimiter(_))
    }
    
    pub fn is_pattern(&self) -> bool {
        matches!(self.token_type, TokenType::Pattern(_))
    }
    
    pub fn is_dollar_expansion(&self) -> bool {
        match self.token_type {
            TokenType::Dollar |
            TokenType::DollarLeftParen |
            TokenType::DollarLeftBrace => true,
            _ => false,
        }
    }
    
    pub fn is_backtick(&self) -> bool {
        matches!(self.token_type, TokenType::Backtick)
    }
    
    pub fn is_bracket(&self) -> bool {
        match self.token_type {
            TokenType::LeftParen |
            TokenType::RightParen |
            TokenType::LeftBrace |
            TokenType::RightBrace => true,
            _ => false,
        }
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenType::Word(s) => write!(f, "Word({})", s),
            TokenType::AssignmentWord(s) => write!(f, "AssignmentWord({})", s),
            TokenType::Name(s) => write!(f, "Name({})", s),
            TokenType::Newline => write!(f, "\\n"),
            TokenType::Semicolon => write!(f, ";"),
            TokenType::Ampersand => write!(f, "&"),
            TokenType::Pipe => write!(f, "|"),
            TokenType::DSemi => write!(f, ";;"),
            TokenType::AndIf => write!(f, "&&"),
            TokenType::OrIf => write!(f, "||"),
            TokenType::Less => write!(f, "<"),
            TokenType::Great => write!(f, ">"),
            TokenType::DLess => write!(f, "<<"),
            TokenType::DGreat => write!(f, ">>"),
            TokenType::LessAnd => write!(f, "<&"),
            TokenType::GreatAnd => write!(f, ">&"),
            TokenType::LessGreat => write!(f, "<>"),
            TokenType::DLessDash => write!(f, "<<-"),
            TokenType::Clobber => write!(f, ">|"),
            TokenType::Number(n) => write!(f, "Number({})", n),
            TokenType::Bang => write!(f, "!"),
            TokenType::Case => write!(f, "case"),
            TokenType::Do => write!(f, "do"),
            TokenType::Done => write!(f, "done"),
            TokenType::Elif => write!(f, "elif"),
            TokenType::Else => write!(f, "else"),
            TokenType::Esac => write!(f, "esac"),
            TokenType::Fi => write!(f, "fi"),
            TokenType::For => write!(f, "for"),
            TokenType::If => write!(f, "if"),
            TokenType::In => write!(f, "in"),
            TokenType::Then => write!(f, "then"),
            TokenType::Until => write!(f, "until"),
            TokenType::While => write!(f, "while"),
            TokenType::Function => write!(f, "function"),
            TokenType::Select => write!(f, "select"),
            TokenType::Time => write!(f, "time"),
            TokenType::LeftParen => write!(f, "("),
            TokenType::RightParen => write!(f, ")"),
            TokenType::LeftBrace => write!(f, "{{"),
            TokenType::RightBrace => write!(f, "}}"),
            TokenType::Dollar => write!(f, "$"),
            TokenType::DollarLeftParen => write!(f, "$("),
            TokenType::DollarLeftBrace => write!(f, "${{"),
            TokenType::Backtick => write!(f, "`"),
            TokenType::Pattern(s) => write!(f, "Pattern({})", s),
            TokenType::HereDocDelimiter(s) => write!(f, "HereDocDelimiter({})", s),
            TokenType::Eof => write!(f, "EOF"),
            TokenType::Error(msg) => write!(f, "Error({})", msg),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Token({} at {}:{})", self.token_type, self.line, self.column)
    }
}

/// Check if a string is a valid variable name according to POSIX
pub fn is_valid_var_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    
    let mut chars = name.chars();
    
    // First character must be letter or underscore
    let first = chars.next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    
    // Remaining characters must be alphanumeric or underscore
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Check if a string is a reserved word
pub fn is_reserved_word(word: &str) -> bool {
    match word {
        "!" | "case" | "do" | "done" | "elif" | "else" | "esac" | "fi" |
        "for" | "if" | "in" | "then" | "until" | "while" | "function" |
        "select" | "time" => true,
        _ => false,
    }
}

/// Get token type for a reserved word
pub fn reserved_word_token_type(word: &str) -> Option<TokenType> {
    match word {
        "!" => Some(TokenType::Bang),
        "case" => Some(TokenType::Case),
        "do" => Some(TokenType::Do),
        "done" => Some(TokenType::Done),
        "elif" => Some(TokenType::Elif),
        "else" => Some(TokenType::Else),
        "esac" => Some(TokenType::Esac),
        "fi" => Some(TokenType::Fi),
        "for" => Some(TokenType::For),
        "if" => Some(TokenType::If),
        "in" => Some(TokenType::In),
        "then" => Some(TokenType::Then),
        "until" => Some(TokenType::Until),
        "while" => Some(TokenType::While),
        "function" => Some(TokenType::Function),
        "select" => Some(TokenType::Select),
        "time" => Some(TokenType::Time),
        _ => None,
    }
}