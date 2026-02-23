//! SSA IR Generator: Converts AST to SSA IR
//! This implements the complete Lexer -> Parser -> IRGenerator[SSA] architecture

use crate::ast::{AstNode, AndOrOperator, CommandSeparator, RedirectType, CaseClause, ParameterOperation};
use crate::ssa_ir::{
    IrBuilder, Function, BasicBlockId, ValueId, ValueType,
    Instruction, RedirectMode, CmpOp,
};

/// SSA IR Generator
pub struct SsaIrGenerator {
    builder: IrBuilder,
    current_function: Option<Function>,
    current_block: Option<BasicBlockId>,
    value_counter: usize,
}

impl SsaIrGenerator {
    /// Create a new SSA IR generator
    pub fn new() -> Self {
        Self {
            builder: IrBuilder::new(),
            current_function: None,
            current_block: None,
            value_counter: 1, // Start from 1
        }
    }
    
    /// Generate SSA IR from AST
    pub fn generate(&mut self, ast: AstNode) -> Function {
        // Create main function
        self.builder.begin_function("main".to_string(), Vec::new());
        self.current_block = Some(BasicBlockId(0));
        
        // Generate IR for the AST
        let exit_status = self.generate_node(ast);
        
        // Return exit status
        if let Some(func) = self.builder.get_current_function() {
            if let Some(block_id) = self.current_block {
                func.add_instruction(block_id, Instruction::Return(exit_status));
            }
        }
        
        // Finish function
        self.builder.end_function().expect("Function should exist")
    }
    
    /// Generate SSA IR for a single AST node
    fn generate_node(&mut self, node: AstNode) -> ValueId {
        match node {
            // ============================================
            // Simple Commands
            // ============================================
            AstNode::SimpleCommand { name, args, .. } => {
                self.generate_simple_command(&name, &args)
            }
            
            AstNode::Assignment { name, value, .. } => {
                self.generate_assignment(&name, &value)
            }
            
            // ============================================
            // Pipelines and Lists
            // ============================================
            AstNode::Pipeline { commands, .. } => {
                self.generate_pipeline(commands)
            }
            
            AstNode::AndOrList { commands, operators, .. } => {
                self.generate_and_or_list(commands, &operators)
            }
            
            AstNode::LogicalAnd { left, right, .. } => {
                self.generate_logical_and(*left, *right)
            }
            
            AstNode::LogicalOr { left, right, .. } => {
                self.generate_logical_or(*left, *right)
            }
            
            AstNode::CommandList { commands, separators, .. } => {
                self.generate_command_list(commands, &separators)
            }
            
            // ============================================
            // Compound Commands
            // ============================================
            AstNode::CompoundCommand { commands, .. } => {
                self.generate_compound_command(commands)
            }
            
            AstNode::Subshell { commands, .. } => {
                self.generate_subshell(commands)
            }
            
            // ============================================
            // Conditional Constructs
            // ============================================
            AstNode::IfStatement { condition, then_branch, else_branch, elif_branches, .. } => {
                self.generate_if_statement(*condition, then_branch, else_branch, elif_branches)
            }
            
            AstNode::CaseStatement { word, cases, .. } => {
                self.generate_case_statement(&word, cases)
            }
            
            // ============================================
            // Loop Constructs
            // ============================================
            AstNode::WhileLoop { condition, body, .. } => {
                self.generate_while_loop(*condition, body)
            }
            
            AstNode::UntilLoop { condition, body, .. } => {
                self.generate_until_loop(*condition, body)
            }
            
            AstNode::ForLoop { variable, items, body, .. } => {
                self.generate_for_loop(&variable, items, body)
            }
            
            AstNode::SelectStatement { variable, items, body, .. } => {
                self.generate_select_statement(&variable, items, body)
            }
            
            // ============================================
            // Function Definitions
            // ============================================
            AstNode::FunctionDefinition { name, body, .. } => {
                self.generate_function_definition(&name, body)
            }
            
            // ============================================
            // Redirections
            // ============================================
            AstNode::Redirection { command, redirect_type, target, fd, .. } => {
                self.generate_redirection(*command, redirect_type, &target, fd)
            }
            
            AstNode::Background { command, .. } => {
                self.generate_background(*command)
            }
            
            // ============================================
            // Command Substitution and Parameter Expansion
            // ============================================
            AstNode::CommandSubstitution { command, backticks, .. } => {
                self.generate_command_substitution(*command, backticks)
            }
            
            AstNode::ParameterExpansion { parameter, operation, .. } => {
                self.generate_parameter_expansion(&parameter, operation)
            }
            
            // ============================================
            // Special Nodes
            // ============================================
            AstNode::NullCommand => {
                self.generate_null_command()
            }
            
            AstNode::Error { message, token } => {
                self.generate_error(&message, token)
            }
        }
    }
    
    // ============================================
    // Generation Methods
    // ============================================
    
    fn generate_simple_command(&mut self, name: &str, args: &[String]) -> ValueId {
        // Create value for command result
        let result = self.create_value(ValueType::ExitStatus);
        
        // Convert arguments to values
        let arg_values: Vec<ValueId> = args.iter()
            .map(|arg| {
                let val = self.create_value(ValueType::String);
                self.add_instruction(Instruction::ConstString(arg.clone(), val));
                val
            })
            .collect();
        
        // Generate call instruction
        self.add_instruction(Instruction::CallExternal(
            name.to_string(),
            arg_values,
            result,
        ));
        
        result
    }
    
    fn generate_assignment(&mut self, name: &str, value: &str) -> ValueId {
        // Create variable
        let var = self.create_value_with_name(ValueType::String, name.to_string());
        self.add_instruction(Instruction::AllocVar(name.to_string(), var));
        
        // Create value
        let val = self.create_value(ValueType::String);
        self.add_instruction(Instruction::ConstString(value.to_string(), val));
        
        // Store value
        self.add_instruction(Instruction::Store(var, val));
        
        // Return success status
        let result = self.create_value(ValueType::ExitStatus);
        self.add_instruction(Instruction::ConstInt(0, result));
        result
    }
    
    fn generate_pipeline(&mut self, commands: Vec<Box<AstNode>>) -> ValueId {
        if commands.is_empty() {
            return self.generate_null_command();
        }
        
        // For now, execute commands sequentially
        // TODO: Implement proper pipeline with pipes
        let mut last_status = self.create_value(ValueType::ExitStatus);
        self.add_instruction(Instruction::ConstInt(0, last_status));
        
        for cmd in commands {
            let status = self.generate_node(*cmd);
            last_status = status;
        }
        
        last_status
    }
    
    fn generate_and_or_list(&mut self, commands: Vec<Box<AstNode>>, operators: &[AndOrOperator]) -> ValueId {
        if commands.is_empty() {
            return self.generate_null_command();
        }
        
        let mut current_status = self.generate_node(*commands[0].clone());
        
        for (i, cmd) in commands.iter().skip(1).enumerate() {
            let operator = &operators[i];
            
            // Create blocks for short-circuit evaluation
            let skip_block = self.create_block();
            let eval_block = self.create_block();
            let merge_block = self.create_block();
            
            // Check current status
            let cond = self.create_value(ValueType::Boolean);
            match operator {
                AndOrOperator::AndIf => {
                    // AND: skip if previous command failed
                    self.add_instruction(Instruction::Cmp(
                        current_status,
                        self.create_const_int(0),
                        CmpOp::Eq,
                        cond,
                    ));
                    self.add_instruction(Instruction::Branch(
                        cond,
                        eval_block,  // If success (status == 0), evaluate next
                        skip_block,  // If failure, skip to end
                    ));
                }
                AndOrOperator::OrIf => {
                    // OR: skip if previous command succeeded
                    self.add_instruction(Instruction::Cmp(
                        current_status,
                        self.create_const_int(0),
                        CmpOp::Ne,
                        cond,
                    ));
                    self.add_instruction(Instruction::Branch(
                        cond,
                        eval_block,  // If failure (status != 0), evaluate next
                        skip_block,  // If success, skip to end
                    ));
                }
            }
            
            // Eval block: evaluate next command
            self.set_current_block(eval_block);
            let cmd_status = self.generate_node(*cmd.clone());
            
            // Jump to merge block
            self.add_instruction(Instruction::Jump(merge_block));
            
            // Skip block: keep current status
            self.set_current_block(skip_block);
            self.add_instruction(Instruction::Jump(merge_block));
            
            // Merge block: phi node to select result
            self.set_current_block(merge_block);
            let phi_result = self.create_value(ValueType::ExitStatus);
            self.add_instruction(Instruction::Phi(
                vec![
                    (eval_block, cmd_status),
                    (skip_block, current_status),
                ],
                phi_result,
            ));
            
            current_status = phi_result;
        }
        
        current_status
    }
    
    fn generate_logical_and(&mut self, left: AstNode, right: AstNode) -> ValueId {
        self.generate_and_or_list(
            vec![Box::new(left), Box::new(right)],
            &[AndOrOperator::AndIf],
        )
    }
    
    fn generate_logical_or(&mut self, left: AstNode, right: AstNode) -> ValueId {
        self.generate_and_or_list(
            vec![Box::new(left), Box::new(right)],
            &[AndOrOperator::OrIf],
        )
    }
    
    fn generate_command_list(&mut self, commands: Vec<Box<AstNode>>, separators: &[CommandSeparator]) -> ValueId {
        if commands.is_empty() {
            return self.generate_null_command();
        }
        
        let mut last_status = self.create_value(ValueType::ExitStatus);
        self.add_instruction(Instruction::ConstInt(0, last_status));
        
        for (i, cmd) in commands.iter().enumerate() {
            if i > 0 {
                match separators[i-1] {
                    CommandSeparator::Ampersand => {
                        // Background execution - for now, just execute in foreground
                        // TODO: Implement proper background execution
                    }
                    _ => {
                        // Sequential execution
                    }
                }
            }
            
            let status = self.generate_node(*cmd.clone());
            last_status = status;
        }
        
        last_status
    }
    
    fn generate_compound_command(&mut self, commands: Vec<Box<AstNode>>) -> ValueId {
        self.generate_command_list(commands, &[])
    }
    
    fn generate_subshell(&mut self, commands: Vec<Box<AstNode>>) -> ValueId {
        // For now, treat subshell same as compound command
        // TODO: Implement proper subshell with separate environment
        self.generate_compound_command(commands)
    }
    
    fn generate_if_statement(
        &mut self,
        condition: AstNode,
        then_branch: Vec<Box<AstNode>>,
        else_branch: Option<Vec<Box<AstNode>>>,
        elif_branches: Vec<(Box<AstNode>, Vec<Box<AstNode>>)>,
    ) -> ValueId {
        let then_block = self.create_block_with_label("then".to_string());
        let else_block = self.create_block_with_label("else".to_string());
        let merge_block = self.create_block_with_label("endif".to_string());
        
        // Generate condition
        let cond_status = self.generate_node(condition);
        
        // Branch based on condition
        let cond = self.create_value(ValueType::Boolean);
        self.add_instruction(Instruction::Cmp(
            cond_status,
            self.create_const_int(0),
            CmpOp::Eq,
            cond,
        ));
        self.add_instruction(Instruction::Branch(
            cond,
            then_block,
            else_block,
        ));
        
        // Then branch
        self.set_current_block(then_block);
        let then_result = self.generate_compound_command(then_branch);
        self.add_instruction(Instruction::Jump(merge_block));
        
        // Else branch
        self.set_current_block(else_block);
        let else_result = if let Some(else_cmds) = else_branch {
            self.generate_compound_command(else_cmds)
        } else {
            self.create_const_int(0)
        };
        self.add_instruction(Instruction::Jump(merge_block));
        
        // Merge block
        self.set_current_block(merge_block);
        let result = self.create_value(ValueType::ExitStatus);
        self.add_instruction(Instruction::Phi(
            vec![
                (then_block, then_result),
                (else_block, else_result),
            ],
            result,
        ));
        
        result
    }
    
    fn generate_case_statement(&mut self, word: &str, cases: Vec<CaseClause>) -> ValueId {
        // TODO: Implement case statement
        // For now, return success
        let result = self.create_value(ValueType::ExitStatus);
        self.add_instruction(Instruction::ConstInt(0, result));
        result
    }
    
    fn generate_while_loop(&mut self, condition: AstNode, body: Vec<Box<AstNode>>) -> ValueId {
        let cond_block = self.create_block_with_label("while_cond".to_string());
        let body_block = self.create_block_with_label("while_body".to_string());
        let exit_block = self.create_block_with_label("while_exit".to_string());
        
        // Jump to condition block
        self.add_instruction(Instruction::Jump(cond_block));
        
        // Condition block
        self.set_current_block(cond_block);
        let cond_status = self.generate_node(condition.clone());
        let cond = self.create_value(ValueType::Boolean);
        self.add_instruction(Instruction::Cmp(
            cond_status,
            self.create_const_int(0),
            CmpOp::Eq,
            cond,
        ));
        self.add_instruction(Instruction::Branch(
            cond,
            body_block,
            exit_block,
        ));
        
        // Body block
        self.set_current_block(body_block);
        self.generate_compound_command(body.clone());
        self.add_instruction(Instruction::Jump(cond_block));
        
        // Exit block
        self.set_current_block(exit_block);
        let result = self.create_value(ValueType::ExitStatus);
        self.add_instruction(Instruction::ConstInt(0, result));
        result
    }
    
    fn generate_until_loop(&mut self, condition: AstNode, body: Vec<Box<AstNode>>) -> ValueId {
        // Until is the opposite of while
        let cond_block = self.create_block_with_label("until_cond".to_string());
        let body_block = self.create_block_with_label("until_body".to_string());
        let exit_block = self.create_block_with_label("until_exit".to_string());
        
        // Jump to condition block
        self.add_instruction(Instruction::Jump(cond_block));
        
        // Condition block
        self.set_current_block(cond_block);
        let cond_status = self.generate_node(condition.clone());
        let cond = self.create_value(ValueType::Boolean);
        self.add_instruction(Instruction::Cmp(
            cond_status,
            self.create_const_int(0),
            CmpOp::Ne,
            cond,
        ));
        self.add_instruction(Instruction::Branch(
            cond,
            body_block,
            exit_block,
        ));
        
        // Body block
        self.set_current_block(body_block);
        self.generate_compound_command(body.clone());
        self.add_instruction(Instruction::Jump(cond_block));
        
        // Exit block
        self.set_current_block(exit_block);
        let result = self.create_value(ValueType::ExitStatus);
        self.add_instruction(Instruction::ConstInt(0, result));
        result
    }
    
    fn generate_for_loop(&mut self, variable: &str, items: Vec<String>, body: Vec<Box<AstNode>>) -> ValueId {
        // Create loop variable
        let var_val = self.create_value_with_name(ValueType::String, variable.to_string());
        self.add_instruction(Instruction::AllocVar(variable.to_string(), var_val));
        
        let loop_start = self.create_block_with_label("for_start".to_string());
        let loop_body = self.create_block_with_label("for_body".to_string());
        let loop_end = self.create_block_with_label("for_end".to_string());
        
        // Jump to start
        self.add_instruction(Instruction::Jump(loop_start));
        
        // Start block: initialize iterator
        self.set_current_block(loop_start);
        
        // For each item
        for item in items {
            // Set variable value
            let item_val = self.create_value(ValueType::String);
            self.add_instruction(Instruction::ConstString(item, item_val));
            self.add_instruction(Instruction::Store(var_val, item_val));
            
            // Execute body
            self.generate_compound_command(body.clone());
        }
        
        // Jump to end
        self.add_instruction(Instruction::Jump(loop_end));
        
        // End block
        self.set_current_block(loop_end);
        let result = self.create_value(ValueType::ExitStatus);
        self.add_instruction(Instruction::ConstInt(0, result));
        result
    }
    
    fn generate_select_statement(&mut self, variable: &str, items: Vec<String>, body: Vec<Box<AstNode>>) -> ValueId {
        // TODO: Implement select statement
        // For now, treat as for loop
        self.generate_for_loop(variable, items, body)
    }
    
    fn generate_function_definition(&mut self, name: &str, body: Vec<Box<AstNode>>) -> ValueId {
        // TODO: Implement function definition
        // For now, just return success
        let result = self.create_value(ValueType::ExitStatus);
        self.add_instruction(Instruction::ConstInt(0, result));
        result
    }
    
    fn generate_redirection(&mut self, command: AstNode, redirect_type: RedirectType, target: &str, fd: Option<i32>) -> ValueId {
        // Generate command first
        let cmd_result = self.generate_node(command);
        
        // TODO: Implement redirection
        // For now, just return command result
        cmd_result
    }
    
    fn generate_background(&mut self, command: AstNode) -> ValueId {
        // TODO: Implement background execution
        // For now, just execute in foreground
        self.generate_node(command)
    }
    
    fn generate_command_substitution(&mut self, command: AstNode, backticks: bool) -> ValueId {
        // TODO: Implement command substitution
        // For now, just execute the command
        self.generate_node(command)
    }
    
    fn generate_parameter_expansion(&mut self, parameter: &str, operation: Option<ParameterOperation>) -> ValueId {
        // TODO: Implement parameter expansion
        // For now, create a string value
        let result = self.create_value(ValueType::String);
        let expanded = match operation {
            Some(op) => {
                match op {
                    ParameterOperation::UseDefault(default) => format!("${{{}}}", default),
                    ParameterOperation::AssignDefault(default) => format!("${{:={}}}", default),
                    ParameterOperation::ErrorIfNull(msg) => format!("${{:?{}}}", msg),
                    ParameterOperation::UseAlternate(alt) => format!("${{:+{}}}", alt),
                    ParameterOperation::Length => format!("${{#{}}}", parameter),
                    ParameterOperation::RemoveSuffix(pattern) => format!("${{%{}}}", pattern),
                    ParameterOperation::RemoveLargestSuffix(pattern) => format!("${{%%{}}}", pattern),
                    ParameterOperation::RemovePrefix(pattern) => format!("${{#{}}}", pattern),
                    ParameterOperation::RemoveLargestPrefix(pattern) => format!("${{##{}}}", pattern),
                }
            }
            None => parameter.to_string(),
        };
        self.add_instruction(Instruction::ConstString(expanded, result));
        result
    }
    
    fn generate_null_command(&mut self) -> ValueId {
        let result = self.create_value(ValueType::ExitStatus);
        self.add_instruction(Instruction::ConstInt(0, result));
        result
    }
    
    fn generate_error(&mut self, message: &str, token: crate::tokens::Token) -> ValueId {
        let result = self.create_value(ValueType::ExitStatus);
        self.add_instruction(Instruction::Error(message.to_string(), token));
        self.add_instruction(Instruction::ConstInt(1, result));
        result
    }
    
    // ============================================
    // Helper Methods
    // ============================================
    
    fn create_value(&mut self, ty: ValueType) -> ValueId {
        if let Some(func) = self.builder.get_current_function() {
            func.create_value(ty)
        } else {
            panic!("No current function");
        }
    }
    
    fn create_value_with_name(&mut self, ty: ValueType, name: String) -> ValueId {
        if let Some(func) = self.builder.get_current_function() {
            func.create_value_with_name(ty, name)
        } else {
            panic!("No current function");
        }
    }
    
    fn create_block(&mut self) -> BasicBlockId {
        if let Some(func) = self.builder.get_current_function() {
            func.create_block()
        } else {
            panic!("No current function");
        }
    }
    
    fn create_block_with_label(&mut self, label: String) -> BasicBlockId {
        if let Some(func) = self.builder.get_current_function() {
            func.create_block_with_label(label)
        } else {
            panic!("No current function");
        }
    }
    
    fn set_current_block(&mut self, block_id: BasicBlockId) {
        self.current_block = Some(block_id);
        self.builder.set_current_block(block_id);
    }
    
    fn add_instruction(&mut self, instr: Instruction) {
        if let Some(block_id) = self.current_block {
            if let Some(func) = self.builder.get_current_function() {
                func.add_instruction(block_id, instr);
            }
        }
    }
    
    fn create_const_int(&mut self, value: i32) -> ValueId {
        let val = self.create_value(ValueType::Integer);
        self.add_instruction(Instruction::ConstInt(value, val));
        val
    }
}