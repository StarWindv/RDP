//! SSA IR Executor
//! Executes SSA IR programs with proper shell semantics

use std::collections::HashMap;
use std::process::Command;

use crate::modules::ssa_ir::{
    Function, BasicBlockId, ValueId, ValueType,
    Instruction, CmpOp,
};
use crate::modules::builtins::Builtins;
use crate::modules::env::ShellEnv;
use crate::modules::variables::get_variable_system;

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
    predecessor_blocks: HashMap<BasicBlockId, BasicBlockId>, // block -> predecessor
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
            builtins: Builtins::new(),
            env: ShellEnv::new(),
            functions: HashMap::new(),
            values: HashMap::new(),
            current_function: None,
            current_block: None,
            program_counter: 0,
            call_stack: Vec::new(),
            predecessor_blocks: HashMap::new(),
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
        self.predecessor_blocks.clear();
        
        // Execute
        let result = self.execute_block(func.entry_block, func);
        
        println!("DEBUG: Execution completed with status: {}", result.as_status());
        
        // Clean up
        self.current_function = None;
        self.current_block = None;
        self.predecessor_blocks.clear();
        
        result.as_status()
    }
    
    /// Execute a basic block
    fn execute_block(&mut self, block_id: BasicBlockId, func: &Function) -> ExecValue {
        let block = func.get_block(block_id).expect("Block should exist");
        self.current_block = Some(block_id);
        self.program_counter = 0;
        
        // First, execute Phi nodes if we have a predecessor
        let pred_block = self.predecessor_blocks.get(&block_id).cloned();
        if let Some(pred_block) = pred_block {
            for instr in &block.instructions {
                if let Instruction::Phi(pairs, result) = instr {
                    // Find the value from the predecessor block
                    let mut phi_value = None;
                    for (pred, value) in pairs {
                        if pred == &pred_block {
                            phi_value = Some(self.get_value(*value));
                            break;
                        }
                    }
                    
                    // Set the phi result value
                    if let Some(value) = phi_value {
                        self.set_value(*result, value.clone());
                    } else {
                        // No matching predecessor, use default
                        self.set_value(*result, ExecValue::Void);
                    }
                } else {
                    // Not a Phi node, break
                    break;
                }
            }
        }
        
        // Execute remaining instructions
        for instr in &block.instructions {
            // Skip Phi nodes (already handled)
            if matches!(instr, Instruction::Phi(_, _)) {
                self.program_counter += 1;
                continue;
            }
            
            let _result = self.execute_instruction(instr, func);
            
            // Check for control flow instructions
            match instr {
                Instruction::Jump(target) => {
                    // Record predecessor for next block
                    self.predecessor_blocks.insert(*target, block_id);
                    return self.execute_block(*target, func);
                }
                Instruction::Branch(cond, true_block, false_block) => {
                    let cond_val = self.get_value(*cond);
                    if cond_val.as_boolean() {
                        // Record predecessor for true block
                        self.predecessor_blocks.insert(*true_block, block_id);
                        return self.execute_block(*true_block, func);
                    } else {
                        // Record predecessor for false block
                        self.predecessor_blocks.insert(*false_block, block_id);
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
            Instruction::Return(_) |
            Instruction::Break(_) |
            Instruction::Continue(_) => ExecValue::Void,
            
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
                    .map(|arg| {
                        let arg_val = self.get_value(*arg);
                        // Arguments should already be expanded at SSA IR generation stage
                        arg_val.as_string()
                    })
                    .collect();
                
                let status = self.builtins.execute(name, &arg_strings, &mut self.env);
                
                // Update exit status in environment
                self.env.exit_status = status;
                
                // Check for errexit option
                if status != 0 && crate::modules::options::errexit_enabled() {
                    // errexit is enabled and command failed
                    // We should exit immediately
                    // For now, just return the error status
                    // In a full implementation, we would need to propagate this up
                }
                
                let value = ExecValue::ExitStatus(status);
                self.set_value(*result, value.clone());
                value
            }
            
            Instruction::CallExternal(cmd, args, result) => {
                let cmd_str = cmd.clone();
                let arg_strings: Vec<String> = args.iter()
                    .map(|arg| {
                        let arg_val = self.get_value(*arg);
                        // Arguments should already be expanded at SSA IR generation stage
                        arg_val.as_string()
                    })
                    .collect();
                
                // Check if it's a built-in command
                let status = if self.builtins.is_builtin(&cmd_str) {
                    self.builtins.execute(&cmd_str, &arg_strings, &mut self.env)
                } else {
                    self.execute_external_command(&cmd_str, &arg_strings)
                };
                
                // Update exit status in environment
                self.env.exit_status = status;
                
                // Check for errexit option
                if status != 0 && crate::modules::options::errexit_enabled() {
                    // errexit is enabled and command failed
                    // We should exit immediately
                    // For now, just return the error status
                    // In a full implementation, we would need to propagate this up
                }
                
                let value = ExecValue::ExitStatus(status);
                self.set_value(*result, value.clone());
                value
            }
            
            // Process operations
            Instruction::Fork(result) => {
                // Simulate fork by returning a dummy PID
                // In a real implementation, we would create a new process
                // For simulation purposes, we'll use a simple counter
                static FORK_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
                let pid = FORK_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                
                let value = ExecValue::ProcessId(pid);
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
            
            // Special operations
            Instruction::Nop => ExecValue::Void,
            Instruction::Error(msg, token) => {
                eprintln!("Error at {}:{}: {}", token.line, token.column, msg);
                ExecValue::ExitStatus(1)
            }
            Instruction::DebugPrint(value) => {
                let val = self.get_value(*value);
                println!("DEBUG: {} = {:?}", value, val);
                ExecValue::Void
            }
            
            // Parameter expansion
            Instruction::ParamExpand(param, op, result) => {
                let param_val = self.get_value(*param).as_string();
                let expanded = self.execute_param_expand(&param_val, op);
                let value = ExecValue::String(expanded);
                self.set_value(*result, value.clone());
                value
            }
            
            // TODO: Implement all other instructions
            _ => {
                // Placeholder for unimplemented instructions
                println!("WARNING: Unimplemented instruction: {:?}", instr);
                ExecValue::Void
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
        
        // Set environment variables from VariableSystem
        let vs = get_variable_system();
        let exported_vars = vs.get_exported_vars();
        for (key, value) in &exported_vars {
            command.env(key, value);
        }
        
        // Also set from env (for backward compatibility)
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
    
    /// Execute parameter expansion
    fn execute_param_expand(&self, param: &str, op: &crate::modules::ssa_ir::ParamExpandOp) -> String {
        let vs = get_variable_system();
        
        match op {
            crate::modules::ssa_ir::ParamExpandOp::Simple => {
                // Simple parameter expansion: ${parameter} or $parameter
                // Look up variable in environment
                if param == "?" {
                    return self.env.exit_status.to_string();
                } else if param == "$" {
                    return std::process::id().to_string();
                } else if param == "0" {
                    return "rs-dash-pro".to_string();
                } else if let Some(var) = vs.get(param) {
                    return var.value.clone();
                } else {
                    return String::new();
                }
            }
            crate::modules::ssa_ir::ParamExpandOp::UseDefault(word) => {
                // ${parameter:-word}
                if let Some(var) = vs.get(param) {
                    var.value.clone()
                } else {
                    word.clone()
                }
            }
            crate::modules::ssa_ir::ParamExpandOp::AssignDefault(word) => {
                // ${parameter:=word}
                let mut vs_mut = get_variable_system();
                if let Some(var) = vs_mut.get(param) {
                    var.value.clone()
                } else {
                    // Assign the value to environment
                    if let Err(e) = vs_mut.set(param.to_string(), word.clone()) {
                        eprintln!("Parameter expansion error: {}", e);
                        String::new()
                    } else {
                        word.clone()
                    }
                }
            }
            crate::modules::ssa_ir::ParamExpandOp::ErrorIfNull(word) => {
                // ${parameter:?word}
                if let Some(var) = vs.get(param) {
                    var.value.clone()
                } else {
                    eprintln!("{}", word);
                    String::new()
                }
            }
            crate::modules::ssa_ir::ParamExpandOp::UseAlternate(word) => {
                // ${parameter:+word}
                if vs.get(param).is_some() {
                    word.clone()
                } else {
                    String::new()
                }
            }
            crate::modules::ssa_ir::ParamExpandOp::Length => {
                // ${#parameter}
                if let Some(var) = vs.get(param) {
                    var.value.len().to_string()
                } else {
                    "0".to_string()
                }
            }
            crate::modules::ssa_ir::ParamExpandOp::RemoveSuffix(pattern) => {
                // ${parameter%pattern}
                if let Some(var) = vs.get(param) {
                    if var.value.ends_with(pattern) {
                        var.value[..var.value.len() - pattern.len()].to_string()
                    } else {
                        var.value.clone()
                    }
                } else {
                    String::new()
                }
            }
            crate::modules::ssa_ir::ParamExpandOp::RemoveLargestSuffix(pattern) => {
                // ${parameter%%pattern}
                if let Some(var) = vs.get(param) {
                    let mut result = var.value.clone();
                    while result.ends_with(pattern) {
                        result = result[..result.len() - pattern.len()].to_string();
                    }
                    result
                } else {
                    String::new()
                }
            }
            crate::modules::ssa_ir::ParamExpandOp::RemovePrefix(pattern) => {
                // ${parameter#pattern}
                if let Some(var) = vs.get(param) {
                    if var.value.starts_with(pattern) {
                        var.value[pattern.len()..].to_string()
                    } else {
                        var.value.clone()
                    }
                } else {
                    String::new()
                }
            }
            crate::modules::ssa_ir::ParamExpandOp::RemoveLargestPrefix(pattern) => {
                // ${parameter##pattern}
                if let Some(var) = vs.get(param) {
                    let mut result = var.value.clone();
                    while result.starts_with(pattern) {
                        result = result[pattern.len()..].to_string();
                    }
                    result
                } else {
                    String::new()
                }
            }
            crate::modules::ssa_ir::ParamExpandOp::Substring(offset) => {
                // ${parameter:offset}
                if let Some(var) = vs.get(param) {
                    let start = if *offset >= 0 {
                        *offset as usize
                    } else {
                        let len = var.value.len() as i32;
                        std::cmp::max(0, len + *offset) as usize
                    };
                    if start < var.value.len() {
                        var.value[start..].to_string()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            }
            crate::modules::ssa_ir::ParamExpandOp::SubstringWithLength(offset, length) => {
                // ${parameter:offset:length}
                if let Some(var) = vs.get(param) {
                    let start = if *offset >= 0 {
                        *offset as usize
                    } else {
                        let len = var.value.len() as i32;
                        std::cmp::max(0, len + *offset) as usize
                    };
                    if start < var.value.len() {
                        let end = std::cmp::min(var.value.len(), start + *length as usize);
                        var.value[start..end].to_string()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            }
            crate::modules::ssa_ir::ParamExpandOp::ReplaceFirst(pattern, replacement) => {
                // ${parameter/pattern/replacement}
                if let Some(var) = vs.get(param) {
                    var.value.replacen(pattern, replacement, 1)
                } else {
                    String::new()
                }
            }
            crate::modules::ssa_ir::ParamExpandOp::ReplaceAll(pattern, replacement) => {
                // ${parameter//pattern/replacement}
                if let Some(var) = vs.get(param) {
                    var.value.replace(pattern, replacement)
                } else {
                    String::new()
                }
            }
            crate::modules::ssa_ir::ParamExpandOp::ReplacePrefix(pattern, replacement) => {
                // ${parameter/#pattern/replacement}
                if let Some(var) = vs.get(param) {
                    if var.value.starts_with(pattern) {
                        replacement.clone() + &var.value[pattern.len()..]
                    } else {
                        var.value.clone()
                    }
                } else {
                    String::new()
                }
            }
            crate::modules::ssa_ir::ParamExpandOp::ReplaceSuffix(pattern, replacement) => {
                // ${parameter/%pattern/replacement}
                if let Some(var) = vs.get(param) {
                    if var.value.ends_with(pattern) {
                        var.value[..var.value.len() - pattern.len()].to_string() + replacement
                    } else {
                        var.value.clone()
                    }
                } else {
                    String::new()
                }
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