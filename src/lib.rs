//! rs-dash-pro library crate
//! A POSIX-compatible shell implementation in Rust with modern SSA architecture

pub mod tokens;
pub mod lexer;
pub mod enhanced_lexer;
pub mod ast;
pub mod parser;
pub mod ssa_ir;
pub mod ssa_ir_generator;
pub mod ssa_executor;
pub mod builtins;
pub mod env;

#[cfg(test)]
mod enhanced_lexer_tests;

// Re-export commonly used types
pub use tokens::{Token, TokenType};
pub use lexer::Lexer;
pub use enhanced_lexer::EnhancedLexer;
pub use ast::AstNode;
pub use parser::Parser;
pub use ssa_ir::{Function, ValueId, BasicBlockId, Instruction};
pub use ssa_ir_generator::SsaIrGenerator;
pub use ssa_executor::SsaExecutor;
pub use builtins::Builtins;
pub use env::ShellEnv;