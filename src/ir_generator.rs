//! IR Generator: Converts AST to IR

use crate::ast::{AstNode, RedirectType};
use crate::ir::{IrInstruction, IrProgram, RedirectType as IrRedirectType};

/// IR Generator
pub struct IrGenerator;

impl IrGenerator {
    /// Convert AST to IR
    pub fn generate(&self, ast: AstNode) -> IrProgram {
        let mut program = IrProgram::new();
        if let Some(instruction) = self.generate_node(ast, &mut program) {
            program.add_instruction(*instruction);
        }
        program
    }
    
    /// Generate IR for a single AST node
    fn generate_node(&self, node: AstNode, program: &mut IrProgram) -> Option<Box<IrInstruction>> {
        match node {
            AstNode::SimpleCommand { name, args, tokens } => {
                Some(Box::new(IrInstruction::ExecuteCommand {
                    name,
                    args,
                    tokens,
                }))
            }
            
            AstNode::Assignment { name, value, token } => {
                Some(Box::new(IrInstruction::SetVariable {
                    name,
                    value,
                    token,
                }))
            }
            
            AstNode::Pipeline { commands, tokens } => {
                let mut ir_commands = Vec::new();
                for cmd in commands {
                    if let Some(ir_cmd) = self.generate_node(*cmd, program) {
                        ir_commands.push(ir_cmd);
                    }
                }
                
                Some(Box::new(IrInstruction::CreatePipeline {
                    commands: ir_commands,
                    tokens,
                }))
            }
            
            AstNode::CommandList { commands, separator: _, tokens } => {
                let mut ir_instructions = Vec::new();
                for cmd in commands {
                    if let Some(ir_instr) = self.generate_node(*cmd, program) {
                        ir_instructions.push(ir_instr);
                    }
                }
                
                if ir_instructions.len() == 1 {
                    Some(ir_instructions.remove(0))
                } else {
                    Some(Box::new(IrInstruction::CompoundBlock {
                        instructions: ir_instructions,
                        tokens,
                    }))
                }
            }
            
            AstNode::LogicalAnd { left, right, tokens } => {
                let left_ir = self.generate_node(*left, program);
                let right_ir = self.generate_node(*right, program);
                
                if let (Some(left), Some(right)) = (left_ir, right_ir) {
                    Some(Box::new(IrInstruction::ConditionalAnd {
                        condition: left,
                        body: right,
                        tokens,
                    }))
                } else {
                    None
                }
            }
            
            AstNode::LogicalOr { left, right, tokens } => {
                let left_ir = self.generate_node(*left, program);
                let right_ir = self.generate_node(*right, program);
                
                if let (Some(left), Some(right)) = (left_ir, right_ir) {
                    Some(Box::new(IrInstruction::ConditionalOr {
                        condition: left,
                        body: right,
                        tokens,
                    }))
                } else {
                    None
                }
            }
            
            AstNode::Redirection { command, redirect_type, target, fd, token } => {
                let command_ir = self.generate_node(*command, program);
                
                if let Some(command) = command_ir {
                    let ir_redirect_type = match redirect_type {
                        RedirectType::Input => IrRedirectType::Input,
                        RedirectType::Output => IrRedirectType::Output,
                        RedirectType::Append => IrRedirectType::Append,
                        RedirectType::HereDoc => IrRedirectType::HereDoc,
                        RedirectType::HereDocStrip => IrRedirectType::HereDocStrip,
                        RedirectType::DupInput => IrRedirectType::DupInput,
                        RedirectType::DupOutput => IrRedirectType::DupOutput,
                        RedirectType::ReadWrite => IrRedirectType::ReadWrite,
                        RedirectType::Clobber => IrRedirectType::Clobber,
                    };
                    
                    Some(Box::new(IrInstruction::Redirect {
                        command,
                        redirect_type: ir_redirect_type,
                        target,
                        fd,
                        token,
                    }))
                } else {
                    None
                }
            }
            
            AstNode::Background { command, token } => {
                let command_ir = self.generate_node(*command, program);
                
                if let Some(command) = command_ir {
                    Some(Box::new(IrInstruction::Background {
                        command,
                        token,
                    }))
                } else {
                    None
                }
            }
            
            AstNode::CompoundCommand { commands, tokens } => {
                let mut ir_instructions = Vec::new();
                for cmd in commands {
                    if let Some(ir_instr) = self.generate_node(*cmd, program) {
                        ir_instructions.push(ir_instr);
                    }
                }
                
                Some(Box::new(IrInstruction::CompoundBlock {
                    instructions: ir_instructions,
                    tokens,
                }))
            }
            
            AstNode::IfStatement { condition, then_branch, else_branch, elif_branches, tokens } => {
                let condition_ir = self.generate_node(*condition, program);
                
                if let Some(condition) = condition_ir {
                    let mut then_ir = Vec::new();
                    for cmd in then_branch {
                        if let Some(ir_instr) = self.generate_node(*cmd, program) {
                            then_ir.push(ir_instr);
                        }
                    }
                    
                    let mut elif_ir = Vec::new();
                    for (elif_cond, elif_body) in elif_branches {
                        if let Some(cond_ir) = self.generate_node(*elif_cond, program) {
                            let mut body_ir = Vec::new();
                            for cmd in elif_body {
                                if let Some(ir_instr) = self.generate_node(*cmd, program) {
                                    body_ir.push(ir_instr);
                                }
                            }
                            elif_ir.push((cond_ir, body_ir));
                        }
                    }
                    
                    let else_ir = else_branch.map(|else_cmds| {
                        let mut ir = Vec::new();
                        for cmd in else_cmds {
                            if let Some(ir_instr) = self.generate_node(*cmd, program) {
                                ir.push(ir_instr);
                            }
                        }
                        ir
                    });
                    
                    Some(Box::new(IrInstruction::IfStatement {
                        condition,
                        then_branch: then_ir,
                        else_branch: else_ir,
                        elif_branches: elif_ir,
                        tokens,
                    }))
                } else {
                    None
                }
            }
            
            AstNode::WhileLoop { condition, body, tokens } => {
                let condition_ir = self.generate_node(*condition, program);
                
                if let Some(condition) = condition_ir {
                    let mut body_ir = Vec::new();
                    for cmd in body {
                        if let Some(ir_instr) = self.generate_node(*cmd, program) {
                            body_ir.push(ir_instr);
                        }
                    }
                    
                    Some(Box::new(IrInstruction::WhileLoop {
                        condition,
                        body: body_ir,
                        tokens,
                    }))
                } else {
                    None
                }
            }
            
            AstNode::UntilLoop { condition, body, tokens } => {
                let condition_ir = self.generate_node(*condition, program);
                
                if let Some(condition) = condition_ir {
                    let mut body_ir = Vec::new();
                    for cmd in body {
                        if let Some(ir_instr) = self.generate_node(*cmd, program) {
                            body_ir.push(ir_instr);
                        }
                    }
                    
                    Some(Box::new(IrInstruction::UntilLoop {
                        condition,
                        body: body_ir,
                        tokens,
                    }))
                } else {
                    None
                }
            }
            
            AstNode::ForLoop { variable, items, body, tokens } => {
                let mut body_ir = Vec::new();
                for cmd in body {
                    if let Some(ir_instr) = self.generate_node(*cmd, program) {
                        body_ir.push(ir_instr);
                    }
                }
                
                Some(Box::new(IrInstruction::ForLoop {
                    variable,
                    items,
                    body: body_ir,
                    tokens,
                }))
            }
            
            AstNode::FunctionDefinition { name, body, tokens } => {
                let mut body_ir = Vec::new();
                for cmd in body {
                    if let Some(ir_instr) = self.generate_node(*cmd, program) {
                        body_ir.push(ir_instr);
                    }
                }
                
                program.add_function(name.clone(), body_ir.clone());
                
                Some(Box::new(IrInstruction::DefineFunction {
                    name,
                    body: body_ir,
                    tokens,
                }))
            }
            
            AstNode::Subshell { commands, tokens } => {
                let mut instructions = Vec::new();
                for cmd in commands {
                    if let Some(ir_instr) = self.generate_node(*cmd, program) {
                        instructions.push(ir_instr);
                    }
                }
                
                Some(Box::new(IrInstruction::Subshell {
                    instructions,
                    tokens,
                }))
            }
            
            AstNode::CommandSubstitution { command, backticks, tokens } => {
                let command_ir = self.generate_node(*command, program);
                
                if let Some(command) = command_ir {
                    Some(Box::new(IrInstruction::CommandSubstitution {
                        command,
                        backticks,
                        tokens,
                    }))
                } else {
                    None
                }
            }
            
            AstNode::NullCommand => {
                Some(Box::new(IrInstruction::Nop))
            }
            
            AstNode::Error { message, token } => {
                Some(Box::new(IrInstruction::Error {
                    message,
                    token,
                }))
            }
        }
    }
}