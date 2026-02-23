//! Intermediate Representation for shell execution
//! This module provides the new SSA-based IR system

// Re-export SSA IR types for convenience
pub use crate::ssa_ir::{
    ValueId, ValueType, Value,
    Instruction, RedirectMode, CmpOp,
    BasicBlockId, BasicBlock,
    Function, IrBuilder,
};

/// IR program (collection of functions)
#[derive(Debug, Clone)]
pub struct IrProgram {
    pub functions: Vec<Function>,
    pub main_function: Option<String>,
}

impl IrProgram {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            main_function: None,
        }
    }
    
    pub fn add_function(&mut self, func: Function) {
        if func.name == "main" {
            self.main_function = Some("main".to_string());
        }
        self.functions.push(func);
    }
    
    pub fn get_main_function(&self) -> Option<&Function> {
        self.functions.iter().find(|f| f.name == "main")
    }
}

impl std::fmt::Display for IrProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "IR Program:")?;
        for func in &self.functions {
            writeln!(f, "{}", func)?;
        }
        Ok(())
    }
}