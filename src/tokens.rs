//! Token definitions for shell parsing

use std::fmt;

/// Token types for shell parsing
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    // Words and identifiers
    Word(String),
    AssignmentWord(String),  // VAR=value
    
    // Operators
    Semicolon,      // ;
    Ampersand,      // &
    Pipe,           // |
    AndIf,          // &&
    OrIf,           // ||
    DSemi,          // ;;
    Less,           // <
    Great,          // >
    DLess,          // <<
    DGreat,         // >>
    LessAnd,        // <&
    GreatAnd,       // >&
    LessGreat,      // <>
    DLessDash,      // <<-
    Clobber,        // >|
    
    // Redirection operators
    RedirectIn,     // <
    RedirectOut,    // >
    RedirectAppend, // >>
    RedirectHere,   // <<
    RedirectHereDoc,// <<-
    RedirectDupIn,  // <&
    RedirectDupOut, // >&
    RedirectReadWrite, // <>
    
    // Control operators
    If,             // if
    Then,           // then
    Else,           // else
    Elif,           // elif
    Fi,             // fi
    Do,             // do
    Done,           // done
    Case,           // case
    Esac,           // esac
    While,          // while
    Until,          // until
    For,            // for
    In,             // in
    Select,         // select
    
    // Reserved words
    Bang,           // !
    Time,           // time
    Function,       // function
    LeftParen,      // (
    RightParen,     // )
    LeftBrace,      // {
    RightBrace,     // }
    
    // Special tokens
    Newline,
    Eof,
    Error(String),
    
    // For command substitution
    DollarLeftParen, // $(
    Backtick,        // `
    
    // For parameter expansion
    DollarLeftBrace, // ${
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
    
    pub fn is_operator(&self) -> bool {
        match self.token_type {
            TokenType::Semicolon |
            TokenType::Ampersand |
            TokenType::Pipe |
            TokenType::AndIf |
            TokenType::OrIf |
            TokenType::DSemi |
            TokenType::Less |
            TokenType::Great |
            TokenType::DLess |
            TokenType::DGreat |
            TokenType::LessAnd |
            TokenType::GreatAnd |
            TokenType::LessGreat |
            TokenType::DLessDash |
            TokenType::Clobber |
            TokenType::RedirectIn |
            TokenType::RedirectOut |
            TokenType::RedirectAppend |
            TokenType::RedirectHere |
            TokenType::RedirectHereDoc |
            TokenType::RedirectDupIn |
            TokenType::RedirectDupOut |
            TokenType::RedirectReadWrite => true,
            _ => false,
        }
    }
    
    pub fn is_control(&self) -> bool {
        match self.token_type {
            TokenType::If |
            TokenType::Then |
            TokenType::Else |
            TokenType::Elif |
            TokenType::Fi |
            TokenType::Do |
            TokenType::Done |
            TokenType::Case |
            TokenType::Esac |
            TokenType::While |
            TokenType::Until |
            TokenType::For |
            TokenType::In |
            TokenType::Select => true,
            _ => false,
        }
    }
    
    pub fn is_reserved(&self) -> bool {
        match self.token_type {
            TokenType::Bang |
            TokenType::Time |
            TokenType::Function => true,
            _ => false,
        }
    }
    
    pub fn is_redirect(&self) -> bool {
        match self.token_type {
            TokenType::Less |
            TokenType::Great |
            TokenType::DLess |
            TokenType::DGreat |
            TokenType::LessAnd |
            TokenType::GreatAnd |
            TokenType::LessGreat |
            TokenType::DLessDash |
            TokenType::Clobber |
            TokenType::RedirectIn |
            TokenType::RedirectOut |
            TokenType::RedirectAppend |
            TokenType::RedirectHere |
            TokenType::RedirectHereDoc |
            TokenType::RedirectDupIn |
            TokenType::RedirectDupOut |
            TokenType::RedirectReadWrite => true,
            _ => false,
        }
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenType::Word(s) => write!(f, "Word({})", s),
            TokenType::AssignmentWord(s) => write!(f, "AssignmentWord({})", s),
            TokenType::Semicolon => write!(f, ";"),
            TokenType::Ampersand => write!(f, "&"),
            TokenType::Pipe => write!(f, "|"),
            TokenType::AndIf => write!(f, "&&"),
            TokenType::OrIf => write!(f, "||"),
            TokenType::DSemi => write!(f, ";;"),
            TokenType::Less => write!(f, "<"),
            TokenType::Great => write!(f, ">"),
            TokenType::DLess => write!(f, "<<"),
            TokenType::DGreat => write!(f, ">>"),
            TokenType::LessAnd => write!(f, "<&"),
            TokenType::GreatAnd => write!(f, ">&"),
            TokenType::LessGreat => write!(f, "<>"),
            TokenType::DLessDash => write!(f, "<<-"),
            TokenType::Clobber => write!(f, ">|"),
            TokenType::RedirectIn => write!(f, "<"),
            TokenType::RedirectOut => write!(f, ">"),
            TokenType::RedirectAppend => write!(f, ">>"),
            TokenType::RedirectHere => write!(f, "<<"),
            TokenType::RedirectHereDoc => write!(f, "<<-"),
            TokenType::RedirectDupIn => write!(f, "<&"),
            TokenType::RedirectDupOut => write!(f, ">&"),
            TokenType::RedirectReadWrite => write!(f, "<>"),
            TokenType::If => write!(f, "if"),
            TokenType::Then => write!(f, "then"),
            TokenType::Else => write!(f, "else"),
            TokenType::Elif => write!(f, "elif"),
            TokenType::Fi => write!(f, "fi"),
            TokenType::Do => write!(f, "do"),
            TokenType::Done => write!(f, "done"),
            TokenType::Case => write!(f, "case"),
            TokenType::Esac => write!(f, "esac"),
            TokenType::While => write!(f, "while"),
            TokenType::Until => write!(f, "until"),
            TokenType::For => write!(f, "for"),
            TokenType::In => write!(f, "in"),
            TokenType::Select => write!(f, "select"),
            TokenType::Bang => write!(f, "!"),
            TokenType::Time => write!(f, "time"),
            TokenType::Function => write!(f, "function"),
            TokenType::LeftParen => write!(f, "("),
            TokenType::RightParen => write!(f, ")"),
            TokenType::LeftBrace => write!(f, "{{"),
            TokenType::RightBrace => write!(f, "}}"),
            TokenType::Newline => write!(f, "\\n"),
            TokenType::Eof => write!(f, "EOF"),
            TokenType::Error(msg) => write!(f, "Error({})", msg),
            TokenType::DollarLeftParen => write!(f, "$("),
            TokenType::Backtick => write!(f, "`"),
            TokenType::DollarLeftBrace => write!(f, "${{"),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Token({} at {}:{})", self.token_type, self.line, self.column)
    }
}
