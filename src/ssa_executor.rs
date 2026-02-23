//! SSA IR Executor
//! Executes SSA IR programs with proper shell semantics

use std::collections::HashMap;
use std::process::Command;

use crate::ssa_ir::{
    Function, BasicBlockId, ValueId, ValueType,
    Instruction, CmpOp,
};
use crate::builtins::Builtins;
use crate::env::ShellEnv;

/// SSA IR Executor
pub struct SsaExecutor {
    builtins: Builtins,
    env: ShellEnv,
    functions: HashMap<String, Function>,
    values: HashMap<ValueId, ExecValue>,
    current_function: Option<String>,
    current_block: Option<BasicBlockId>,
    program_counter: usize,
    call_stack: Vec<(String, BasicBlockId, usize)>, // (function, block, pc)
}

/// Executable value
#[derive(Debug, Clone)]
enum ExecValue {
    String(String),
    Integer(i32),
    Boolean(bool),
    FileDescriptor(i32),
    ProcessId(u32),
    ExitStatus(i32),
    Void,
}

impl ExecValue {
    fn get_type(&self) -> ValueType {
        match self {
            ExecValue::String(_) => ValueType::String,
            ExecValue::Integer(_) => ValueType::Integer,
            ExecValue::Boolean(_) => ValueType::Boolean,
            ExecValue::FileDescriptor(_) => ValueType::FileDescriptor,
            ExecValue::ProcessId(_) => ValueType::ProcessId,
            ExecValue::ExitStatus(_) => ValueType::ExitStatus,
            ExecValue::Void => ValueType::Void,
        }
    }
    
    fn as_string(&self) -> String {
        match self {
            ExecValue::String(s) => s.clone(),
            ExecValue::Integer(i) => i.to_string(),
            ExecValue::Boolean(b) => b.to_string(),
            ExecValue::FileDescriptor(fd) => fd.to_string(),
            ExecValue::ProcessId(pid) => pid.to_string(),
            ExecValue::ExitStatus(status) => status.to_string(),
            ExecValue::Void => String::new(),
        }
    }
    
    fn as_integer(&self) -> i32 {
        match self {
            ExecValue::String(s) => s.parse().unwrap_or(0),
            ExecValue::Integer(i) => *i,
            ExecValue::Boolean(b) => if *b { 1 } else { 0 },
            ExecValue::FileDescriptor(fd) => *fd,
            ExecValue::ProcessId(pid) => *pid as i32,
            ExecValue::ExitStatus(status) => *status,
            ExecValue::Void => 0,
        }
    }
    
    fn as_boolean(&self) -> bool {
        match self {
            ExecValue::String(s) => !s.is_empty(),
            ExecValue::Integer(i) => *i != 0,
            ExecValue::Boolean(b) => *b,
            ExecValue::FileDescriptor(fd) => *fd >= 0,
            ExecValue::ProcessId(pid) => *pid > 0,
            ExecValue::ExitStatus(status) => *status == 0,
            ExecValue::Void => false,
        }
    }
    
    fn as_fd(&self) -> i32 {
        match self {
            ExecValue::FileDescriptor(fd) => *fd,
            _ => -1,
        }
    }
    
    fn as_pid(&self) -> u32 {
        match self {
            ExecValue::ProcessId(pid) => *pid,
            _ => 0,
        }
    }
    
    fn as_status(&self) -> i32 {
        match self {
            ExecValue::ExitStatus(status) => *status,
            _ => 0,
        }
    }
}

impl SsaExecutor {
    /// Create a new SSA executor
    pub fn new() -> Self {
        Self {
            builtins: Builtins,
            env: ShellEnv::new(),
            functions: HashMap::new(),
            values: HashMap::new(),
            current_function: None,
            current_block: None,
            program_counter: 0,
            call_stack: Vec::new(),
        }
    }
    
    /// Execute a function
    pub fn execute_function(&mut self, func: &Function) -> i32 {
        println!("DEBUG: Starting execution of function: {}", func.name);
        
        // Store function
        self.functions.insert(func.name.clone(), func.clone());
        
        // Set up execution context
        self.current_function = Some(func.name.clone());
        self.current_block = Some(func.entry_block);
        self.program_counter = 0;
        self.call_stack.clear();
        
        // Execute
        let result = self.execute_block(func.entry_block, func);
        
        println!("DEBUG: Execution completed with status: {}", result.as_status());
        
        // Clean up
        self.current_function = None;
        self.current_block = None;
        
        result.as_status()
    }
    
    /// Execute a basic block
    fn execute_block(&mut self, block_id: BasicBlockId, func: &Function) -> ExecValue {
        let block = func.get_block(block_id).expect("Block should exist");
        self.current_block = Some(block_id);
        self.program_counter = 0;
        
        for instr in &block.instructions {
            let result = self.execute_instruction(instr, func);
            
            // Check for control flow instructions
            match instr {
                Instruction::Jump(target) => {
                    return self.execute_block(*target, func);
                }
                Instruction::Branch(cond, true_block, false_block) => {
                    let cond_val = self.get_value(*cond);
                    if cond_val.as_boolean() {
                        return self.execute_block(*true_block, func);
                    } else {
                        return self.execute_block(*false_block, func);
                    }
                }
                Instruction::Return(status) => {
                    return self.get_value(*status);
                }
                _ => {}
            }
            
            self.program_counter += 1;
        }
        
        // If we reach the end of the block without a terminator,
        // it's an error (blocks should end with a terminator)
        ExecValue::ExitStatus(1)
    }
    
    /// Execute a single instruction
    fn execute_instruction(&mut self, instr: &Instruction, _func: &Function) -> ExecValue {
        match instr {
            // Control flow (handled in execute_block)
            Instruction::Jump(_) |
            Instruction::Branch(_, _, _) |
            Instruction::Return(_) => ExecValue::Void,
            
            // Variable operations
            Instruction::AllocVar(_name, result) => {
                let value = ExecValue::String(String::new());
                self.set_value(*result, value.clone());
                value
            }
            
            Instruction::Store(var, value) => {
                let val = self.get_value(*value);
                // In real shell, we'd store in environment
                // For now, just store in values map
                self.set_value(*var, val.clone());
                ExecValue::Void
            }
            
            Instruction::Load(var, result) => {
                let val = self.get_value(*var);
                self.set_value(*result, val.clone());
                val
            }
            
            // Command execution
            Instruction::CallBuiltin(name, args, result) => {
                let arg_strings: Vec<String> = args.iter()
                    .map(|arg| self.get_value(*arg).as_string())
                    .collect();
                
                let status = self.builtins.execute(name, &arg_strings, &mut self.env);
                let value = ExecValue::ExitStatus(status);
                self.set_value(*result, value.clone());
                value
            }
            
            Instruction::CallExternal(cmd, args, result) => {
                let cmd_str = cmd.clone();
                let arg_strings: Vec<String> = args.iter()
                    .map(|arg| self.get_value(*arg).as_string())
                    .collect();
                
                // Check if it's a built-in command
                let status = if self.builtins.is_builtin(&cmd_str) {
                    self.builtins.execute(&cmd_str, &arg_strings, &mut self.env)
                } else {
                    self.execute_external_command(&cmd_str, &arg_strings)
                };
                
                let value = ExecValue::ExitStatus(status);
                self.set_value(*result, value.clone());
                value
            }
            
            // Process operations
            Instruction::Fork(result) => {
                // TODO: Implement fork
                // For now, return pid 0 (current process)
                let value = ExecValue::ProcessId(0);
                self.set_value(*result, value.clone());
                value
            }
            
            Instruction::Exec(pid, cmd, args) => {
                // TODO: Implement exec
                // For now, just execute command
                let _pid_val = self.get_value(*pid);
                let cmd_str = cmd.clone();
                let arg_strings: Vec<String> = args.iter()
                    .map(|arg| self.get_value(*arg).as_string())
                    .collect();
                
                let status = self.execute_external_command(&cmd_str, &arg_strings);
                ExecValue::ExitStatus(status)
            }
            
            Instruction::Wait(_pid, result) => {
                // TODO: Implement wait
                // For now, return success
                let value = ExecValue::ExitStatus(0);
                self.set_value(*result, value.clone());
                value
            }
            
            Instruction::Exit(status) => {
                let status_val = self.get_value(*status).as_status();
                ExecValue::ExitStatus(status_val)
            }
            
            // Pipeline operations
            Instruction::CreatePipe(read_fd, write_fd) => {
                // TODO: Implement pipe creation
                // For now, create dummy file descriptors
                let read_val = ExecValue::FileDescriptor(3);
                let write_val = ExecValue::FileDescriptor(4);
                self.set_value(*read_fd, read_val.clone());
                self.set_value(*write_fd, write_val.clone());
                ExecValue::Void
            }
            
            Instruction::DupFd(_old_fd, new_fd, result) => {
                // TODO: Implement dup
                // For now, just return new_fd
                let value = ExecValue::FileDescriptor(*new_fd);
                self.set_value(*result, value.clone());
                value
            }
            
            Instruction::CloseFd(_fd) => {
                // TODO: Implement close
                ExecValue::Void
            }
            
            Instruction::Redirect(_fd, _target, _mode) => {
                // TODO: Implement redirection
                ExecValue::Void
            }
            
            // String operations
            Instruction::Concat(str1, str2, result) => {
                let s1 = self.get_value(*str1).as_string();
                let s2 = self.get_value(*str2).as_string();
                let value = ExecValue::String(s1 + &s2);
                self.set_value(*result, value.clone());
                value
            }
            
            Instruction::Substr(str, start, len, result) => {
                let s = self.get_value(*str).as_string();
                let start_idx = self.get_value(*start).as_integer() as usize;
                let len_val = self.get_value(*len).as_integer() as usize;
                
                let end_idx = std::cmp::min(start_idx + len_val, s.len());
                let substr = if start_idx < s.len() {
                    s[start_idx..end_idx].to_string()
                } else {
                    String::new()
                };
                
                let value = ExecValue::String(substr);
                self.set_value(*result, value.clone());
                value
            }
            
            Instruction::Length(str, result) => {
                let s = self.get_value(*str).as_string();
                let value = ExecValue::Integer(s.len() as i32);
                self.set_value(*result, value.clone());
                value
            }
            
            // Arithmetic operations
            Instruction::Add(a, b, result) => {
                let a_val = self.get_value(*a).as_integer();
                let b_val = self.get_value(*b).as_integer();
                let value = ExecValue::Integer(a_val + b_val);
                self.set_value(*result, value.clone());
                value
            }
            
            Instruction::Sub(a, b, result) => {
                let a_val = self.get_value(*a).as_integer();
                let b_val = self.get_value(*b).as_integer();
                let value = ExecValue::Integer(a_val - b_val);
                self.set_value(*result, value.clone());
                value
            }
            
            Instruction::Mul(a, b, result) => {
                let a_val = self.get_value(*a).as_integer();
                let b_val = self.get_value(*b).as_integer();
                let value = ExecValue::Integer(a_val * b_val);
                self.set_value(*result, value.clone());
                value
            }
            
            Instruction::Div(a, b, result) => {
                let a_val = self.get_value(*a).as_integer();
                let b_val = self.get_value(*b).as_integer();
                let value = if b_val != 0 {
                    ExecValue::Integer(a_val / b_val)
                } else {
                    ExecValue::Integer(0)
                };
                self.set_value(*result, value.clone());
                value
            }
            
            Instruction::Mod(a, b, result) => {
                let a_val = self.get_value(*a).as_integer();
                let b_val = self.get_value(*b).as_integer();
                let value = if b_val != 0 {
                    ExecValue::Integer(a_val % b_val)
                } else {
                    ExecValue::Integer(0)
                };
                self.set_value(*result, value.clone());
                value
            }
            
            // Logical operations
            Instruction::And(a, b, result) => {
                let a_val = self.get_value(*a).as_boolean();
                let b_val = self.get_value(*b).as_boolean();
                let value = ExecValue::Boolean(a_val && b_val);
                self.set_value(*result, value.clone());
                value
            }
            
            Instruction::Or(a, b, result) => {
                let a_val = self.get_value(*a).as_boolean();
                let b_val = self.get_value(*b).as_boolean();
                let value = ExecValue::Boolean(a_val || b_val);
                self.set_value(*result, value.clone());
                value
            }
            
            Instruction::Not(a, result) => {
                let a_val = self.get_value(*a).as_boolean();
                let value = ExecValue::Boolean(!a_val);
                self.set_value(*result, value.clone());
                value
            }
            
            // Comparison operations
            Instruction::Cmp(a, b, op, result) => {
                let a_val = self.get_value(*a).as_integer();
                let b_val = self.get_value(*b).as_integer();
                
                let cmp_result = match op {
                    CmpOp::Eq => a_val == b_val,
                    CmpOp::Ne => a_val != b_val,
                    CmpOp::Lt => a_val < b_val,
                    CmpOp::Le => a_val <= b_val,
                    CmpOp::Gt => a_val > b_val,
                    CmpOp::Ge => a_val >= b_val,
                };
                
                let value = ExecValue::Boolean(cmp_result);
                self.set_value(*result, value.clone());
                value
            }
            
            // Constants
            Instruction::ConstString(val, result) => {
                let value = ExecValue::String(val.clone());
                self.set_value(*result, value.clone());
                value
            }
            
            Instruction::ConstInt(val, result) => {
                let value = ExecValue::Integer(*val);
                self.set_value(*result, value.clone());
                value
            }
            
            Instruction::ConstBool(val, result) => {
                let value = ExecValue::Boolean(*val);
                self.set_value(*result, value.clone());
                value
            }
            
            // Phi function
            Instruction::Phi(_pairs, _result) => {
                // Phi functions are handled during block execution
                // We should never execute them directly
                ExecValue::Void
            }
            
            // Special
            Instruction::Nop => ExecValue::Void,
            
            Instruction::Error(msg, token) => {
                eprintln!("Error at {}:{}: {}", token.line, token.column, msg);
                ExecValue::ExitStatus(1)
            }
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
    
    /// Get a value by ID
    fn get_value(&self, id: ValueId) -> ExecValue {
        self.values.get(&id)
            .cloned()
            .unwrap_or_else(|| ExecValue::Void)
    }
    
    /// Set a value by ID
    fn set_value(&mut self, id: ValueId, value: ExecValue) {
        self.values.insert(id, value);
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