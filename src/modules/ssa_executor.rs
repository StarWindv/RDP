/// SSA IR Executor - executes SSA IR programs with proper shell semantics.
/// This module implements a virtual machine that executes SSA IR instructions,
/// handling process management, I/O redirection, variable expansion, and more.
use std::collections::HashMap;
use std::process::{Child, Command};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use crate::modules::builtins::Builtins;
use crate::modules::env::ShellEnv;
use crate::modules::ssa_ir::{BasicBlockId, CmpOp, Function, Instruction, ValueId, ValueType};
use crate::modules::variables::{get_variable_system, VarAttribute};

/// SSA IR Executor - executes SSA IR functions
pub struct SsaExecutor {
    builtins: Builtins,
    env: ShellEnv,
    functions: HashMap<String, Function>,
    values: HashMap<ValueId, ExecValue>,
    var_names: HashMap<ValueId, String>,          // Track which ValueId corresponds to which variable name
    current_function: Option<String>,
    current_block: Option<BasicBlockId>,
    program_counter: usize,
    call_stack: Vec<(String, BasicBlockId, usize)>, // (function, block, pc)
    predecessor_blocks: HashMap<BasicBlockId, BasicBlockId>, // block -> predecessor
    processes: HashMap<u32, Child>,                 // Track child processes by PID
    foreground_job: Option<usize>,                  // Current foreground job ID
    signal_handlers: HashMap<i32, BasicBlockId>,    // Signal number -> handler block
    loop_context: Vec<LoopContext>,                 // Stack of loop contexts for break/continue
}

/// Loop context for tracking break/continue targets
#[derive(Debug, Clone)]
struct LoopContext {
    exit_block: BasicBlockId,      // Target for break
    update_block: BasicBlockId,    // Target for continue
}

/// Executable value representation
#[derive(Debug, Clone)]
enum ExecValue {
    String(String),
    Integer(i32),
    Boolean(bool),
    FileDescriptor(i32),
    ProcessId(u32),
    ExitStatus(i32),
    Array(Vec<String>),
    Void,
}

impl ExecValue {
    /// Get the type of this value
    fn get_type(&self) -> ValueType {
        match self {
            ExecValue::String(_) => ValueType::String,
            ExecValue::Integer(_) => ValueType::Integer,
            ExecValue::Boolean(_) => ValueType::Boolean,
            ExecValue::FileDescriptor(_) => ValueType::FileDescriptor,
            ExecValue::ProcessId(_) => ValueType::ProcessId,
            ExecValue::ExitStatus(_) => ValueType::ExitStatus,
            ExecValue::Array(_) => ValueType::Array,
            ExecValue::Void => ValueType::Void,
        }
    }

    /// Convert to string representation
    fn as_string(&self) -> String {
        match self {
            ExecValue::String(s) => s.clone(),
            ExecValue::Integer(i) => i.to_string(),
            ExecValue::Boolean(b) => b.to_string(),
            ExecValue::FileDescriptor(fd) => fd.to_string(),
            ExecValue::ProcessId(pid) => pid.to_string(),
            ExecValue::ExitStatus(status) => status.to_string(),
            ExecValue::Array(arr) => arr.join(" "),
            ExecValue::Void => String::new(),
        }
    }

    /// Convert to integer representation
    fn as_integer(&self) -> i32 {
        match self {
            ExecValue::String(s) => s.parse().unwrap_or(0),
            ExecValue::Integer(i) => *i,
            ExecValue::Boolean(b) => {
                if *b {
                    1
                } else {
                    0
                }
            }
            ExecValue::FileDescriptor(fd) => *fd,
            ExecValue::ProcessId(pid) => *pid as i32,
            ExecValue::ExitStatus(status) => *status,
            ExecValue::Array(arr) => arr.len() as i32,
            ExecValue::Void => 0,
        }
    }

    /// Convert to boolean representation
    fn as_boolean(&self) -> bool {
        match self {
            ExecValue::String(s) => !s.is_empty(),
            ExecValue::Integer(i) => *i != 0,
            ExecValue::Boolean(b) => *b,
            ExecValue::FileDescriptor(fd) => *fd >= 0,
            ExecValue::ProcessId(pid) => *pid > 0,
            ExecValue::ExitStatus(status) => *status == 0,
            ExecValue::Array(arr) => !arr.is_empty(),
            ExecValue::Void => false,
        }
    }

    /// Convert to file descriptor (if applicable)
    fn as_fd(&self) -> Option<i32> {
        match self {
            ExecValue::FileDescriptor(fd) => Some(*fd),
            _ => None,
        }
    }

    /// Convert to process ID (if applicable)
    fn as_pid(&self) -> Option<u32> {
        match self {
            ExecValue::ProcessId(pid) => Some(*pid),
            _ => None,
        }
    }

    /// Convert to exit status (if applicable)
    fn as_status(&self) -> Option<i32> {
        match self {
            ExecValue::ExitStatus(status) => Some(*status),
            _ => None,
        }
    }

    /// Convert to array (if applicable)
    fn as_array(&self) -> Option<Vec<String>> {
        match self {
            ExecValue::Array(arr) => Some(arr.clone()),
            _ => None,
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
            var_names: HashMap::new(),
            current_function: None,
            current_block: None,
            program_counter: 0,
            call_stack: Vec::new(),
            predecessor_blocks: HashMap::new(),
            processes: HashMap::new(),
            foreground_job: None,
            signal_handlers: HashMap::new(),
            loop_context: Vec::new(),
        }
    }

    /// Register a user-defined function
    pub fn register_function(&mut self, name: String, func: Function) {
        self.functions.insert(name, func);
    }

    /// Execute a function
    pub fn execute_function(&mut self, func: &Function) -> i32 {
        // Store function
        self.functions.insert(func.name.clone(), func.clone());

        // Set up execution context
        self.current_function = Some(func.name.clone());
        self.current_block = Some(func.entry_block);
        self.program_counter = 0;
        self.call_stack.clear();
        self.predecessor_blocks.clear();
        self.var_names.clear(); // Clear variable name mapping for new function

        // Execute
        let result = self.execute_block(func.entry_block, func);

        // Clean up
        self.current_function = None;
        self.current_block = None;
        self.predecessor_blocks.clear();

        result.as_status().unwrap_or(0)
    }

    /// Execute a basic block
    fn execute_block(&mut self, block_id: BasicBlockId, func: &Function) -> ExecValue {
        let block = func.get_block(block_id).expect("Block should exist");
        self.current_block = Some(block_id);
        self.program_counter = 0;

        // Detect and manage loop contexts based on block labels
        if let Some(label) = &block.label {
            if label.contains("_body") && !label.contains("_exit") {
                // Entering a loop body - set up context if this is a known loop type
                if label.contains("while_body") || label.contains("until_body") || label.contains("for_body") {
                    // Find the corresponding exit block
                    if let Some(exit_block) = self.find_exit_block(func, label) {
                        // Find the update/condition block
                        if let Some(update_block) = self.find_update_block(func, label) {
                            self.loop_context.push(LoopContext {
                                exit_block,
                                update_block,
                            });
                        }
                    }
                }
            }
        }

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
                Instruction::Break(level) => {
                    // Get the loop context level (default 1 for innermost loop)
                    let loop_level = level.unwrap_or(1) as usize;
                    if loop_level > 0 && loop_level <= self.loop_context.len() {
                        let idx = self.loop_context.len() - loop_level;
                        let exit_block = self.loop_context[idx].exit_block;
                        self.predecessor_blocks.insert(exit_block, block_id);
                        return self.execute_block(exit_block, func);
                    } else {
                        eprintln!("Warning: break without enclosing loop");
                        return ExecValue::ExitStatus(1);
                    }
                }
                Instruction::Continue(level) => {
                    // Get the loop context level (default 1 for innermost loop)
                    let loop_level = level.unwrap_or(1) as usize;
                    if loop_level > 0 && loop_level <= self.loop_context.len() {
                        let idx = self.loop_context.len() - loop_level;
                        let update_block = self.loop_context[idx].update_block;
                        self.predecessor_blocks.insert(update_block, block_id);
                        return self.execute_block(update_block, func);
                    } else {
                        eprintln!("Warning: continue without enclosing loop");
                        return ExecValue::ExitStatus(1);
                    }
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
            Instruction::Jump(_)
            | Instruction::Branch(_, _, _)
            | Instruction::Return(_)
            | Instruction::Break(_)
            | Instruction::Continue(_) => ExecValue::Void,

            // Variable operations
            Instruction::AllocVar(name, result) => {
                let value = ExecValue::String(String::new());
                self.set_value(*result, value.clone());
                self.var_names.insert(*result, name.clone()); // Track this ValueId -> variable name
                value
            }

            Instruction::Store(var, value) => {
                let val = self.get_value(*value);
                // Store in values map
                self.set_value(*var, val.clone());
                
                // Also update the variable system if this is a tracked variable
                if let Some(var_name) = self.var_names.get(var).cloned() {
                    let mut vs = get_variable_system();
                    let _ = vs.set(var_name, val.as_string());
                }
                ExecValue::Void
            }

            Instruction::Load(var, result) => {
                let val = self.get_value(*var);
                self.set_value(*result, val.clone());
                val
            }

            // Command execution
            Instruction::CallBuiltin(name, args, result) => {
                let arg_strings: Vec<String> = args
                    .iter()
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
                let arg_strings: Vec<String> = args
                    .iter()
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
                // Real fork implementation
                let pid = match self.fork_process() {
                    Ok(pid) => pid,
                    Err(e) => {
                        eprintln!("fork failed: {}", e);
                        let value = ExecValue::ProcessId(0); // 0 indicates fork failure
                        self.set_value(*result, value.clone());
                        return value;
                    }
                };

                let value = ExecValue::ProcessId(pid);
                self.set_value(*result, value.clone());
                value
            }

            Instruction::Exec(pid, cmd, args) => {
                // Execute command in forked process
                let pid_val = self.get_value(*pid).as_pid();
                if let Some(pid) = pid_val {
                    if pid == 0 {
                        // Child process - execute command
                        let cmd_str = cmd.clone();
                        let arg_strings: Vec<String> = args
                            .iter()
                            .map(|arg| self.get_value(*arg).as_string())
                            .collect();

                        // Execute command
                        let status = self.execute_external_command_in_child(&cmd_str, &arg_strings);
                        return ExecValue::ExitStatus(status);
                    } else {
                        // Parent process - just return success
                        return ExecValue::ExitStatus(0);
                    }
                }
                ExecValue::ExitStatus(1)
            }

            Instruction::Wait(pid, result) => {
                let pid_val = self.get_value(*pid).as_pid();
                let status = if let Some(pid) = pid_val {
                    self.wait_for_process(pid)
                } else {
                    1 // Error
                };

                let value = ExecValue::ExitStatus(status);
                self.set_value(*result, value.clone());
                value
            }

            Instruction::Exit(status) => {
                let status_val = self.get_value(*status).as_status();
                ExecValue::ExitStatus(status_val.unwrap_or(0))
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

            // Function call operations
            Instruction::CallFunction(func_name, args, result) => {
                // Get function name
                let func_name_val = func_name.clone();

                // Check if it's a built-in function
                if self.builtins.is_builtin(&func_name_val) {
                    // Execute as builtin
                    let arg_strings: Vec<String> = args
                        .iter()
                        .map(|arg| self.get_value(*arg).as_string())
                        .collect();

                    let status = self
                        .builtins
                        .execute(&func_name_val, &arg_strings, &mut self.env);
                    let value = ExecValue::ExitStatus(status);
                    self.set_value(*result, value.clone());
                    return value;
                }

                // Look up user-defined function
                if let Some(func) = self.functions.get(&func_name_val) {
                    // Clone the function to avoid borrowing issues
                    let func_clone = func.clone();

                    // Enter function scope
                    let mut vs = get_variable_system();
                    vs.enter_scope();

                    // Set positional parameters
                    // $0 is function name
                    vs.set("0".to_string(), func_name_val.clone())
                        .unwrap_or_else(|e| eprintln!("Failed to set $0: {}", e));

                    // Set arguments $1, $2, etc.
                    for (i, arg_id) in args.iter().enumerate() {
                        let arg_val = self.get_value(*arg_id).as_string();
                        let param_name = (i + 1).to_string();
                        vs.set(param_name, arg_val)
                            .unwrap_or_else(|e| eprintln!("Failed to set ${}: {}", i + 1, e));
                    }

                    // Set special parameter $# (number of arguments)
                    vs.set("#".to_string(), args.len().to_string())
                        .unwrap_or_else(|e| eprintln!("Failed to set $#: {}", e));

                    // Set special parameter $* and $@ (all arguments)
                    let all_args: Vec<String> = args
                        .iter()
                        .map(|arg_id| self.get_value(*arg_id).as_string())
                        .collect();
                    let all_args_str = all_args.join(" ");
                    vs.set("*".to_string(), all_args_str.clone())
                        .unwrap_or_else(|e| eprintln!("Failed to set $*: {}", e));
                    vs.set("@".to_string(), all_args_str)
                        .unwrap_or_else(|e| eprintln!("Failed to set $@: {}", e));

                    // Save current context
                    let old_function = self.current_function.clone();
                    let old_block = self.current_block;
                    let old_pc = self.program_counter;
                    let old_stack = self.call_stack.clone();

                    // Push return address onto call stack
                    if let Some(current_func) = &old_function {
                        if let Some(current_block) = old_block {
                            self.call_stack
                                .push((current_func.clone(), current_block, old_pc));
                        }
                    }

                    // Execute function
                    let result_value = self.execute_block(func_clone.entry_block, &func_clone);

                    // Exit function scope
                    vs.exit_scope()
                        .unwrap_or_else(|e| eprintln!("Failed to exit function scope: {}", e));

                    // Restore context
                    self.current_function = old_function;
                    self.current_block = old_block;
                    self.program_counter = old_pc;
                    self.call_stack = old_stack;

                    // Set result value
                    self.set_value(*result, result_value.clone());
                    result_value
                } else {
                    // Function not found
                    eprintln!("{}: command not found", func_name_val);
                    let value = ExecValue::ExitStatus(127);
                    self.set_value(*result, value.clone());
                    value
                }
            }

            // Export, unset, readonly variable operations
            Instruction::ExportVar(var) => {
                let var_name = self.get_value(*var).as_string();
                let vs = get_variable_system();
                let vs_mut = vs;
                if let Some(var_data) = vs_mut.get(&var_name) {
                    // We need to clone and modify, then set back
                    let mut new_var = var_data.clone();
                    new_var.add_attribute(VarAttribute::Exported);
                    // Since we can't get mutable reference, we need to remove and insert
                    // For now, just skip - we'll handle this properly later
                }
                ExecValue::Void
            }

            Instruction::UnsetVar(var) => {
                let var_name = self.get_value(*var).as_string();
                let vs = get_variable_system();
                let mut vs_mut = vs;
                let _ = vs_mut.unset(&var_name);
                ExecValue::Void
            }

            Instruction::ReadonlyVar(var) => {
                let var_name = self.get_value(*var).as_string();
                let vs = get_variable_system();
                let vs_mut = vs;
                if let Some(var_data) = vs_mut.get(&var_name) {
                    // We need to clone and modify, then set back
                    let mut new_var = var_data.clone();
                    new_var.add_attribute(VarAttribute::ReadOnly);
                    // Since we can't get mutable reference, we need to remove and insert
                    // For now, just skip - we'll handle this properly later
                }
                ExecValue::Void
            }

            // Kill and trap operations
            Instruction::Kill(pid, signal) => {
                let pid_val = self.get_value(*pid).as_pid();
                let status = if let Some(pid) = pid_val {
                    self.kill_process(pid, *signal)
                } else {
                    1 // Error
                };
                ExecValue::ExitStatus(status)
            }

            Instruction::Trap(signal, handler) => {
                let signal_val = self.get_value(*signal).as_integer();
                println!(
                    "Setting trap for signal {} to handler block {}",
                    signal_val, handler.0
                );
                // Store the signal handler
                self.signal_handlers.insert(signal_val, *handler);
                ExecValue::Void
            }

            // Here document
            Instruction::HereDoc(_content, result) => {
                // TODO: Implement here document
                // For now, create a dummy file descriptor
                let value = ExecValue::FileDescriptor(5);
                self.set_value(*result, value.clone());
                value
            }

            // Pattern matching and glob expansion
            Instruction::PatternMatch(str, pattern, result) => {
                let s = self.get_value(*str).as_string();
                // Use glob pattern matching
                let matches = self.matches_pattern(&s, pattern);
                let value = ExecValue::Boolean(matches);
                self.set_value(*result, value.clone());
                value
            }

            Instruction::GlobExpand(pattern, result) => {
                // TODO: Implement glob expansion
                // For now, just return the pattern as a string
                let pattern_val = self.get_value(*pattern).as_string();
                let value = ExecValue::String(pattern_val);
                self.set_value(*result, value.clone());
                value
            }

            // Bit operations
            Instruction::BitAnd(a, b, result) => {
                let a_val = self.get_value(*a).as_integer();
                let b_val = self.get_value(*b).as_integer();
                let value = ExecValue::Integer(a_val & b_val);
                self.set_value(*result, value.clone());
                value
            }

            Instruction::BitOr(a, b, result) => {
                let a_val = self.get_value(*a).as_integer();
                let b_val = self.get_value(*b).as_integer();
                let value = ExecValue::Integer(a_val | b_val);
                self.set_value(*result, value.clone());
                value
            }

            Instruction::BitXor(a, b, result) => {
                let a_val = self.get_value(*a).as_integer();
                let b_val = self.get_value(*b).as_integer();
                let value = ExecValue::Integer(a_val ^ b_val);
                self.set_value(*result, value.clone());
                value
            }

            Instruction::BitNot(a, result) => {
                let a_val = self.get_value(*a).as_integer();
                let value = ExecValue::Integer(!a_val);
                self.set_value(*result, value.clone());
                value
            }

            Instruction::ShiftLeft(a, b, result) => {
                let a_val = self.get_value(*a).as_integer();
                let b_val = self.get_value(*b).as_integer();
                let value = ExecValue::Integer(a_val << b_val);
                self.set_value(*result, value.clone());
                value
            }

            Instruction::ShiftRight(a, b, result) => {
                let a_val = self.get_value(*a).as_integer();
                let b_val = self.get_value(*b).as_integer();
                let value = ExecValue::Integer(a_val >> b_val);
                self.set_value(*result, value.clone());
                value
            }

            Instruction::Neg(a, result) => {
                let a_val = self.get_value(*a).as_integer();
                let value = ExecValue::Integer(-a_val);
                self.set_value(*result, value.clone());
                value
            }

            // Array operations
            Instruction::CreateArray(result) => {
                // TODO: Implement array
                let value = ExecValue::String(String::new());
                self.set_value(*result, value.clone());
                value
            }

            Instruction::ArraySet(_array, _index, _value) => {
                // TODO: Implement array set properly
                ExecValue::Void
            }

            Instruction::ArrayGet(array, index, result) => {
                let arr = self.get_value(*array);
                let idx = self.get_value(*index).as_integer() as usize;
                
                if let ExecValue::Array(arr_vec) = arr {
                    let value = if idx < arr_vec.len() {
                        ExecValue::String(arr_vec[idx].clone())
                    } else {
                        ExecValue::String(String::new()) // Out of bounds returns empty string
                    };
                    self.set_value(*result, value.clone());
                    value
                } else {
                    let value = ExecValue::String(String::new());
                    self.set_value(*result, value.clone());
                    value
                }
            }

            Instruction::ArrayLength(array, result) => {
                let arr = self.get_value(*array);
                let len = match arr {
                    ExecValue::Array(arr_vec) => arr_vec.len() as i32,
                    _ => 0,
                };
                let value = ExecValue::Integer(len);
                self.set_value(*result, value.clone());
                value
            }

            Instruction::ArrayKeys(_array, result) => {
                // TODO: Implement array keys
                let value = ExecValue::String(String::new());
                self.set_value(*result, value.clone());
                value
            }

            Instruction::ConstArray(values, result) => {
                let value = ExecValue::Array(values.clone());
                self.set_value(*result, value.clone());
                value
            }

            // Command substitution
            Instruction::CommandSub(cmd_result, result) => {
                // TODO: Implement command substitution
                // For now, just return exit status as string
                let status = self.get_value(*cmd_result).as_status();
                let value = ExecValue::String(status.map(|s| s.to_string()).unwrap_or_default());
                self.set_value(*result, value.clone());
                value
            }

            // Parameter expansion
            Instruction::ParamExpand(param, op, result) => {
                let param_val = self.get_value(*param);
                let param_str = param_val.as_string();
                let expanded = self.execute_param_expand(&param_str, op);
                let value = ExecValue::String(expanded);
                self.set_value(*result, value.clone());
                value
            }

            // Handle any other instructions not explicitly matched
            _ => {
                eprintln!("Warning: Unimplemented instruction: {:?}", instr);
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
    fn execute_param_expand(
        &self,
        param: &str,
        op: &crate::modules::ssa_ir::ParamExpandOp,
    ) -> String {
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
        self.values
            .get(&id)
            .cloned()
            .unwrap_or_else(|| ExecValue::Void)
    }

    /// Set a value by ID
    fn set_value(&mut self, id: ValueId, value: ExecValue) {
        self.values.insert(id, value);
    }

    // ============================================
    // Process Management Methods
    // ============================================

    /// Fork a new process
    fn fork_process(&mut self) -> Result<u32, String> {
        #[cfg(unix)]
        {
            use nix::unistd::fork;
            use nix::unistd::ForkResult;

            unsafe {
                match fork() {
                    Ok(ForkResult::Parent { child, .. }) => {
                        let pid = child.as_raw() as u32;
                        // Store child process (we'll get it when we wait)
                        // For now, just store placeholder
                        self.processes.insert(
                            pid,
                            Command::new("true").spawn().map_err(|e| e.to_string())?,
                        );
                        Ok(pid)
                    }
                    Ok(ForkResult::Child) => {
                        // Child process - return 0 to indicate child
                        Ok(0)
                    }
                    Err(e) => Err(format!("fork failed: {}", e)),
                }
            }
        }

        #[cfg(not(unix))]
        {
            // On Windows, we can't fork, so we simulate it
            // We'll spawn a new process and return its PID
            let child = Command::new("cmd")
                .arg("/c")
                .arg("echo")
                .arg("fork simulation")
                .spawn()
                .map_err(|e| e.to_string())?;

            let pid = child.id() as u32;
            self.processes.insert(pid, child);
            Ok(pid)
        }
    }

    /// Execute external command in child process
    fn execute_external_command_in_child(&self, cmd: &str, args: &[String]) -> i32 {
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

        #[cfg(unix)]
        {
            // Use exec to replace current process
            use std::os::unix::process::CommandExt;
            let error = command.exec();
            eprintln!("exec failed: {}", error);
            std::process::exit(1);
        }

        #[cfg(not(unix))]
        {
            // On Windows, we can't exec, so we spawn and exit
            match command.spawn() {
                Ok(mut child) => match child.wait() {
                    Ok(status) => status.code().unwrap_or(1),
                    Err(e) => {
                        eprintln!("wait failed: {}", e);
                        1
                    }
                },
                Err(e) => {
                    eprintln!("spawn failed: {}", e);
                    1
                }
            }
        }
    }

    /// Wait for a process to complete
    fn wait_for_process(&mut self, pid: u32) -> i32 {
        if let Some(mut child) = self.processes.remove(&pid) {
            match child.wait() {
                Ok(status) => status.code().unwrap_or(128),
                Err(e) => {
                    eprintln!("wait failed: {}", e);
                    1
                }
            }
        } else {
            eprintln!("process {} not found", pid);
            1
        }
    }

    // ============================================
    // Signal Handling Methods
    // ============================================

    /// Check for pending signals and execute handlers
    fn check_signals(&mut self, func: &Function) -> bool {
        // For now, we don't have actual signal delivery
        // In a real implementation, we would check a signal queue
        false
    }

    /// Kill a process
    fn kill_process(&mut self, pid: u32, signal: i32) -> i32 {
        if let Some(child) = self.processes.get_mut(&pid) {
            #[cfg(unix)]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;

                let nix_signal = match signal {
                    2 => Signal::SIGINT,
                    9 => Signal::SIGKILL,
                    15 => Signal::SIGTERM,
                    _ => Signal::SIGTERM,
                };

                match kill(Pid::from_raw(pid as i32), nix_signal) {
                    Ok(_) => 0,
                    Err(e) => {
                        eprintln!("kill failed: {}", e);
                        1
                    }
                }
            }

            #[cfg(not(unix))]
            {
                // On Windows, try to kill the process
                let _ = child.kill();
                0
            }
        } else {
            eprintln!("process {} not found", pid);
            1
        }
    }

    // ============================================
    // Loop Context Management
    // ============================================

    /// Check if a word matches a shell glob pattern
    fn matches_pattern(&self, word: &str, pattern: &str) -> bool {
        // Handle the exact pattern matching for shell case statements
        // Supports * (any characters), ? (single character), and [...] (character class)
        self.pattern_match_recursive(word.chars().collect(), pattern.chars().collect())
    }

    fn pattern_match_recursive(&self, word: Vec<char>, pattern: Vec<char>) -> bool {
        let mut w_idx = 0;
        let mut p_idx = 0;

        while w_idx < word.len() && p_idx < pattern.len() {
            match pattern[p_idx] {
                '*' => {
                    // Try to match zero or more characters
                    if p_idx + 1 == pattern.len() {
                        return true; // * at end matches everything
                    }

                    // Try to match the rest of the pattern
                    let rest_pattern = pattern[p_idx + 1..].to_vec();
                    for k in w_idx..=word.len() {
                        let rest_word = word[k..].to_vec();
                        if self.pattern_match_recursive(rest_word, rest_pattern.clone()) {
                            return true;
                        }
                    }
                    return false;
                }
                '?' => {
                    // Match any single character
                    w_idx += 1;
                    p_idx += 1;
                }
                '[' => {
                    // Character class matching - simple implementation
                    // Find the closing ]
                    let mut close_idx = p_idx + 1;
                    while close_idx < pattern.len() && pattern[close_idx] != ']' {
                        close_idx += 1;
                    }

                    if close_idx < pattern.len() {
                        // Extract character class
                        let char_class = &pattern[p_idx + 1..close_idx];
                        if w_idx < word.len() {
                            let matches = char_class.contains(&word[w_idx]);
                            if matches {
                                w_idx += 1;
                                p_idx = close_idx + 1;
                            } else {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    } else {
                        // No closing ], treat [ as literal
                        if w_idx < word.len() && word[w_idx] == '[' {
                            w_idx += 1;
                            p_idx += 1;
                        } else {
                            return false;
                        }
                    }
                }
                c => {
                    // Match exact character
                    if w_idx < word.len() && word[w_idx] == c {
                        w_idx += 1;
                        p_idx += 1;
                    } else {
                        return false;
                    }
                }
            }
        }

        // Special case: pattern ends with * and we've consumed all word
        if p_idx < pattern.len() && pattern[p_idx] == '*' && w_idx == word.len() {
            p_idx += 1;
        }

        // Check if we've consumed all of both strings
        w_idx == word.len() && p_idx == pattern.len()
    }

    /// Find the exit block for a loop given its label
    fn find_exit_block(&self, func: &Function, loop_body_label: &str) -> Option<BasicBlockId> {
        // Extract loop name from label (e.g., "while_body" -> "while", "for_body_i" -> "for_i")
        let loop_name = if loop_body_label.ends_with("_body") {
            &loop_body_label[..loop_body_label.len() - 5] // Remove "_body"
        } else {
            loop_body_label
        };

        // Look for the corresponding exit block
        let exit_label = format!("{}_exit", loop_name);
        for block in func.blocks.values() {
            if let Some(label) = &block.label {
                if label == &exit_label {
                    return Some(block.id);
                }
            }
        }
        None
    }

    /// Find the update/condition block for a loop given its label
    fn find_update_block(&self, func: &Function, loop_body_label: &str) -> Option<BasicBlockId> {
        // Extract loop name from label
        let loop_name = if loop_body_label.ends_with("_body") {
            &loop_body_label[..loop_body_label.len() - 5]
        } else {
            loop_body_label
        };

        // Look for the condition or update block (depends on loop type)
        // Try condition first (while_cond, until_cond, for_cond)
        let cond_label = format!("{}_cond", loop_name);
        for block in func.blocks.values() {
            if let Some(label) = &block.label {
                if label == &cond_label {
                    return Some(block.id);
                }
            }
        }

        // Try update block for for loops (for_update_i)
        let update_label = format!("{}_update", loop_name);
        for block in func.blocks.values() {
            if let Some(label) = &block.label {
                if label == &update_label {
                    return Some(block.id);
                }
            }
        }

        None
    }

    /// Extract loop metadata from block labels to set up loop context
    fn setup_loop_context(&mut self, func: &Function, block_id: BasicBlockId) {
        let block = func.get_block(block_id);
        if let Some(block) = block {
            if let Some(label) = &block.label {
                // Detect loop entry blocks and set up context
                if label.contains("_cond") || label.contains("_body") {
                    // This might be a loop block, try to find exit and update blocks
                    // For now, we'll rely on the generator to provide proper structure
                    // This is a simplification; a real implementation would be more sophisticated
                }
            }
        }
    }

    /// Get the loop context for break/continue handling
    fn get_loop_exit_block(&self, level: usize) -> Option<BasicBlockId> {
        if level > 0 && level <= self.loop_context.len() {
            let idx = self.loop_context.len() - level;
            Some(self.loop_context[idx].exit_block)
        } else {
            None
        }
    }

    /// Get the loop update block for continue
    fn get_loop_update_block(&self, level: usize) -> Option<BasicBlockId> {
        if level > 0 && level <= self.loop_context.len() {
            let idx = self.loop_context.len() - level;
            Some(self.loop_context[idx].update_block)
        } else {
            None
        }
    }
}
