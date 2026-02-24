//! Intermediate Representation (IR) definitions for shell execution

use std::fmt;

use crate::tokens::Token;

/// IR instruction types
#[derive(Debug, Clone)]
pub enum IrInstruction {
    /// Execute a simple command
    ExecuteCommand {
        name: String,
        args: Vec<String>,
        tokens: Vec<Token>,
    },
    
    /// Set environment variable
    SetVariable {
        name: String,
        value: String,
        token: Token,
    },
    
    /// Create pipeline
    CreatePipeline {
        commands: Vec<Box<IrInstruction>>,
        tokens: Vec<Token>,
    },
    
    /// Conditional execution (AND)
    ConditionalAnd {
        condition: Box<IrInstruction>,
        body: Box<IrInstruction>,
        tokens: Vec<Token>,
    },
    
    /// Conditional execution (OR)
    ConditionalOr {
        condition: Box<IrInstruction>,
        body: Box<IrInstruction>,
        tokens: Vec<Token>,
    },
    
    /// Redirection
    Redirect {
        command: Box<IrInstruction>,
        redirect_type: RedirectType,
        target: String,
        fd: Option<i32>,
        token: Token,
    },
    
    /// Background execution
    Background {
        command: Box<IrInstruction>,
        token: Token,
    },
    
    /// Compound block
    CompoundBlock {
        instructions: Vec<Box<IrInstruction>>,
        tokens: Vec<Token>,
    },
    
    /// If statement
    IfStatement {
        condition: Box<IrInstruction>,
        then_branch: Vec<Box<IrInstruction>>,
        else_branch: Option<Vec<Box<IrInstruction>>>,
        elif_branches: Vec<(Box<IrInstruction>, Vec<Box<IrInstruction>>)>,
        tokens: Vec<Token>,
    },
    
    /// While loop
    WhileLoop {
        condition: Box<IrInstruction>,
        body: Vec<Box<IrInstruction>>,
        tokens: Vec<Token>,
    },
    
    /// Until loop
    UntilLoop {
        condition: Box<IrInstruction>,
        body: Vec<Box<IrInstruction>>,
        tokens: Vec<Token>,
    },
    
    /// For loop
    ForLoop {
        variable: String,
        items: Vec<String>,
        body: Vec<Box<IrInstruction>>,
        tokens: Vec<Token>,
    },
    
    /// Function definition
    DefineFunction {
        name: String,
        body: Vec<Box<IrInstruction>>,
        tokens: Vec<Token>,
    },
    
    /// Call function
    CallFunction {
        name: String,
        args: Vec<String>,
        tokens: Vec<Token>,
    },
    
    /// Subshell execution
    Subshell {
        instructions: Vec<Box<IrInstruction>>,
        tokens: Vec<Token>,
    },
    
    /// Command substitution
    CommandSubstitution {
        command: Box<IrInstruction>,
        backticks: bool,
        tokens: Vec<Token>,
    },
    
    /// No operation
    Nop,
    
    /// Error instruction
    Error {
        message: String,
        token: Token,
    },
}

/// Redirection types (same as AST for now)
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

impl fmt::Display for IrInstruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IrInstruction::ExecuteCommand { name, args, .. } => {
                write!(f, "ExecuteCommand({}", name)?;
                for arg in args {
                    write!(f, " {}", arg)?;
                }
                write!(f, ")")
            }
            IrInstruction::SetVariable { name, value, .. } => {
                write!(f, "SetVariable({}={})", name, value)
            }
            IrInstruction::CreatePipeline { commands, .. } => {
                write!(f, "CreatePipeline(")?;
                for (i, cmd) in commands.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{}", cmd)?;
                }
                write!(f, ")")
            }
            IrInstruction::ConditionalAnd { condition, body, .. } => {
                write!(f, "ConditionalAnd({} && {})", condition, body)
            }
            IrInstruction::ConditionalOr { condition, body, .. } => {
                write!(f, "ConditionalOr({} || {})", condition, body)
            }
            IrInstruction::Redirect { command, redirect_type, target, fd, .. } => {
                let fd_str = if let Some(fd) = fd {
                    format!("{}", fd)
                } else {
                    String::new()
                };
                write!(f, "Redirect({} {} {}{})", command, 
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
            IrInstruction::Background { command, .. } => {
                write!(f, "Background({} &)", command)
            }
            IrInstruction::CompoundBlock { instructions, .. } => {
                write!(f, "CompoundBlock{{")?;
                for instr in instructions {
                    write!(f, " {};", instr)?;
                }
                write!(f, " }}")
            }
            IrInstruction::IfStatement { condition, then_branch, else_branch, elif_branches, .. } => {
                write!(f, "IfStatement(if {}; then", condition)?;
                for instr in then_branch {
                    write!(f, " {};", instr)?;
                }
                for (cond, body) in elif_branches {
                    write!(f, " elif {}; then", cond)?;
                    for instr in body {
                        write!(f, " {};", instr)?;
                    }
                }
                if let Some(else_instrs) = else_branch {
                    write!(f, " else")?;
                    for instr in else_instrs {
                        write!(f, " {};", instr)?;
                    }
                }
                write!(f, " fi)")
            }
            IrInstruction::WhileLoop { condition, body, .. } => {
                write!(f, "WhileLoop(while {}; do", condition)?;
                for instr in body {
                    write!(f, " {};", instr)?;
                }
                write!(f, " done)")
            }
            IrInstruction::UntilLoop { condition, body, .. } => {
                write!(f, "UntilLoop(until {}; do", condition)?;
                for instr in body {
                    write!(f, " {};", instr)?;
                }
                write!(f, " done)")
            }
            IrInstruction::ForLoop { variable, items, body, .. } => {
                write!(f, "ForLoop(for {} in", variable)?;
                for item in items {
                    write!(f, " {}", item)?;
                }
                write!(f, "; do")?;
                for instr in body {
                    write!(f, " {};", instr)?;
                }
                write!(f, " done)")
            }
            IrInstruction::DefineFunction { name, body, .. } => {
                write!(f, "DefineFunction({}() {{", name)?;
                for instr in body {
                    write!(f, " {};", instr)?;
                }
                write!(f, " }})")
            }
            IrInstruction::CallFunction { name, args, .. } => {
                write!(f, "CallFunction({}", name)?;
                for arg in args {
                    write!(f, " {}", arg)?;
                }
                write!(f, ")")
            }
            IrInstruction::Subshell { instructions, .. } => {
                write!(f, "Subshell((")?;
                for instr in instructions {
                    write!(f, " {};", instr)?;
                }
                write!(f, "))")
            }
            IrInstruction::CommandSubstitution { command, backticks, .. } => {
                if *backticks {
                    write!(f, "CommandSubstitution(`{}`)", command)
                } else {
                    write!(f, "CommandSubstitution($({}))", command)
                }
            }
            IrInstruction::Nop => write!(f, "Nop"),
            IrInstruction::Error { message, .. } => write!(f, "Error({})", message),
        }
    }
}

/// IR program (sequence of instructions)
#[derive(Debug, Clone)]
pub struct IrProgram {
    pub instructions: Vec<Box<IrInstruction>>,
    pub functions: Vec<(String, Vec<Box<IrInstruction>>)>,
}

impl IrProgram {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            functions: Vec::new(),
        }
    }
    
    pub fn add_instruction(&mut self, instruction: IrInstruction) {
        self.instructions.push(Box::new(instruction));
    }
    
    pub fn add_function(&mut self, name: String, body: Vec<Box<IrInstruction>>) {
        self.functions.push((name, body));
    }
}

impl fmt::Display for IrProgram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IR Program:\n")?;
        
        // Functions
        if !self.functions.is_empty() {
            write!(f, "Functions:\n")?;
            for (name, body) in &self.functions {
                write!(f, "  {}() {{\n", name)?;
                for instr in body {
                    write!(f, "    {};\n", instr)?;
                }
                write!(f, "  }}\n")?;
            }
        }
        
        // Main instructions
        write!(f, "Main:\n")?;
        for instr in &self.instructions {
            write!(f, "  {};\n", instr)?;
        }
        
        Ok(())
    }
}