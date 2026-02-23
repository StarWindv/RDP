//! Executor for IR programs

use std::process::Command;

use crate::builtins::Builtins;
use crate::ir::{IrInstruction, IrProgram};
use crate::env::ShellEnv;

/// Executor for IR programs
pub struct Executor {
    builtins: Builtins,
    env: ShellEnv,
}

impl Executor {
    /// Create a new executor
    pub fn new() -> Self {
        Self {
            builtins: Builtins,
            env: ShellEnv::new(),
        }
    }
    
    /// Execute an IR program
    pub fn execute(&mut self, program: &IrProgram) -> i32 {
        // First, define functions
        for (name, _body) in &program.functions {
            // Store function definition in environment
            // For now, we'll just note that it exists
            self.env.set_var(format!("__func_{}", name), "defined".to_string());
        }
        
        // Execute main instructions
        for instruction in &program.instructions {
            if let Err(status) = self.execute_instruction(instruction) {
                self.env.exit_status = status;
                return status;
            }
        }
        
        self.env.exit_status
    }
    
    /// Execute a single instruction
    fn execute_instruction(&mut self, instruction: &Box<IrInstruction>) -> Result<(), i32> {
        match instruction.as_ref() {
            IrInstruction::ExecuteCommand { name, args, .. } => {
                let status = self.execute_command(name, args);
                self.env.exit_status = status;
                if status != 0 {
                    return Err(status);
                }
            }
            
            IrInstruction::SetVariable { name, value, .. } => {
                let expanded_value = self.env.expand_variables(value);
                self.env.set_var(name.clone(), expanded_value);
            }
            
            IrInstruction::CreatePipeline { commands, .. } => {
                let status = self.execute_pipeline(commands);
                self.env.exit_status = status;
                if status != 0 {
                    return Err(status);
                }
            }
            
            IrInstruction::ConditionalAnd { condition, body, .. } => {
                // Execute condition
                self.execute_instruction(condition)?;
                
                // If condition succeeded, execute body
                if self.env.exit_status == 0 {
                    self.execute_instruction(body)?;
                }
            }
            
            IrInstruction::ConditionalOr { condition, body, .. } => {
                // Execute condition
                self.execute_instruction(condition)?;
                
                // If condition failed, execute body
                if self.env.exit_status != 0 {
                    self.execute_instruction(body)?;
                }
            }
            
            IrInstruction::Redirect { command, redirect_type, target, fd, .. } => {
                // TODO: Implement redirection
                // For now, just execute the command without redirection
                let _ = redirect_type;
                let _ = target;
                let _ = fd;
                let _ = command;
            }
            
            IrInstruction::Background { command, .. } => {
                // TODO: Implement background execution
                // For now, just execute in foreground
                self.execute_instruction(command)?;
            }
            
            IrInstruction::CompoundBlock { instructions, .. } => {
                for instr in instructions {
                    self.execute_instruction(instr)?;
                }
            }
            
            IrInstruction::IfStatement { condition, then_branch, else_branch, elif_branches, .. } => {
                // Execute condition
                self.execute_instruction(condition)?;
                
                if self.env.exit_status == 0 {
                    // Execute then branch
                    for instr in then_branch {
                        self.execute_instruction(instr)?;
                    }
                } else {
                    // Check elif branches
                    let mut elif_executed = false;
                    for (elif_cond, elif_body) in elif_branches {
                        self.execute_instruction(elif_cond)?;
                        if self.env.exit_status == 0 {
                            for instr in elif_body {
                                self.execute_instruction(instr)?;
                            }
                            elif_executed = true;
                            break;
                        }
                    }
                    
                    // If no elif executed, execute else branch
                    if !elif_executed {
                        if let Some(else_body) = else_branch {
                            for instr in else_body {
                                self.execute_instruction(instr)?;
                            }
                        }
                    }
                }
            }
            
            IrInstruction::WhileLoop { condition, body, .. } => {
                loop {
                    // Execute condition
                    self.execute_instruction(condition)?;
                    
                    // If condition failed, break
                    if self.env.exit_status != 0 {
                        break;
                    }
                    
                    // Execute body
                    for instr in body {
                        self.execute_instruction(instr)?;
                    }
                }
            }
            
            IrInstruction::UntilLoop { condition, body, .. } => {
                loop {
                    // Execute condition
                    self.execute_instruction(condition)?;
                    
                    // If condition succeeded, break
                    if self.env.exit_status == 0 {
                        break;
                    }
                    
                    // Execute body
                    for instr in body {
                        self.execute_instruction(instr)?;
                    }
                }
            }
            
            IrInstruction::ForLoop { variable, items, body, .. } => {
                for item in items {
                    // Set variable for this iteration
                    let expanded_item = self.env.expand_variables(item);
                    self.env.set_var(variable.clone(), expanded_item);
                    
                    // Execute body
                    for instr in body {
                        self.execute_instruction(instr)?;
                    }
                }
            }
            
            IrInstruction::DefineFunction { name, .. } => {
                // Function definitions were already handled
                // Just return success
                let _ = name;
            }
            
            IrInstruction::CallFunction { name, args, .. } => {
                // TODO: Implement function calls
                // For now, treat as external command
                let status = self.execute_command(name, args);
                self.env.exit_status = status;
                if status != 0 {
                    return Err(status);
                }
            }
            
            IrInstruction::Subshell { instructions, .. } => {
                // TODO: Implement subshell execution
                // For now, just execute instructions in current shell
                for instr in instructions {
                    self.execute_instruction(instr)?;
                }
            }
            
            IrInstruction::CommandSubstitution { command, .. } => {
                // TODO: Implement command substitution
                // For now, just execute the command
                self.execute_instruction(command)?;
            }
            
            IrInstruction::Nop => {
                // Do nothing
            }
            
            IrInstruction::Error { message, token } => {
                eprintln!("Error at {}:{}: {}", token.line, token.column, message);
                return Err(1);
            }
        }
        
        Ok(())
    }
    
    /// Execute a command (built-in or external)
    fn execute_command(&mut self, name: &str, args: &[String]) -> i32 {
        // Expand variables in arguments
        let expanded_args: Vec<String> = args.iter()
            .map(|arg| self.env.expand_variables(arg))
            .collect();
        
        if self.builtins.is_builtin(name) {
            self.builtins.execute(name, &expanded_args, &mut self.env)
        } else {
            self.execute_external_command(name, &expanded_args)
        }
    }
    
    /// Execute an external command
    fn execute_external_command(&self, cmd: &str, args: &[String]) -> i32 {
        // Find command in PATH
        let full_path = match self.env.find_in_path(cmd) {
            Some(path) => path,
            None => {
                eprintln!("{}: command not found", cmd);
                return 127;
            }
        };
        
        // Prepare command
        let mut command = Command::new(&full_path);
        for arg in args {
            command.arg(arg);
        }
        command.current_dir(&self.env.current_dir);
        
        // Set environment variables
        for (key, value) in &self.env.vars {
            command.env(key, value);
        }
        
        // Execute command
        match command.status() {
            Ok(status) => {
                if let Some(code) = status.code() {
                    code
                } else {
                    // Command was terminated by a signal
                    128
                }
            }
            Err(e) => {
                eprintln!("{}: {}", cmd, e);
                1
            }
        }
    }
    
    /// Execute a pipeline
    fn execute_pipeline(&mut self, commands: &[Box<IrInstruction>]) -> i32 {
        // For now, just execute commands sequentially
        // TODO: Implement proper pipeline with pipes
        let mut last_status = 0;
        
        for cmd in commands {
            if let Err(status) = self.execute_instruction(cmd) {
                last_status = status;
            }
        }
        
        last_status
    }
    
    /// Get current environment
    pub fn get_env(&self) -> &ShellEnv {
        &self.env
    }
    
    /// Get mutable environment
    pub fn get_env_mut(&mut self) -> &mut ShellEnv {
        &mut self.env
    }
}