//! IR module for intermediate representation

pub mod ir;
mod generator;

pub use generator::IrGenerator;
pub use ir::{IrInstruction, IrProgram, RedirectType};