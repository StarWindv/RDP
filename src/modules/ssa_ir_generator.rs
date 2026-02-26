//! SSA IR Generator: Converts AST to SSA IR
//! This implements the complete Lexer -> Parser -> IRGenerator[SSA] architecture

use crate::modules::ast::{AstNode, AndOrOperator, CommandSeparator, RedirectType, CaseClause, ParameterOperation};
use crate::modules::ssa_ir::{
    IrBuilder, Function, BasicBlockId, ValueId, ValueType,
    Instruction, CmpOp, ParamExpandOp,
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
            // Variable management commands
            // ============================================
            AstNode::Export { variables, .. } => {
                self.generate_export_command(&variables)
            }
            
            AstNode::Unset { variables, .. } => {
                self.generate_unset_command(&variables)
            }
            
            AstNode::Readonly { variables, .. } => {
                self.generate_readonly_command(&variables)
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
        
        // Convert arguments to values, expanding variables if needed
        let arg_values: Vec<ValueId> = args.iter()
            .map(|arg| {
                // Check if argument needs expansion
                if self.needs_expansion(arg) {
                    self.generate_argument_expansion(arg)
                } else {
                    // No expansion needed, create constant string
                    let val = self.create_value(ValueType::String);
                    self.add_instruction(Instruction::ConstString(arg.clone(), val));
                    val
                }
            })
            .collect();
        
        // Check if it's a built-in command
        // TODO: Actually check if it's a builtin
        // For now, we'll treat all commands as external and let the executor handle builtins
        // But we need to add builtin support in the executor
        
        // Generate call instruction
        self.add_instruction(Instruction::CallExternal(
            name.to_string(),
            arg_values,
            result,
        ));
        
        result
    }
    
    /// Check if a string needs expansion (contains $, `, etc.)
    fn needs_expansion(&self, s: &str) -> bool {
        // Check for variable expansion: $VAR, ${VAR}
        if s.contains('$') {
            return true;
        }
        
        // Check for command substitution: $(command) or `command`
        if s.contains("$(") || s.contains('`') {
            return true;
        }
        
        // Check for arithmetic expansion: $((expression))
        if s.contains("$((") {
            return true;
        }
        
        // Check for tilde expansion: ~, ~user
        if s.starts_with('~') {
            return true;
        }
        
        false
    }
    
    /// Generate expansion for an argument string
    fn generate_argument_expansion(&mut self, arg: &str) -> ValueId {
        // For now, implement a simple version that handles $VAR and ${VAR}
        // In a full implementation, we would need to parse the string and handle
        // mixed expansions like "prefix_${VAR}_suffix"
        
        // Check if it's a simple variable reference
        if arg.starts_with('$') {
            // Remove $ prefix
            let var_name = if arg.starts_with("${") && arg.ends_with('}') {
                // ${VAR} syntax
                &arg[2..arg.len()-1]
            } else if arg.starts_with('$') {
                // $VAR syntax
                &arg[1..]
            } else {
                arg
            };
            
            // Create parameter expansion instruction
            let param_val = self.create_value(ValueType::String);
            self.add_instruction(Instruction::ConstString(var_name.to_string(), param_val));
            
            let result = self.create_value(ValueType::String);
            self.add_instruction(Instruction::ParamExpand(param_val, crate::modules::ssa_ir::ParamExpandOp::Simple, result));
            
            return result;
        }
        
        // For now, if we can't handle the expansion, return as-is
        // TODO: Implement full expansion logic
        let val = self.create_value(ValueType::String);
        self.add_instruction(Instruction::ConstString(arg.to_string(), val));
        val
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
        
        // Single command - no pipe needed
        if commands.len() == 1 {
            return self.generate_node(*commands[0].clone());
        }
        
        // For multiple commands, we need to create a pipeline
        // We'll create a chain of processes connected by pipes
        
        // Create pipes for connecting commands
        // For N commands, we need N-1 pipes
        let mut pipes = Vec::new();
        for _i in 0..commands.len() - 1 {
            let read_fd = self.create_value(ValueType::FileDescriptor);
            let write_fd = self.create_value(ValueType::FileDescriptor);
            self.add_instruction(Instruction::CreatePipe(read_fd, write_fd));
            pipes.push((read_fd, write_fd));
        }
        
        // Execute each command in the pipeline
        let mut pids = Vec::new();
        let mut last_status = self.create_value(ValueType::ExitStatus);
        self.add_instruction(Instruction::ConstInt(0, last_status));
        
        for (i, cmd) in commands.iter().enumerate() {
            // Fork for each command in the pipeline
            let pid = self.create_value(ValueType::ProcessId);
            self.add_instruction(Instruction::Fork(pid));
            
            // Create blocks for parent and child
            let parent_block = self.create_block_with_label(format!("pipe_parent_{}", i));
            let child_block = self.create_block_with_label(format!("pipe_child_{}", i));
            
            // Branch based on fork result
            let zero_const = self.create_const_int(0);
            let is_child = self.create_value(ValueType::Boolean);
            self.add_instruction(Instruction::Cmp(pid, zero_const, CmpOp::Eq, is_child));
            self.add_instruction(Instruction::Branch(is_child, child_block, parent_block));
            
            // Child block: execute command with proper redirections
            self.set_current_block(child_block);
            
            // Set up pipe redirections
            if i > 0 {
                // Not first command: read from previous pipe
                let (prev_read, _) = &pipes[i - 1];
                // Redirect stdin from previous pipe's read end
                let dup_result = self.create_value(ValueType::FileDescriptor);
                self.add_instruction(Instruction::DupFd(*prev_read, 0, dup_result));
                self.add_instruction(Instruction::CloseFd(*prev_read));
            }
            
            if i < commands.len() - 1 {
                // Not last command: write to next pipe
                let (_, next_write) = &pipes[i];
                // Redirect stdout to next pipe's write end
                let dup_result = self.create_value(ValueType::FileDescriptor);
                self.add_instruction(Instruction::DupFd(*next_write, 1, dup_result));
                self.add_instruction(Instruction::CloseFd(*next_write));
            }
            
            // Close all pipe file descriptors in child
            for (read_fd, write_fd) in &pipes {
                if i == 0 || (i > 0 && read_fd != &pipes[i - 1].0) {
                    self.add_instruction(Instruction::CloseFd(*read_fd));
                }
                if i == commands.len() - 1 || (i < commands.len() - 1 && write_fd != &pipes[i].1) {
                    self.add_instruction(Instruction::CloseFd(*write_fd));
                }
            }
            
            // Execute the command
            let cmd_status = self.generate_node(*cmd.clone());
            self.add_instruction(Instruction::Exit(cmd_status));
            
            // Parent block: store PID and continue
            self.set_current_block(parent_block);
            pids.push(pid);
            
            // Close pipe ends that parent doesn't need
            if i > 0 {
                let (prev_read, _) = &pipes[i - 1];
                self.add_instruction(Instruction::CloseFd(*prev_read));
            }
            if i < commands.len() - 1 {
                let (_, next_write) = &pipes[i];
                self.add_instruction(Instruction::CloseFd(*next_write));
            }
            
            // Continue with next command or wait for all
            if i < commands.len() - 1 {
                // More commands to fork
                let next_block = self.create_block_with_label(format!("pipe_next_{}", i));
                self.add_instruction(Instruction::Jump(next_block));
                self.set_current_block(next_block);
            }
        }
        
        // Wait for all child processes
        let wait_block = self.create_block_with_label("pipe_wait".to_string());
        self.set_current_block(wait_block);
        
        // Wait for the last process (others will have been waited for as they exit)
        if let Some(last_pid) = pids.last() {
            let status = self.create_value(ValueType::ExitStatus);
            self.add_instruction(Instruction::Wait(*last_pid, status));
            last_status = status;
        }
        
        // Close any remaining pipe file descriptors
        for (read_fd, write_fd) in &pipes {
            self.add_instruction(Instruction::CloseFd(*read_fd));
            self.add_instruction(Instruction::CloseFd(*write_fd));
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
            let zero_const = self.create_const_int(0);
            match operator {
                AndOrOperator::AndIf => {
                    // AND: skip if previous command failed
                    self.add_instruction(Instruction::Cmp(
                        current_status,
                        zero_const,
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
                        zero_const,
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
        _elif_branches: Vec<(Box<AstNode>, Vec<Box<AstNode>>)>,
    ) -> ValueId {
        let then_block = self.create_block_with_label("then".to_string());
        let else_block = self.create_block_with_label("else".to_string());
        let merge_block = self.create_block_with_label("endif".to_string());
        
        // Generate condition
        let cond_status = self.generate_node(condition);
        
        // Branch based on condition
        let cond = self.create_value(ValueType::Boolean);
        let zero_const = self.create_const_int(0);
        self.add_instruction(Instruction::Cmp(
            cond_status,
            zero_const,
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
        // Case statement: case word in pattern1) commands;; pattern2) commands;; esac
        
        // Evaluate the word
        let word_val = self.create_value(ValueType::String);
        self.add_instruction(Instruction::ConstString(word.to_string(), word_val));
        
        // Create blocks for each case and a default/exit block
        let mut case_blocks = Vec::new();
        let exit_block = self.create_block_with_label("case_exit".to_string());
        let default_block = self.create_block_with_label("case_default".to_string());
        
        // Generate blocks for each case
        for (i, case) in cases.iter().enumerate() {
            let case_block = self.create_block_with_label(format!("case_{}", i));
            let next_block = if i < cases.len() - 1 {
                self.create_block_with_label(format!("case_next_{}", i))
            } else {
                default_block
            };
            
            case_blocks.push((case_block, next_block, case));
        }
        
        // Start with first case
        let mut current_block = self.current_block.unwrap();
        
        for (i, (case_block, next_block, case)) in case_blocks.iter().enumerate() {
            // Jump to case block
            self.set_current_block(current_block);
            self.add_instruction(Instruction::Jump(*case_block));
            
            // Case block: check patterns
            self.set_current_block(*case_block);
            
            // For each pattern in this case
            let mut pattern_checks = Vec::new();
            for pattern in &case.patterns {
                // TODO: Implement pattern matching
                // For now, just do string equality
                let pattern_val = self.create_value(ValueType::String);
                self.add_instruction(Instruction::ConstString(pattern.clone(), pattern_val));
                
                let match_result = self.create_value(ValueType::Boolean);
                self.add_instruction(Instruction::Cmp(
                    word_val,
                    pattern_val,
                    CmpOp::Eq,
                    match_result,
                ));
                
                pattern_checks.push(match_result);
            }
            
            // Combine pattern checks with OR
            let mut combined_match = if let Some(first) = pattern_checks.first() {
                *first
            } else {
                // No patterns, always match (default case)
                let always_true = self.create_value(ValueType::Boolean);
                self.add_instruction(Instruction::ConstBool(true, always_true));
                always_true
            };
            
            for check in pattern_checks.iter().skip(1) {
                let new_combined = self.create_value(ValueType::Boolean);
                self.add_instruction(Instruction::Or(combined_match, *check, new_combined));
                combined_match = new_combined;
            }
            
            // Branch: if matched, execute body, else go to next case
            let body_block = self.create_block_with_label(format!("case_body_{}", i));
            self.add_instruction(Instruction::Branch(
                combined_match,
                body_block,
                *next_block,
            ));
            
            // Body block: execute case commands
            self.set_current_block(body_block);
            let _body_result = self.generate_compound_command(
                case.body.iter().map(|c| c.clone()).collect()
            );
            
            // Jump to exit after executing body
            self.add_instruction(Instruction::Jump(exit_block));
            
            // Set up for next case
            current_block = *next_block;
        }
        
        // Default block: no pattern matched (do nothing)
        self.set_current_block(default_block);
        self.add_instruction(Instruction::Jump(exit_block));
        
        // Exit block
        self.set_current_block(exit_block);
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
        let zero_const = self.create_const_int(0);
        self.add_instruction(Instruction::Cmp(
            cond_status,
            zero_const,
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
        let zero_const = self.create_const_int(0);
        self.add_instruction(Instruction::Cmp(
            cond_status,
            zero_const,
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
        let _loop_body = self.create_block_with_label("for_body".to_string());
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
        // Select statement: select var in list; do commands; done
        // Presents a menu of items and executes body with selected item
        
        // Create selection variable
        let var_val = self.create_value_with_name(ValueType::String, variable.to_string());
        self.add_instruction(Instruction::AllocVar(variable.to_string(), var_val));
        
        // Create loop for selection
        let loop_start = self.create_block_with_label("select_start".to_string());
        let loop_body = self.create_block_with_label("select_body".to_string());
        let loop_exit = self.create_block_with_label("select_exit".to_string());
        
        // Jump to start
        self.add_instruction(Instruction::Jump(loop_start));
        
        // Start block: display menu
        self.set_current_block(loop_start);
        
        // TODO: Display menu and get user input
        // For now, just use first item
        
        if let Some(first_item) = items.first() {
            // Set variable to first item
            let item_val = self.create_value(ValueType::String);
            self.add_instruction(Instruction::ConstString(first_item.clone(), item_val));
            self.add_instruction(Instruction::Store(var_val, item_val));
            
            // Jump to body
            self.add_instruction(Instruction::Jump(loop_body));
        } else {
            // No items, exit
            self.add_instruction(Instruction::Jump(loop_exit));
        }
        
        // Body block: execute commands with selected variable
        self.set_current_block(loop_body);
        let _body_result = self.generate_compound_command(body.clone());
        
        // Check if we should continue (placeholder - in real select, user can break)
        let continue_val = self.create_value(ValueType::Boolean);
        self.add_instruction(Instruction::ConstBool(false, continue_val)); // Exit after one iteration for now
        
        self.add_instruction(Instruction::Branch(
            continue_val,
            loop_start,
            loop_exit,
        ));
        
        // Exit block
        self.set_current_block(loop_exit);
        let result = self.create_value(ValueType::ExitStatus);
        self.add_instruction(Instruction::ConstInt(0, result));
        
        result
    }
    
    fn generate_function_definition(&mut self, name: &str, _body: Vec<Box<AstNode>>) -> ValueId {
        // Function definition: name() { body; }
        // In SSA IR, we create a new function
        
        // For now, we'll just store the function body in a placeholder
        // In a full implementation, we would create a new Function and store it
        
        // Create a function value placeholder
        let func_val = self.create_value_with_name(ValueType::String, name.to_string());
        
        // Store function name (placeholder for actual function)
        let func_name_val = self.create_value(ValueType::String);
        self.add_instruction(Instruction::ConstString(name.to_string(), func_name_val));
        self.add_instruction(Instruction::Store(func_val, func_name_val));
        
        // TODO: Actually create and store function body
        // For now, just return success
        
        let result = self.create_value(ValueType::ExitStatus);
        self.add_instruction(Instruction::ConstInt(0, result));
        result
    }
    
    fn generate_redirection(&mut self, command: AstNode, redirect_type: RedirectType, target: &str, fd: Option<i32>) -> ValueId {
        // Generate command first
        let cmd_result = self.generate_node(command);
        
        // Get file descriptor (default based on redirect type)
        let target_fd = fd.unwrap_or_else(|| {
            match redirect_type {
                RedirectType::Input | 
                RedirectType::HereDoc | 
                RedirectType::HereDocStrip |
                RedirectType::DupInput |
                RedirectType::ReadWrite => 0, // stdin
                _ => 1, // stdout
            }
        });
        
        // Create value for file descriptor
        let fd_val = self.create_value(ValueType::FileDescriptor);
        self.add_instruction(Instruction::ConstInt(target_fd, fd_val));
        
        // Convert redirect type to SSA IR mode
        let mode = match redirect_type {
            RedirectType::Input => crate::modules::ssa_ir::RedirectMode::Read,
            RedirectType::Output => crate::modules::ssa_ir::RedirectMode::Write,
            RedirectType::Append => crate::modules::ssa_ir::RedirectMode::Append,
            RedirectType::HereDoc => crate::modules::ssa_ir::RedirectMode::HereDoc,
            RedirectType::HereDocStrip => crate::modules::ssa_ir::RedirectMode::HereDocStrip,
            RedirectType::DupInput => crate::modules::ssa_ir::RedirectMode::DupRead,
            RedirectType::DupOutput => crate::modules::ssa_ir::RedirectMode::DupWrite,
            RedirectType::ReadWrite => crate::modules::ssa_ir::RedirectMode::ReadWrite,
            RedirectType::Clobber => crate::modules::ssa_ir::RedirectMode::Write, // Same as output for now
        };
        
        // Add redirection instruction
        self.add_instruction(Instruction::Redirect(
            fd_val,
            target.to_string(),
            mode,
        ));
        
        cmd_result
    }
    
    fn generate_background(&mut self, command: AstNode) -> ValueId {
        // For background execution, we need to fork and execute command in child process
        // Parent process should return success immediately without waiting
        
        // Fork a new process
        let pid = self.create_value(ValueType::ProcessId);
        self.add_instruction(Instruction::Fork(pid));
        
        // Create blocks for parent and child
        let parent_block = self.create_block_with_label("background_parent".to_string());
        let child_block = self.create_block_with_label("background_child".to_string());
        
        // Branch based on fork result
        let zero_const = self.create_const_int(0);
        let is_child = self.create_value(ValueType::Boolean);
        self.add_instruction(Instruction::Cmp(pid, zero_const, CmpOp::Eq, is_child));
        self.add_instruction(Instruction::Branch(is_child, child_block, parent_block));
        
        // Child block: execute the command
        self.set_current_block(child_block);
        let cmd_status = self.generate_node(command);
        self.add_instruction(Instruction::Exit(cmd_status));
        
        // Parent block: return success immediately (don't wait for child)
        self.set_current_block(parent_block);
        
        // TODO: Register the background job with job control
        // For now, just return success
        
        let result = self.create_value(ValueType::ExitStatus);
        self.add_instruction(Instruction::ConstInt(0, result));
        result
    }
    
    fn generate_command_substitution(&mut self, command: AstNode, _backticks: bool) -> ValueId {
        // Command substitution needs to capture output of a command
        // We'll create a pipe, execute the command with output redirected to the pipe,
        // then read from the pipe
        
        // Create a pipe for capturing output
        let read_fd = self.create_value(ValueType::FileDescriptor);
        let write_fd = self.create_value(ValueType::FileDescriptor);
        self.add_instruction(Instruction::CreatePipe(read_fd, write_fd));
        
        // Fork to execute command
        let pid = self.create_value(ValueType::ProcessId);
        self.add_instruction(Instruction::Fork(pid));
        
        // Create blocks for parent and child
        let parent_block = self.create_block_with_label("cmdsub_parent".to_string());
        let child_block = self.create_block_with_label("cmdsub_child".to_string());
        
        // Branch based on fork result
        let zero_const = self.create_const_int(0);
        let is_child = self.create_value(ValueType::Boolean);
        self.add_instruction(Instruction::Cmp(pid, zero_const, CmpOp::Eq, is_child));
        self.add_instruction(Instruction::Branch(is_child, child_block, parent_block));
        
        // Child block: redirect stdout to pipe, execute command
        self.set_current_block(child_block);
        
        // Redirect stdout to write end of pipe
        let dup_result = self.create_value(ValueType::FileDescriptor);
        self.add_instruction(Instruction::DupFd(write_fd, 1, dup_result));
        
        // Close both ends of pipe (child only needs write end)
        self.add_instruction(Instruction::CloseFd(read_fd));
        self.add_instruction(Instruction::CloseFd(write_fd));
        
        // Execute command
        let cmd_status = self.generate_node(command);
        
        // Exit with command status
        self.add_instruction(Instruction::Exit(cmd_status));
        
        // Parent block: read from pipe using command substitution
        self.set_current_block(parent_block);
        
        // Close write end (parent only needs read end)
        self.add_instruction(Instruction::CloseFd(write_fd));
        
        // Wait for child process to get exit status
        let wait_status = self.create_value(ValueType::ExitStatus);
        self.add_instruction(Instruction::Wait(pid, wait_status));
        
        // Create result value for command substitution
        let result = self.create_value(ValueType::String);
        
        // Use CommandSub instruction to capture output from read_fd
        self.add_instruction(Instruction::CommandSub(wait_status, result));
        
        // Close read end after reading
        self.add_instruction(Instruction::CloseFd(read_fd));
        
        result
    }
    
    fn generate_parameter_expansion(&mut self, parameter: &str, operation: Option<ParameterOperation>) -> ValueId {
        // Create parameter value
        let param_val = self.create_value(ValueType::String);
        self.add_instruction(Instruction::ConstString(parameter.to_string(), param_val));
        
        // Convert AST operation to SSA IR operation
        let op = match operation {
            Some(ast_op) => match ast_op {
                ParameterOperation::UseDefault(word) => ParamExpandOp::UseDefault(word),
                ParameterOperation::AssignDefault(word) => ParamExpandOp::AssignDefault(word),
                ParameterOperation::ErrorIfNull(word) => ParamExpandOp::ErrorIfNull(word),
                ParameterOperation::UseAlternate(word) => ParamExpandOp::UseAlternate(word),
                ParameterOperation::Length => ParamExpandOp::Length,
                ParameterOperation::RemoveSuffix(word) => ParamExpandOp::RemoveSuffix(word),
                ParameterOperation::RemoveLargestSuffix(word) => ParamExpandOp::RemoveLargestSuffix(word),
                ParameterOperation::RemovePrefix(word) => ParamExpandOp::RemovePrefix(word),
                ParameterOperation::RemoveLargestPrefix(word) => ParamExpandOp::RemoveLargestPrefix(word),
            },
            None => ParamExpandOp::Simple,
        };
        
        // Create result value
        let result = self.create_value(ValueType::String);
        self.add_instruction(Instruction::ParamExpand(param_val, op, result));
        
        result
    }
    
    fn generate_null_command(&mut self) -> ValueId {
        let result = self.create_value(ValueType::ExitStatus);
        self.add_instruction(Instruction::ConstInt(0, result));
        result
    }
    
    fn generate_error(&mut self, message: &str, token: crate::modules::tokens::Token) -> ValueId {
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
fn generate_export_command(&mut self, variables: &[String]) -> ValueId {
        for var in variables {
            let var_val = self.create_value_with_name(ValueType::String, var.clone());
            self.add_instruction(Instruction::ExportVar(var_val));
        }
        
        let result = self.create_value(ValueType::ExitStatus);
        self.add_instruction(Instruction::ConstInt(0, result));
        result
    }
    
    fn generate_unset_command(&mut self, variables: &[String]) -> ValueId {
        for var in variables {
            let var_val = self.create_value_with_name(ValueType::String, var.clone());
            self.add_instruction(Instruction::UnsetVar(var_val));
        }
        
        let result = self.create_value(ValueType::ExitStatus);
        self.add_instruction(Instruction::ConstInt(0, result));
        result
    }
    
    fn generate_readonly_command(&mut self, variables: &[String]) -> ValueId {
        for var in variables {
            let var_val = self.create_value_with_name(ValueType::String, var.clone());
            self.add_instruction(Instruction::ReadonlyVar(var_val));
        }
        
        let result = self.create_value(ValueType::ExitStatus);
        self.add_instruction(Instruction::ConstInt(0, result));
        result
    }
}