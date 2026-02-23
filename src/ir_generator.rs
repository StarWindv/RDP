//! Legacy IR Generator (kept for compatibility)
//! The new SSA IR generator is in ssa_ir_generator.rs

use crate::ast::{AstNode, RedirectType};
use crate::ssa_ir::{Function, IrBuilder};

/// Legacy IR Generator (deprecated)
pub struct IrGenerator;

impl IrGenerator {
    /// Convert AST to legacy IR (deprecated)
    pub fn generate(&self, ast: AstNode) -> Function {
        // For compatibility, create a simple function
        let mut builder = IrBuilder::new();
        builder.begin_function("legacy_main".to_string(), Vec::new());
        
        // TODO: Implement legacy IR generation if needed
        // For now, just return an empty function
        
        builder.end_function().unwrap_or_else(|| Function::new("legacy_main".to_string(), Vec::new()))
    }
}