//! rs-dash-pro modules
//! Organized by functional components following POSIX Shell architecture

// Core modules
pub mod ast;
pub mod env;
pub mod lexer;
pub mod parser;
pub mod tokens;

// SSA architecture modules
pub mod optimize;
pub mod ssa_executor;
pub mod ssa_ir;
pub mod ssa_ir_generator;

// Builtins modules
pub mod builtins;

// Shell components
pub mod arithmetic;
pub mod control;
pub mod expansion;
pub mod functions;
pub mod here_doc;
pub mod job_control;
pub mod job_control_enhanced;
pub mod job_control_safe;
pub mod options;
pub mod options_enhanced;
pub mod param_expand;
pub mod pipeline;
pub mod process_substitution;
pub mod redirection;
pub mod subshell;
pub mod variables;

// Re-export commonly used types
pub use ast::AstNode;
pub use builtins::Builtins;
pub use env::ShellEnv;
pub use lexer::Lexer;
pub use parser::Parser;
pub use ssa_executor::SsaExecutor;
pub use ssa_ir::{BasicBlockId, Function, Instruction, ValueId};
pub use ssa_ir_generator::SsaIrGenerator;
pub use tokens::{Token, TokenType};
