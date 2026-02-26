//! rs-dash-pro modules
//! Organized by functional components following POSIX Shell architecture

// Core modules
pub mod tokens;
pub mod lexer;
pub mod ast;
pub mod parser;
pub mod env;

// SSA architecture modules
pub mod ssa_ir;
pub mod ssa_ir_generator;
pub mod ssa_executor;
pub mod optimize;

// Builtins modules
pub mod builtins;

// Shell components
pub mod expansion;
pub mod redirection;
pub mod pipeline;
pub mod control;
pub mod functions;
pub mod arithmetic;
pub mod param_expand;
pub mod subshell;
pub mod process_substitution;
pub mod job_control;
pub mod job_control_enhanced;
pub mod options;
pub mod options_enhanced;
pub mod variables;
pub mod here_doc;

// Re-export commonly used types
pub use tokens::{Token, TokenType};
pub use lexer::Lexer;
pub use ast::AstNode;
pub use parser::Parser;
pub use env::ShellEnv;
pub use ssa_ir::{Function, ValueId, BasicBlockId, Instruction};
pub use ssa_ir_generator::SsaIrGenerator;
pub use ssa_executor::SsaExecutor;
pub use builtins::Builtins;

