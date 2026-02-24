//! Abstract Syntax Tree (AST) definitions for shell parsing

use std::fmt;

use crate::tokens::Token;

/// AST node types
#[derive(Debug, Clone)]
pub enum AstNode {
    /// Simple command: name and arguments
    SimpleCommand {
        name: String,
        args: Vec<String>,
        tokens: Vec<Token>,
    },
    
    /// Variable assignment: VAR=value
    Assignment {
        name: String,
        value: String,
        token: Token,
    },
    
    /// Pipeline: command1 | command2
    Pipeline {
        commands: Vec<Box<AstNode>>,
        tokens: Vec<Token>,
    },
    
    /// Command list with separator: cmd1 ; cmd2
    CommandList {
        commands: Vec<Box<AstNode>>,
        separator: CommandSeparator,
        tokens: Vec<Token>,
    },
    
    /// Logical AND: cmd1 && cmd2
    LogicalAnd {
        left: Box<AstNode>,
        right: Box<AstNode>,
        tokens: Vec<Token>,
    },
    
    /// Logical OR: cmd1 || cmd2
    LogicalOr {
        left: Box<AstNode>,
        right: Box<AstNode>,
        tokens: Vec<Token>,
    },
    
    /// Redirection: command > file
    Redirection {
        command: Box<AstNode>,
        redirect_type: RedirectType,
        target: String,
        fd: Option<i32>,
        token: Token,
    },
    
    /// Background command: command &
    Background {
        command: Box<AstNode>,
        token: Token,
    },
    
    /// Compound command: { commands; }
    CompoundCommand {
        commands: Vec<Box<AstNode>>,
        tokens: Vec<Token>,
    },
    
    /// If statement: if condition; then commands; fi
    IfStatement {
        condition: Box<AstNode>,
        then_branch: Vec<Box<AstNode>>,
        else_branch: Option<Vec<Box<AstNode>>>,
        elif_branches: Vec<(Box<AstNode>, Vec<Box<AstNode>>)>,
        tokens: Vec<Token>,
    },
    
    /// While loop: while condition; do commands; done
    WhileLoop {
        condition: Box<AstNode>,
        body: Vec<Box<AstNode>>,
        tokens: Vec<Token>,
    },
    
    /// Until loop: until condition; do commands; done
    UntilLoop {
        condition: Box<AstNode>,
        body: Vec<Box<AstNode>>,
        tokens: Vec<Token>,
    },
    
    /// For loop: for var in list; do commands; done
    ForLoop {
        variable: String,
        items: Vec<String>,
        body: Vec<Box<AstNode>>,
        tokens: Vec<Token>,
    },
    
    /// Function definition: name() { body; }
    FunctionDefinition {
        name: String,
        body: Vec<Box<AstNode>>,
        tokens: Vec<Token>,
    },
    
    /// Subshell: ( commands )
    Subshell {
        commands: Vec<Box<AstNode>>,
        tokens: Vec<Token>,
    },
    
    /// Command substitution: $(command) or `command`
    CommandSubstitution {
        command: Box<AstNode>,
        backticks: bool,  // true for `command`, false for $(command)
        tokens: Vec<Token>,
    },
    
    /// Null command (empty line)
    NullCommand,
    
    /// Error node
    Error {
        message: String,
        token: Token,
    },
}

/// Command separator types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandSeparator {
    Semicolon,  // ;
    Newline,    // \n
    Ampersand,  // &
}

/// Redirection types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedirectType {
    Input,           // <
    Output,          // >
    Append,          // >>
    HereDoc,         // <<
    HereDocStrip,    // <<-
    DupInput,        // <&
    DupOutput,       // >&
    ReadWrite,       // <>
    Clobber,         // >|
}

impl fmt::Display for AstNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AstNode::SimpleCommand { name, args, .. } => {
                write!(f, "SimpleCommand({}", name)?;
                for arg in args {
                    write!(f, " {}", arg)?;
                }
                write!(f, ")")
            }
            AstNode::Assignment { name, value, .. } => {
                write!(f, "Assignment({}={})", name, value)
            }
            AstNode::Pipeline { commands, .. } => {
                write!(f, "Pipeline(")?;
                for (i, cmd) in commands.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{}", cmd)?;
                }
                write!(f, ")")
            }
            AstNode::CommandList { commands, separator, .. } => {
                write!(f, "CommandList(")?;
                for (i, cmd) in commands.iter().enumerate() {
                    if i > 0 {
                        write!(f, " {} ", match separator {
                            CommandSeparator::Semicolon => ";",
                            CommandSeparator::Newline => "\\n",
                            CommandSeparator::Ampersand => "&",
                        })?;
                    }
                    write!(f, "{}", cmd)?;
                }
                write!(f, ")")
            }
            AstNode::LogicalAnd { left, right, .. } => {
                write!(f, "LogicalAnd({} && {})", left, right)
            }
            AstNode::LogicalOr { left, right, .. } => {
                write!(f, "LogicalOr({} || {})", left, right)
            }
            AstNode::Redirection { command, redirect_type, target, fd, .. } => {
                let fd_str = if let Some(fd) = fd {
                    format!("{}", fd)
                } else {
                    String::new()
                };
                write!(f, "Redirection({} {} {}{})", command, 
                    match redirect_type {
                        RedirectType::Input => "<",
                        RedirectType::Output => ">",
                        RedirectType::Append => ">>",
                        RedirectType::HereDoc => "<<",
                        RedirectType::HereDocStrip => "<<-",
                        RedirectType::DupInput => "<&",
                        RedirectType::DupOutput => ">&",
                        RedirectType::ReadWrite => "<>",
                        RedirectType::Clobber => ">|",
                    },
                    target,
                    if !fd_str.is_empty() { format!(" {}", fd_str) } else { String::new() }
                )
            }
            AstNode::Background { command, .. } => {
                write!(f, "Background({} &)", command)
            }
            AstNode::CompoundCommand { commands, .. } => {
                write!(f, "CompoundCommand{{")?;
                for cmd in commands {
                    write!(f, " {};", cmd)?;
                }
                write!(f, " }}")
            }
            AstNode::IfStatement { condition, then_branch, else_branch, elif_branches, .. } => {
                write!(f, "IfStatement(if {}; then", condition)?;
                for cmd in then_branch {
                    write!(f, " {};", cmd)?;
                }
                for (cond, body) in elif_branches {
                    write!(f, " elif {}; then", cond)?;
                    for cmd in body {
                        write!(f, " {};", cmd)?;
                    }
                }
                if let Some(else_cmds) = else_branch {
                    write!(f, " else")?;
                    for cmd in else_cmds {
                        write!(f, " {};", cmd)?;
                    }
                }
                write!(f, " fi)")
            }
            AstNode::WhileLoop { condition, body, .. } => {
                write!(f, "WhileLoop(while {}; do", condition)?;
                for cmd in body {
                    write!(f, " {};", cmd)?;
                }
                write!(f, " done)")
            }
            AstNode::UntilLoop { condition, body, .. } => {
                write!(f, "UntilLoop(until {}; do", condition)?;
                for cmd in body {
                    write!(f, " {};", cmd)?;
                }
                write!(f, " done)")
            }
            AstNode::ForLoop { variable, items, body, .. } => {
                write!(f, "ForLoop(for {} in", variable)?;
                for item in items {
                    write!(f, " {}", item)?;
                }
                write!(f, "; do")?;
                for cmd in body {
                    write!(f, " {};", cmd)?;
                }
                write!(f, " done)")
            }
            AstNode::FunctionDefinition { name, body, .. } => {
                write!(f, "FunctionDefinition({}() {{", name)?;
                for cmd in body {
                    write!(f, " {};", cmd)?;
                }
                write!(f, " }})")
            }
            AstNode::Subshell { commands, .. } => {
                write!(f, "Subshell((")?;
                for cmd in commands {
                    write!(f, " {};", cmd)?;
                }
                write!(f, "))")
            }
            AstNode::CommandSubstitution { command, backticks, .. } => {
                if *backticks {
                    write!(f, "CommandSubstitution(`{}`)", command)
                } else {
                    write!(f, "CommandSubstitution($({}))", command)
                }
            }
            AstNode::NullCommand => write!(f, "NullCommand"),
            AstNode::Error { message, .. } => write!(f, "Error({})", message),
        }
    }
}

/// Parse error
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub token: Token,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Parse error at {}:{}: {}", 
            self.token.line, self.token.column, self.message)
    }
}

impl std::error::Error for ParseError {}