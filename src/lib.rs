//! rs-dash-pro library crate
//! A POSIX-compatible shell implementation in Rust with modern SSA architecture

pub mod modules;

// Re-export commonly used types from modules
pub use modules::{
    ast::{
        AndOrOperator, AstNode, CaseClause, CommandSeparator, ParameterOperation, ParseError,
        RedirectType,
    },
    builtins::{BuiltinRegistry, Builtins},
    env::ShellEnv,
    lexer::Lexer,
    parser::Parser,
    ssa_executor::SsaExecutor,
    ssa_ir::{BasicBlockId, CmpOp, Function, Instruction, IrBuilder, ValueId, ValueType},
    ssa_ir_generator::SsaIrGenerator,
    tokens::{Token, TokenType},
};
