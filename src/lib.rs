//! rs-dash-pro library crate
//! A POSIX-compatible shell implementation in Rust with modern SSA architecture

pub mod modules;

// Re-export commonly used types from modules
pub use modules::{
    tokens::{Token, TokenType},
    lexer::Lexer,
    ast::AstNode,
    parser::Parser,
    env::ShellEnv,
    ssa_ir::{Function, ValueId, BasicBlockId, Instruction},
    ssa_ir_generator::SsaIrGenerator,
    ssa_executor::SsaExecutor,
    builtins::Builtins,
};

// For backward compatibility
pub use modules as tokens;
pub use modules as lexer;
pub use modules as ast;
pub use modules as parser;
pub use modules as ssa_ir;
pub use modules as ssa_ir_generator;
pub use modules as ssa_executor;
pub use modules as builtins;
pub use modules as env;