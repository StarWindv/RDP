//! rs-dash-pro library crate
//! A POSIX-compatible shell implementation in Rust with modern SSA architecture

pub mod modules;

// Re-export commonly used types from modules
pub use modules::{
    tokens::{Token, TokenType},
    lexer::Lexer,
    ast::{AstNode, AndOrOperator, CommandSeparator, ParseError, RedirectType, CaseClause, ParameterOperation},
    parser::Parser,
    env::ShellEnv,
    ssa_ir::{Function, ValueId, BasicBlockId, ValueType, Instruction, CmpOp, IrBuilder},
    ssa_ir_generator::SsaIrGenerator,
    ssa_executor::SsaExecutor,
    builtins::{Builtins, BuiltinCommand, BuiltinRegistry},
};