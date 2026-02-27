//! Complete POSIX Shell Abstract Syntax Tree (AST) definitions

use std::fmt;

use crate::modules::tokens::Token;

/// AST node types for POSIX Shell
#[derive(Debug, Clone)]
pub enum AstNode {
    // ============================================
    // Simple Commands
    // ============================================
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

    // ============================================
    // Pipelines and Lists
    // ============================================
    /// Pipeline: command1 | command2
    Pipeline {
        commands: Vec<Box<AstNode>>,
        tokens: Vec<Token>,
    },

    /// AND-OR list: cmd1 && cmd2 || cmd3
    AndOrList {
        commands: Vec<Box<AstNode>>,
        operators: Vec<AndOrOperator>, // Same length as commands-1
        tokens: Vec<Token>,
    },

    /// Logical AND: left && right
    LogicalAnd {
        left: Box<AstNode>,
        right: Box<AstNode>,
        tokens: Vec<Token>,
    },

    /// Logical OR: left || right
    LogicalOr {
        left: Box<AstNode>,
        right: Box<AstNode>,
        tokens: Vec<Token>,
    },

    /// Command list: cmd1; cmd2 & cmd3
    CommandList {
        commands: Vec<Box<AstNode>>,
        separators: Vec<CommandSeparator>, // Same length as commands-1
        tokens: Vec<Token>,
    },

    // ============================================
    // Compound Commands
    // ============================================
    /// Compound command: { commands; }
    CompoundCommand {
        commands: Vec<Box<AstNode>>,
        tokens: Vec<Token>,
    },

    /// Subshell: ( commands )
    Subshell {
        commands: Vec<Box<AstNode>>,
        tokens: Vec<Token>,
    },

    // ============================================
    // Conditional Constructs
    // ============================================
    /// If statement: if condition; then commands; fi
    IfStatement {
        condition: Box<AstNode>,
        then_branch: Vec<Box<AstNode>>,
        else_branch: Option<Vec<Box<AstNode>>>,
        elif_branches: Vec<(Box<AstNode>, Vec<Box<AstNode>>)>,
        tokens: Vec<Token>,
    },

    /// Case statement: case word in pattern) commands;; esac
    CaseStatement {
        word: String,
        cases: Vec<CaseClause>,
        tokens: Vec<Token>,
    },

    // ============================================
    // Loop Constructs
    // ============================================
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

    /// Select statement: select var in list; do commands; done
    SelectStatement {
        variable: String,
        items: Vec<String>,
        body: Vec<Box<AstNode>>,
        tokens: Vec<Token>,
    },

    // ============================================
    // Function Definitions
    // ============================================
    /// Function definition: name() { body; }
    FunctionDefinition {
        name: String,
        body: Vec<Box<AstNode>>,
        tokens: Vec<Token>,
    },

    // ============================================
    // Redirections
    // ============================================
    /// Redirection: command > file
    Redirection {
        command: Box<AstNode>,
        redirect_type: RedirectType,
        target: String,
        fd: Option<i32>, // File descriptor, None means default (0 for input, 1 for output)
        token: Token,
    },

    /// Background command: command &
    Background { command: Box<AstNode>, token: Token },

    // ============================================
    // Command Substitution and Parameter Expansion
    // ============================================
    /// Command substitution: $(command) or `command`
    CommandSubstitution {
        command: Box<AstNode>,
        backticks: bool, // true for `command`, false for $(command)
        tokens: Vec<Token>,
    },

    /// Parameter expansion: ${parameter}
    ParameterExpansion {
        parameter: String,
        operation: Option<ParameterOperation>,
        tokens: Vec<Token>,
    },

    // ============================================
    // Special Nodes
    // ============================================
    /// Null command (empty line)
    NullCommand,

    /// Export command: export VAR
    Export {
        variables: Vec<String>,
        tokens: Vec<Token>,
    },

    /// Unset command: unset VAR
    Unset {
        variables: Vec<String>,
        tokens: Vec<Token>,
    },

    /// Readonly command: readonly VAR
    Readonly {
        variables: Vec<String>,
        tokens: Vec<Token>,
    },

    /// Error node
    Error { message: String, token: Token },
}

/// AND-OR operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndOrOperator {
    AndIf, // &&
    OrIf,  // ||
}

/// Command separator types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandSeparator {
    Semicolon, // ;
    Newline,   // \n
    Ampersand, // &
}

/// Redirection types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedirectType {
    Input,        // <
    Output,       // >
    Append,       // >>
    HereDoc,      // <<
    HereDocStrip, // <<-
    DupInput,     // <&
    DupOutput,    // >&
    ReadWrite,    // <>
    Clobber,      // >|
}

/// Case clause for case statements
#[derive(Debug, Clone)]
pub struct CaseClause {
    pub patterns: Vec<String>,
    pub body: Vec<Box<AstNode>>,
}

/// Parameter expansion operations
#[derive(Debug, Clone)]
pub enum ParameterOperation {
    // ${parameter:-word} - Use default value
    UseDefault(String),
    // ${parameter:=word} - Assign default value
    AssignDefault(String),
    // ${parameter:?word} - Error if null or unset
    ErrorIfNull(String),
    // ${parameter:+word} - Use alternate value
    UseAlternate(String),
    // ${#parameter} - String length
    Length,
    // ${parameter%word} - Remove smallest suffix pattern
    RemoveSuffix(String),
    // ${parameter%%word} - Remove largest suffix pattern
    RemoveLargestSuffix(String),
    // ${parameter#word} - Remove smallest prefix pattern
    RemovePrefix(String),
    // ${parameter##word} - Remove largest prefix pattern
    RemoveLargestPrefix(String),
}

/// Parse error
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub token: Token,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Parse error at {}:{}: {}",
            self.token.line, self.token.column, self.message
        )
    }
}

impl std::error::Error for ParseError {}

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

            AstNode::AndOrList {
                commands,
                operators,
                ..
            } => {
                write!(f, "AndOrList(")?;
                for (i, cmd) in commands.iter().enumerate() {
                    if i > 0 {
                        write!(
                            f,
                            " {} ",
                            match operators[i - 1] {
                                AndOrOperator::AndIf => "&&",
                                AndOrOperator::OrIf => "||",
                            }
                        )?;
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

            AstNode::CommandList {
                commands,
                separators,
                ..
            } => {
                write!(f, "CommandList(")?;
                for (i, cmd) in commands.iter().enumerate() {
                    if i > 0 {
                        write!(
                            f,
                            " {} ",
                            match separators[i - 1] {
                                CommandSeparator::Semicolon => ";",
                                CommandSeparator::Newline => "\\n",
                                CommandSeparator::Ampersand => "&",
                            }
                        )?;
                    }
                    write!(f, "{}", cmd)?;
                }
                write!(f, ")")
            }

            AstNode::CompoundCommand { commands, .. } => {
                write!(f, "CompoundCommand{{")?;
                for cmd in commands {
                    write!(f, " {};", cmd)?;
                }
                write!(f, " }}")
            }

            AstNode::Subshell { commands, .. } => {
                write!(f, "Subshell((")?;
                for cmd in commands {
                    write!(f, " {};", cmd)?;
                }
                write!(f, "))")
            }

            AstNode::IfStatement {
                condition,
                then_branch,
                else_branch,
                elif_branches,
                ..
            } => {
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

            AstNode::CaseStatement { word, cases, .. } => {
                write!(f, "CaseStatement(case {} in", word)?;
                for case in cases {
                    for pattern in &case.patterns {
                        write!(f, " {}", pattern)?;
                    }
                    write!(f, ")")?;
                    for cmd in &case.body {
                        write!(f, " {};", cmd)?;
                    }
                    write!(f, ";;")?;
                }
                write!(f, " esac)")
            }

            AstNode::WhileLoop {
                condition, body, ..
            } => {
                write!(f, "WhileLoop(while {}; do", condition)?;
                for cmd in body {
                    write!(f, " {};", cmd)?;
                }
                write!(f, " done)")
            }

            AstNode::UntilLoop {
                condition, body, ..
            } => {
                write!(f, "UntilLoop(until {}; do", condition)?;
                for cmd in body {
                    write!(f, " {};", cmd)?;
                }
                write!(f, " done)")
            }

            AstNode::ForLoop {
                variable,
                items,
                body,
                ..
            } => {
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

            AstNode::SelectStatement {
                variable,
                items,
                body,
                ..
            } => {
                write!(f, "SelectStatement(select {} in", variable)?;
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

            AstNode::Redirection {
                command,
                redirect_type,
                target,
                fd,
                ..
            } => {
                let fd_str = if let Some(fd) = fd {
                    format!("{}", fd)
                } else {
                    String::new()
                };
                write!(
                    f,
                    "Redirection({} {} {}{})",
                    command,
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
                    if !fd_str.is_empty() {
                        format!(" {}", fd_str)
                    } else {
                        String::new()
                    }
                )
            }

            AstNode::Background { command, .. } => {
                write!(f, "Background({} &)", command)
            }

            AstNode::CommandSubstitution {
                command, backticks, ..
            } => {
                if *backticks {
                    write!(f, "CommandSubstitution(`{}`)", command)
                } else {
                    write!(f, "CommandSubstitution($({}))", command)
                }
            }

            AstNode::ParameterExpansion {
                parameter,
                operation,
                ..
            } => {
                write!(f, "ParameterExpansion(${{{}}}", parameter)?;
                if let Some(op) = operation {
                    match op {
                        ParameterOperation::UseDefault(word) => write!(f, ":-{}", word)?,
                        ParameterOperation::AssignDefault(word) => write!(f, ":=+{}", word)?,
                        ParameterOperation::ErrorIfNull(word) => write!(f, ":?{}", word)?,
                        ParameterOperation::UseAlternate(word) => write!(f, ":+{}", word)?,
                        ParameterOperation::Length => write!(f, "#")?,
                        ParameterOperation::RemoveSuffix(word) => write!(f, "%{}", word)?,
                        ParameterOperation::RemoveLargestSuffix(word) => write!(f, "%%{}", word)?,
                        ParameterOperation::RemovePrefix(word) => write!(f, "#{}", word)?,
                        ParameterOperation::RemoveLargestPrefix(word) => write!(f, "##{}", word)?,
                    }
                }
                write!(f, ")")
            }

            AstNode::NullCommand => write!(f, "NullCommand"),

            AstNode::Export { variables, .. } => {
                write!(f, "Export(")?;
                for (i, var) in variables.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", var)?;
                }
                write!(f, ")")
            }

            AstNode::Unset { variables, .. } => {
                write!(f, "Unset(")?;
                for (i, var) in variables.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", var)?;
                }
                write!(f, ")")
            }

            AstNode::Readonly { variables, .. } => {
                write!(f, "Readonly(")?;
                for (i, var) in variables.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", var)?;
                }
                write!(f, ")")
            }

            AstNode::Error { message, .. } => write!(f, "Error({})", message),
        }
    }
}
