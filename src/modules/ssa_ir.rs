//! Static Single Assignment (SSA) Intermediate Representation for POSIX Shell
//!
//! This module defines the SSA IR used by rs-dash-pro. SSA (Static Single Assignment)
//! form ensures that each variable is assigned exactly once, which enables
//! powerful optimizations and simplifies analysis.

use std::collections::{HashMap, BTreeMap};
use std::fmt;

use crate::modules::tokens::Token;

// ============================================================================
// Core Types
// ============================================================================

/// Unique identifier for values in SSA form
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ValueId(pub usize);

impl fmt::Display for ValueId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

/// Types of values in shell
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueType {
    String,
    Integer,
    Boolean,
    FileDescriptor,
    ProcessId,
    ExitStatus,
    Array,
    Void,
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValueType::String => write!(f, "string"),
            ValueType::Integer => write!(f, "int"),
            ValueType::Boolean => write!(f, "bool"),
            ValueType::FileDescriptor => write!(f, "fd"),
            ValueType::ProcessId => write!(f, "pid"),
            ValueType::ExitStatus => write!(f, "status"),
            ValueType::Array => write!(f, "array"),
            ValueType::Void => write!(f, "void"),
        }
    }
}

/// Value in SSA form with type information
#[derive(Debug, Clone)]
pub struct Value {
    pub id: ValueId,
    pub ty: ValueType,
    pub name: Option<String>,
}

impl Value {
    /// Create a new value with the given type
    pub fn new(id: ValueId, ty: ValueType) -> Self {
        Self {
            id,
            ty,
            name: None,
        }
    }
    
    /// Create a new value with a name
    pub fn with_name(id: ValueId, ty: ValueType, name: String) -> Self {
        Self {
            id,
            ty,
            name: Some(name),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "%{}:{} '{}'", self.id.0, self.ty, name)
        } else {
            write!(f, "%{}:{}", self.id.0, self.ty)
        }
    }
}

// ============================================================================
// Instructions
// ============================================================================

/// SSA instruction representing a single operation
#[derive(Debug, Clone)]
pub enum Instruction {
    // ============================================
    // Control flow instructions
    // ============================================
    
    /// Unconditional jump to a basic block
    Jump(BasicBlockId),
    
    /// Conditional branch based on a boolean value
    Branch(ValueId, BasicBlockId, BasicBlockId), // cond, true_block, false_block
    
    /// Return from function with exit status
    Return(ValueId),
    
    /// Break out of a loop with optional level
    Break(Option<i32>),    // break [n]
    
    /// Continue to next iteration of a loop with optional level
    Continue(Option<i32>), // continue [n]
    
    // ============================================
    // Variable and environment operations
    // ============================================
    
    /// Allocate a new variable with given name
    AllocVar(String, ValueId), // name, result
    
    /// Store a value into a variable
    Store(ValueId, ValueId),   // var, value
    
    /// Load a value from a variable
    Load(ValueId, ValueId),    // var, result
    
    /// Mark a variable as exported
    ExportVar(ValueId),        // export variable
    
    /// Remove a variable from environment
    UnsetVar(ValueId),         // unset variable
    
    /// Mark a variable as read-only
    ReadonlyVar(ValueId),      // readonly variable
    
    // ============================================
    // Command execution
    // ============================================
    
    /// Call a built-in shell command
    CallBuiltin(String, Vec<ValueId>, ValueId), // name, args, result(status)
    
    /// Call an external command
    CallExternal(String, Vec<ValueId>, ValueId), // cmd, args, result(status)
    
    /// Call a user-defined function
    CallFunction(String, Vec<ValueId>, ValueId), // function call
    
    // ============================================
    // Process and job control
    // ============================================
    
    /// Fork a new process
    Fork(ValueId), // result(pid)
    
    /// Execute a command in a forked process
    Exec(ValueId, String, Vec<ValueId>), // pid, cmd, args
    
    /// Wait for a process to complete
    Wait(ValueId, ValueId), // pid, result(status)
    
    /// Exit with given status
    Exit(ValueId), // status
    
    /// Send signal to a process
    Kill(ValueId, i32), // pid, signal
    
    /// Set signal handler
    Trap(ValueId, BasicBlockId), // signal, handler_block
    
    // ============================================
    // Pipeline and redirection operations
    // ============================================
    
    /// Create a pipe for inter-process communication
    CreatePipe(ValueId, ValueId), // result(read_fd), result(write_fd)
    
    /// Duplicate a file descriptor
    DupFd(ValueId, i32, ValueId), // old_fd, new_fd, result
    
    /// Close a file descriptor
    CloseFd(ValueId),
    
    /// Redirect input/output to/from a file or device
    Redirect(ValueId, String, RedirectMode), // fd, target, mode
    
    /// Create a here-document
    HereDoc(String, ValueId), // content, result(fd)
    
    // ============================================
    // String and pattern matching operations
    // ============================================
    
    /// Concatenate two strings
    Concat(ValueId, ValueId, ValueId), // str1, str2, result
    
    /// Extract substring from a string
    Substr(ValueId, ValueId, ValueId, ValueId), // str, start, len, result
    
    /// Get length of a string
    Length(ValueId, ValueId), // str, result
    
    /// Pattern matching using shell glob patterns
    PatternMatch(ValueId, String, ValueId), // str, pattern, result(bool)
    
    /// Expand glob patterns to matching filenames
    GlobExpand(ValueId, ValueId), // pattern, result(list)
    
    // ============================================
    // Arithmetic operations
    // ============================================
    
    /// Integer addition
    Add(ValueId, ValueId, ValueId), // a, b, result
    
    /// Integer subtraction
    Sub(ValueId, ValueId, ValueId),
    
    /// Integer multiplication
    Mul(ValueId, ValueId, ValueId),
    
    /// Integer division
    Div(ValueId, ValueId, ValueId),
    
    /// Integer modulo
    Mod(ValueId, ValueId, ValueId),
    
    /// Integer negation
    Neg(ValueId, ValueId), // a, result
    
    /// Bitwise AND
    BitAnd(ValueId, ValueId, ValueId),
    
    /// Bitwise OR
    BitOr(ValueId, ValueId, ValueId),
    
    /// Bitwise XOR
    BitXor(ValueId, ValueId, ValueId),
    
    /// Bitwise NOT
    BitNot(ValueId, ValueId),
    
    /// Bitwise shift left
    ShiftLeft(ValueId, ValueId, ValueId),
    
    /// Bitwise shift right
    ShiftRight(ValueId, ValueId, ValueId),
    
    // ============================================
    // Logical and comparison operations
    // ============================================
    
    /// Logical AND
    And(ValueId, ValueId, ValueId), // a, b, result
    
    /// Logical OR
    Or(ValueId, ValueId, ValueId),
    
    /// Logical NOT
    Not(ValueId, ValueId), // a, result
    
    /// Numeric comparison
    Cmp(ValueId, ValueId, CmpOp, ValueId), // a, b, op, result
    
    // ============================================
    // Array and list operations
    // ============================================
    
    /// Create a new array
    CreateArray(ValueId), // result(array)
    
    /// Set element in array
    ArraySet(ValueId, ValueId, ValueId), // array, index, value
    
    /// Get element from array
    ArrayGet(ValueId, ValueId, ValueId), // array, index, result
    
    /// Get length of array
    ArrayLength(ValueId, ValueId), // array, result
    
    /// Get all keys/indices of array
    ArrayKeys(ValueId, ValueId), // array, result(list)
    
    // ============================================
    // Command substitution and parameter expansion
    // ============================================
    
    /// Execute command and capture output
    CommandSub(ValueId, ValueId), // command_result, result(string)
    
    /// Parameter expansion with various operations
    ParamExpand(ValueId, ParamExpandOp, ValueId), // parameter, operation, result
    
    // ============================================
    // Constants and literals
    // ============================================
    
    /// String constant
    ConstString(String, ValueId), // value, result
    
    /// Integer constant
    ConstInt(i32, ValueId),       // value, result
    
    /// Boolean constant
    ConstBool(bool, ValueId),     // value, result
    
    /// Array constant
    ConstArray(Vec<String>, ValueId), // values, result(array)
    
    // ============================================
    // SSA-specific operations
    // ============================================
    
    /// Phi function for SSA form - selects value based on predecessor block
    Phi(Vec<(BasicBlockId, ValueId)>, ValueId), // incoming values, result
    
    // ============================================
    // Special operations
    // ============================================
    
    /// No operation
    Nop,
    
    /// Error with message and location
    Error(String, Token),
    
    /// Debug print for development
    DebugPrint(ValueId), // value to print for debugging
}

/// Redirection modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedirectMode {
    Read,           // <
    Write,          // >
    Append,         // >>
    ReadWrite,      // <>
    DupRead,        // <&
    DupWrite,       // >&
    HereDoc,        // <<
    HereDocStrip,   // <<-
}

/// Comparison operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Eq,  // ==
    Ne,  // !=
    Lt,  // <
    Le,  // <=
    Gt,  // >
    Ge,  // >=
}

/// Parameter expansion operations
#[derive(Debug, Clone)]
pub enum ParamExpandOp {
    // ${parameter}
    Simple,
    // ${parameter:-word} - Use default value
    UseDefault(String),
    // ${parameter:=word} - Assign default value
    AssignDefault(String),
    // ${parameter:?word} - Error if null or unset
    ErrorIfNull(String),
    // ${parameter:+word} - Use alternate value
    UseAlternate(String),
    // ${#parameter} - String length
    Length,
    // ${parameter%word} - Remove smallest suffix pattern
    RemoveSuffix(String),
    // ${parameter%%word} - Remove largest suffix pattern
    RemoveLargestSuffix(String),
    // ${parameter#word} - Remove smallest prefix pattern
    RemovePrefix(String),
    // ${parameter##word} - Remove largest prefix pattern
    RemoveLargestPrefix(String),
    // ${parameter:offset} - Substring from offset
    Substring(i32),
    // ${parameter:offset:length} - Substring with length
    SubstringWithLength(i32, i32),
    // ${parameter/pattern/string} - Replace first match
    ReplaceFirst(String, String),
    // ${parameter//pattern/string} - Replace all matches
    ReplaceAll(String, String),
    // ${parameter/#pattern/string} - Replace prefix match
    ReplacePrefix(String, String),
    // ${parameter/%pattern/string} - Replace suffix match
    ReplaceSuffix(String, String),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Instruction::Jump(block) => write!(f, "jump .{}", block.0),
            
            Instruction::Branch(cond, true_block, false_block) => {
                write!(f, "branch {} .{} .{}", cond, true_block.0, false_block.0)
            }
            
            Instruction::Return(status) => write!(f, "return {}", status),
            
            Instruction::Break(n) => {
                if let Some(n) = n {
                    write!(f, "break {}", n)
                } else {
                    write!(f, "break")
                }
            }
            
            Instruction::Continue(n) => {
                if let Some(n) = n {
                    write!(f, "continue {}", n)
                } else {
                    write!(f, "continue")
                }
            }
            
            Instruction::AllocVar(name, result) => {
                write!(f, "{} = alloc_var '{}'", result, name)
            }
            
            Instruction::Store(var, value) => {
                write!(f, "store {} {}", var, value)
            }
            
            Instruction::Load(var, result) => {
                write!(f, "{} = load {}", result, var)
            }
            
            Instruction::ExportVar(var) => {
                write!(f, "export {}", var)
            }
            
            Instruction::UnsetVar(var) => {
                write!(f, "unset {}", var)
            }
            
            Instruction::ReadonlyVar(var) => {
                write!(f, "readonly {}", var)
            }
            
            Instruction::CallBuiltin(name, args, result) => {
                write!(f, "{} = call_builtin '{}'", result, name)?;
                for arg in args {
                    write!(f, " {}", arg)?;
                }
                Ok(())
            }
            
            Instruction::CallExternal(cmd, args, result) => {
                write!(f, "{} = call_external '{}'", result, cmd)?;
                for arg in args {
                    write!(f, " {}", arg)?;
                }
                Ok(())
            }
            
            Instruction::CallFunction(func_name, args, result) => {
                write!(f, "{} = call_function '{}'", result, func_name)?;
                for arg in args {
                    write!(f, " {}", arg)?;
                }
                Ok(())
            }
            
            Instruction::Fork(result) => {
                write!(f, "{} = fork", result)
            }
            
            Instruction::Exec(pid, cmd, args) => {
                write!(f, "exec {} '{}'", pid, cmd)?;
                for arg in args {
                    write!(f, " {}", arg)?;
                }
                Ok(())
            }
            
            Instruction::Wait(pid, result) => {
                write!(f, "{} = wait {}", result, pid)
            }
            
            Instruction::Exit(status) => {
                write!(f, "exit {}", status)
            }
            
            Instruction::Kill(pid, signal) => {
                write!(f, "kill {} {}", pid, signal)
            }
            
            Instruction::Trap(signal, handler) => {
                write!(f, "trap {} .{}", signal, handler.0)
            }
            
            Instruction::CreatePipe(read_fd, write_fd) => {
                write!(f, "{} {} = create_pipe", read_fd, write_fd)
            }
            
            Instruction::DupFd(old_fd, new_fd, result) => {
                write!(f, "{} = dup_fd {} {}", result, old_fd, new_fd)
            }
            
            Instruction::CloseFd(fd) => {
                write!(f, "close_fd {}", fd)
            }
            
            Instruction::Redirect(fd, target, mode) => {
                write!(f, "redirect {} '{}' {:?}", fd, target, mode)
            }
            
            Instruction::HereDoc(content, result) => {
                write!(f, "{} = heredoc '{}'", result, content)
            }
            
            Instruction::Concat(str1, str2, result) => {
                write!(f, "{} = concat {} {}", result, str1, str2)
            }
            
            Instruction::Substr(str, start, len, result) => {
                write!(f, "{} = substr {} {} {}", result, str, start, len)
            }
            
            Instruction::Length(str, result) => {
                write!(f, "{} = length {}", result, str)
            }
            
            Instruction::PatternMatch(str, pattern, result) => {
                write!(f, "{} = pattern_match {} '{}'", result, str, pattern)
            }
            
            Instruction::GlobExpand(pattern, result) => {
                write!(f, "{} = glob_expand {}", result, pattern)
            }
            
            Instruction::Add(a, b, result) => {
                write!(f, "{} = add {} {}", result, a, b)
            }
            
            Instruction::Sub(a, b, result) => {
                write!(f, "{} = sub {} {}", result, a, b)
            }
            
            Instruction::Mul(a, b, result) => {
                write!(f, "{} = mul {} {}", result, a, b)
            }
            
            Instruction::Div(a, b, result) => {
                write!(f, "{} = div {} {}", result, a, b)
            }
            
            Instruction::Mod(a, b, result) => {
                write!(f, "{} = mod {} {}", result, a, b)
            }
            
            Instruction::Neg(a, result) => {
                write!(f, "{} = neg {}", result, a)
            }
            
            Instruction::BitAnd(a, b, result) => {
                write!(f, "{} = bit_and {} {}", result, a, b)
            }
            
            Instruction::BitOr(a, b, result) => {
                write!(f, "{} = bit_or {} {}", result, a, b)
            }
            
            Instruction::BitXor(a, b, result) => {
                write!(f, "{} = bit_xor {} {}", result, a, b)
            }
            
            Instruction::BitNot(a, result) => {
                write!(f, "{} = bit_not {}", result, a)
            }
            
            Instruction::ShiftLeft(a, b, result) => {
                write!(f, "{} = shift_left {} {}", result, a, b)
            }
            
            Instruction::ShiftRight(a, b, result) => {
                write!(f, "{} = shift_right {} {}", result, a, b)
            }
            
            Instruction::And(a, b, result) => {
                write!(f, "{} = and {} {}", result, a, b)
            }
            
            Instruction::Or(a, b, result) => {
                write!(f, "{} = or {} {}", result, a, b)
            }
            
            Instruction::Not(a, result) => {
                write!(f, "{} = not {}", result, a)
            }
            
            Instruction::Cmp(a, b, op, result) => {
                let op_str = match op {
                    CmpOp::Eq => "eq",
                    CmpOp::Ne => "ne",
                    CmpOp::Lt => "lt",
                    CmpOp::Le => "le",
                    CmpOp::Gt => "gt",
                    CmpOp::Ge => "ge",
                };
                write!(f, "{} = cmp {} {} {}", result, a, b, op_str)
            }
            
            Instruction::CreateArray(result) => {
                write!(f, "{} = create_array", result)
            }
            
            Instruction::ArraySet(array, index, value) => {
                write!(f, "array_set {} {} {}", array, index, value)
            }
            
            Instruction::ArrayGet(array, index, result) => {
                write!(f, "{} = array_get {} {}", result, array, index)
            }
            
            Instruction::ArrayLength(array, result) => {
                write!(f, "{} = array_length {}", result, array)
            }
            
            Instruction::ArrayKeys(array, result) => {
                write!(f, "{} = array_keys {}", result, array)
            }
            
            Instruction::CommandSub(cmd_result, result) => {
                write!(f, "{} = command_sub {}", result, cmd_result)
            }
            
            Instruction::ParamExpand(param, op, result) => {
                write!(f, "{} = param_expand {} ", result, param)?;
                match op {
                    ParamExpandOp::Simple => write!(f, "simple"),
                    ParamExpandOp::UseDefault(word) => write!(f, ":-'{}'", word),
                    ParamExpandOp::AssignDefault(word) => write!(f, ":=+'{}'", word),
                    ParamExpandOp::ErrorIfNull(word) => write!(f, ":?'{}'", word),
                    ParamExpandOp::UseAlternate(word) => write!(f, ":+'{}'", word),
                    ParamExpandOp::Length => write!(f, "#"),
                    ParamExpandOp::RemoveSuffix(word) => write!(f, "%'{}'", word),
                    ParamExpandOp::RemoveLargestSuffix(word) => write!(f, "%%'{}'", word),
                    ParamExpandOp::RemovePrefix(word) => write!(f, "#'{}'", word),
                    ParamExpandOp::RemoveLargestPrefix(word) => write!(f, "##'{}'", word),
                    ParamExpandOp::Substring(offset) => write!(f, ":{}", offset),
                    ParamExpandOp::SubstringWithLength(offset, length) => write!(f, ":{}:{}", offset, length),
                    ParamExpandOp::ReplaceFirst(pattern, replacement) => write!(f, "/'{}/{}'", pattern, replacement),
                    ParamExpandOp::ReplaceAll(pattern, replacement) => write!(f, "//'{}/{}'", pattern, replacement),
                    ParamExpandOp::ReplacePrefix(pattern, replacement) => write!(f, "/#'{}/{}'", pattern, replacement),
                    ParamExpandOp::ReplaceSuffix(pattern, replacement) => write!(f, "/%'{}/{}'", pattern, replacement),
                }
            }
            
            Instruction::ConstString(value, result) => {
                write!(f, "{} = const_string '{}'", result, value)
            }
            
            Instruction::ConstInt(value, result) => {
                write!(f, "{} = const_int {}", result, value)
            }
            
            Instruction::ConstBool(value, result) => {
                write!(f, "{} = const_bool {}", result, value)
            }
            
            Instruction::ConstArray(values, result) => {
                write!(f, "{} = const_array [", result)?;
                for (i, val) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "'{}'", val)?;
                }
                write!(f, "]")
            }
            
            Instruction::Phi(pairs, result) => {
                write!(f, "{} = phi", result)?;
                for (block, value) in pairs {
                    write!(f, " [.{}: {}]", block.0, value)?;
                }
                Ok(())
            }
            
            Instruction::Nop => write!(f, "nop"),
            
            Instruction::Error(msg, token) => {
                write!(f, "error '{}' at {}:{}", msg, token.line, token.column)
            }
            
            Instruction::DebugPrint(value) => {
                write!(f, "debug_print {}", value)
            }
        }
    }
}

// ============================================================================
// Basic Blocks
// ============================================================================

/// Unique identifier for basic blocks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BasicBlockId(pub usize);

impl fmt::Display for BasicBlockId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "b{}", self.0)
    }
}

/// Basic block in SSA form - a sequence of instructions with a single entry point
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BasicBlockId,
    pub label: Option<String>,
    pub instructions: Vec<Instruction>,
    pub predecessors: Vec<BasicBlockId>,
}

impl BasicBlock {
    /// Create a new basic block with the given ID
    pub fn new(id: BasicBlockId) -> Self {
        Self {
            id,
            label: None,
            instructions: Vec::new(),
            predecessors: Vec::new(),
        }
    }
    
    /// Create a new basic block with a label
    pub fn with_label(id: BasicBlockId, label: String) -> Self {
        Self {
            id,
            label: Some(label),
            instructions: Vec::new(),
            predecessors: Vec::new(),
        }
    }
    
    /// Add an instruction to the end of the block
    pub fn add_instruction(&mut self, instr: Instruction) {
        self.instructions.push(instr);
    }
    
    /// Add a predecessor block
    pub fn add_predecessor(&mut self, pred: BasicBlockId) {
        self.predecessors.push(pred);
    }
    
    /// Get the terminator instruction if this block has one
    pub fn terminator(&self) -> Option<&Instruction> {
        self.instructions.last()
    }
}

impl fmt::Display for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(label) = &self.label {
            write!(f, ".{} {}:", self.id.0, label)?;
        } else {
            write!(f, ".{}:", self.id.0)?;
        }
        
        if !self.predecessors.is_empty() {
            write!(f, "  // pred:")?;
            for pred in &self.predecessors {
                write!(f, " .{}", pred.0)?;
            }
        }
        writeln!(f)?;
        
        for instr in &self.instructions {
            writeln!(f, "    {}", instr)?;
        }
        
        Ok(())
    }
}

// ============================================================================
// Functions
// ============================================================================

/// Function in SSA form - represents a shell function or script
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub entry_block: BasicBlockId,
    pub blocks: BTreeMap<BasicBlockId, BasicBlock>,
    pub values: HashMap<ValueId, Value>,
    pub next_value_id: usize,
    pub next_block_id: usize,
}

impl Function {
    /// Create a new function with the given name and parameters
    pub fn new(name: String, params: Vec<String>) -> Self {
        let entry_block = BasicBlockId(0);
        let mut blocks = BTreeMap::new();
        blocks.insert(entry_block, BasicBlock::new(entry_block));
        
        Self {
            name,
            params,
            entry_block,
            blocks,
            values: HashMap::new(),
            next_value_id: 1, // Start from 1, 0 is reserved
            next_block_id: 1,
        }
    }
    
    /// Create a new value with the given type
    pub fn create_value(&mut self, ty: ValueType) -> ValueId {
        let id = ValueId(self.next_value_id);
        self.next_value_id += 1;
        self.values.insert(id, Value::new(id, ty));
        id
    }
    
    /// Create a new value with a name
    pub fn create_value_with_name(&mut self, ty: ValueType, name: String) -> ValueId {
        let id = ValueId(self.next_value_id);
        self.next_value_id += 1;
        self.values.insert(id, Value::with_name(id, ty, name));
        id
    }
    
    /// Create a new basic block
    pub fn create_block(&mut self) -> BasicBlockId {
        let id = BasicBlockId(self.next_block_id);
        self.next_block_id += 1;
        self.blocks.insert(id, BasicBlock::new(id));
        id
    }
    
    /// Create a new basic block with a label
    pub fn create_block_with_label(&mut self, label: String) -> BasicBlockId {
        let id = BasicBlockId(self.next_block_id);
        self.next_block_id += 1;
        self.blocks.insert(id, BasicBlock::with_label(id, label));
        id
    }
    
    /// Get a reference to a basic block
    pub fn get_block(&self, id: BasicBlockId) -> Option<&BasicBlock> {
        self.blocks.get(&id)
    }
    
    /// Get a mutable reference to a basic block
    pub fn get_block_mut(&mut self, id: BasicBlockId) -> Option<&mut BasicBlock> {
        self.blocks.get_mut(&id)
    }
    
    /// Add an instruction to a basic block
    pub fn add_instruction(&mut self, block_id: BasicBlockId, instr: Instruction) {
        if let Some(block) = self.blocks.get_mut(&block_id) {
            block.add_instruction(instr);
        }
    }
    
    /// Add a jump instruction from one block to another
    pub fn add_jump(&mut self, from_block: BasicBlockId, to_block: BasicBlockId) {
        self.add_instruction(from_block, Instruction::Jump(to_block));
        if let Some(to_block) = self.blocks.get_mut(&to_block) {
            to_block.add_predecessor(from_block);
        }
    }
    
    /// Add a conditional branch instruction
    pub fn add_branch(&mut self, from_block: BasicBlockId, cond: ValueId, 
                      true_block: BasicBlockId, false_block: BasicBlockId) {
        self.add_instruction(from_block, Instruction::Branch(cond, true_block, false_block));
        
        if let Some(true_block) = self.blocks.get_mut(&true_block) {
            true_block.add_predecessor(from_block);
        }
        
        if let Some(false_block) = self.blocks.get_mut(&false_block) {
            false_block.add_predecessor(from_block);
        }
    }
    
    /// Get all basic blocks in order
    pub fn blocks_in_order(&self) -> Vec<&BasicBlock> {
        self.blocks.values().collect()
    }
    
    /// Get all basic block IDs in order
    pub fn block_ids_in_order(&self) -> Vec<BasicBlockId> {
        self.blocks.keys().copied().collect()
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "function {}(", self.name)?;
        for (i, param) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", param)?;
        }
        writeln!(f, ") {{")?;
        
        // Blocks are already sorted in BTreeMap
        for (_, block) in &self.blocks {
            write!(f, "{}", block)?;
        }
        
        writeln!(f, "}}")
    }
}

// ============================================================================
// IR Builder
// ============================================================================

/// Builder for SSA IR with a fluent interface
pub struct IrBuilder {
    current_function: Option<Function>,
    current_block: Option<BasicBlockId>,
}

impl IrBuilder {
    /// Create a new IR builder
    pub fn new() -> Self {
        Self {
            current_function: None,
            current_block: None,
        }
    }
    
    /// Begin a new function
    pub fn begin_function(&mut self, name: String, params: Vec<String>) {
        let func = Function::new(name, params);
        self.current_function = Some(func);
        self.current_block = Some(BasicBlockId(0));
    }
    
    /// End the current function and return it
    pub fn end_function(&mut self) -> Option<Function> {
        self.current_function.take()
    }
    
    /// Get mutable reference to the current function
    pub fn get_current_function(&mut self) -> Option<&mut Function> {
        self.current_function.as_mut()
    }
    
    /// Create a new value in the current function
    pub fn create_value(&mut self, ty: ValueType) -> Option<ValueId> {
        self.current_function.as_mut().map(|f| f.create_value(ty))
    }
    
    /// Create a new value with a name in the current function
    pub fn create_value_with_name(&mut self, ty: ValueType, name: String) -> Option<ValueId> {
        self.current_function.as_mut().map(|f| f.create_value_with_name(ty, name))
    }
    
    /// Create a new basic block in the current function
    pub fn create_block(&mut self) -> Option<BasicBlockId> {
        self.current_function.as_mut().map(|f| f.create_block())
    }
    
    /// Create a new basic block with a label in the current function
    pub fn create_block_with_label(&mut self, label: String) -> Option<BasicBlockId> {
        self.current_function.as_mut().map(|f| f.create_block_with_label(label))
    }
    
    /// Set the current basic block
    pub fn set_current_block(&mut self, block_id: BasicBlockId) {
        self.current_block = Some(block_id);
    }
    
    /// Add an instruction to the current block
    pub fn add_instruction(&mut self, instr: Instruction) -> bool {
        if let (Some(func), Some(block_id)) = (&mut self.current_function, self.current_block) {
            func.add_instruction(block_id, instr);
            true
        } else {
            false
        }
    }
    
    /// Add a jump from current block to target block
    pub fn add_jump(&mut self, to_block: BasicBlockId) -> bool {
        if let (Some(func), Some(from_block)) = (&mut self.current_function, self.current_block) {
            func.add_jump(from_block, to_block);
            true
        } else {
            false
        }
    }
    
    /// Add a conditional branch from current block
    pub fn add_branch(&mut self, cond: ValueId, true_block: BasicBlockId, false_block: BasicBlockId) -> bool {
        if let (Some(func), Some(from_block)) = (&mut self.current_function, self.current_block) {
            func.add_branch(from_block, cond, true_block, false_block);
            true
        } else {
            false
        }
    }
    
    /// Get the current block ID
    pub fn current_block(&self) -> Option<BasicBlockId> {
        self.current_block
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Check if a value is a constant
pub fn is_constant_value(value_id: ValueId, func: &Function) -> bool {
    // Check if this value comes from a constant instruction
    for (_, block) in &func.blocks {
        for instr in &block.instructions {
            match instr {
                Instruction::ConstString(_, result) |
                Instruction::ConstInt(_, result) |
                Instruction::ConstBool(_, result) |
                Instruction::ConstArray(_, result) => {
                    if *result == value_id {
                        return true;
                    }
                }
                _ => {}
            }
        }
    }
    false
}

/// Get the type of a value
pub fn get_value_type(value_id: ValueId, func: &Function) -> Option<ValueType> {
    func.values.get(&value_id).map(|v| v.ty.clone())
}

/// Create a simple main function for testing
pub fn create_test_function(name: &str) -> Function {
    let mut func = Function::new(name.to_string(), Vec::new());
    
    // Create a simple echo command
    let str_val = func.create_value(ValueType::String);
    func.add_instruction(func.entry_block, Instruction::ConstString("Hello, world!".to_string(), str_val));
    
    let args = vec![str_val];
    let result = func.create_value(ValueType::ExitStatus);
    func.add_instruction(func.entry_block, Instruction::CallBuiltin("echo".to_string(), args, result));
    
    func.add_instruction(func.entry_block, Instruction::Return(result));
    
    func
}