//! Static Single Assignment (SSA) Intermediate Representation for POSIX Shell

use std::collections::HashMap;
use std::fmt;

use crate::modules::tokens::Token;

// ============================================================================
// Core Types
// ============================================================================

/// Unique identifier for values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueId(pub usize);

impl fmt::Display for ValueId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

/// Types of values in shell
#[derive(Debug, Clone, PartialEq)]
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

/// Value in SSA form
#[derive(Debug, Clone)]
pub struct Value {
    pub id: ValueId,
    pub ty: ValueType,
    pub name: Option<String>,
}

impl Value {
    pub fn new(id: ValueId, ty: ValueType) -> Self {
        Self {
            id,
            ty,
            name: None,
        }
    }
    
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

/// SSA instruction
#[derive(Debug, Clone)]
pub enum Instruction {
    // ============================================
    // Control flow
    // ============================================
    Jump(BasicBlockId),
    Branch(ValueId, BasicBlockId, BasicBlockId), // cond, true_block, false_block
    Return(ValueId),
    Break(Option<i32>),    // break [n]
    Continue(Option<i32>), // continue [n]
    
    // ============================================
    // Variable and environment operations
    // ============================================
    AllocVar(String, ValueId), // name, result
    Store(ValueId, ValueId),   // var, value
    Load(ValueId, ValueId),    // var, result
    ExportVar(ValueId),        // export variable
    UnsetVar(ValueId),         // unset variable
    ReadonlyVar(ValueId),      // readonly variable
    
    // ============================================
    // Command execution
    // ============================================
    CallBuiltin(String, Vec<ValueId>, ValueId), // name, args, result(status)
    CallExternal(String, Vec<ValueId>, ValueId), // cmd, args, result(status)
    CallFunction(String, Vec<ValueId>, ValueId), // function call
    
    // ============================================
    // Process and job control
    // ============================================
    Fork(ValueId), // result(pid)
    Exec(ValueId, String, Vec<ValueId>), // pid, cmd, args
    Wait(ValueId, ValueId), // pid, result(status)
    Exit(ValueId), // status
    Kill(ValueId, i32), // pid, signal
    Trap(ValueId, BasicBlockId), // signal, handler_block
    
    // ============================================
    // Pipeline and redirection operations
    // ============================================
    CreatePipe(ValueId, ValueId), // result(read_fd), result(write_fd)
    DupFd(ValueId, i32, ValueId), // old_fd, new_fd, result
    CloseFd(ValueId),
    Redirect(ValueId, String, RedirectMode), // fd, target, mode
    HereDoc(String, ValueId), // content, result(fd)
    
    // ============================================
    // String and pattern matching operations
    // ============================================
    Concat(ValueId, ValueId, ValueId), // str1, str2, result
    Substr(ValueId, ValueId, ValueId, ValueId), // str, start, len, result
    Length(ValueId, ValueId), // str, result
    PatternMatch(ValueId, String, ValueId), // str, pattern, result(bool)
    GlobExpand(ValueId, ValueId), // pattern, result(list)
    
    // ============================================
    // Arithmetic operations
    // ============================================
    Add(ValueId, ValueId, ValueId), // a, b, result
    Sub(ValueId, ValueId, ValueId),
    Mul(ValueId, ValueId, ValueId),
    Div(ValueId, ValueId, ValueId),
    Mod(ValueId, ValueId, ValueId),
    Neg(ValueId, ValueId), // a, result
    BitAnd(ValueId, ValueId, ValueId),
    BitOr(ValueId, ValueId, ValueId),
    BitXor(ValueId, ValueId, ValueId),
    BitNot(ValueId, ValueId),
    ShiftLeft(ValueId, ValueId, ValueId),
    ShiftRight(ValueId, ValueId, ValueId),
    
    // ============================================
    // Logical and comparison operations
    // ============================================
    And(ValueId, ValueId, ValueId), // a, b, result
    Or(ValueId, ValueId, ValueId),
    Not(ValueId, ValueId), // a, result
    Cmp(ValueId, ValueId, CmpOp, ValueId), // a, b, op, result
    
    // ============================================
    // Array and list operations
    // ============================================
    CreateArray(ValueId), // result(array)
    ArraySet(ValueId, ValueId, ValueId), // array, index, value
    ArrayGet(ValueId, ValueId, ValueId), // array, index, result
    ArrayLength(ValueId, ValueId), // array, result
    ArrayKeys(ValueId, ValueId), // array, result(list)
    
    // ============================================
    // Command substitution and parameter expansion
    // ============================================
    CommandSub(ValueId, ValueId), // command_result, result(string)
    ParamExpand(ValueId, ParamExpandOp, ValueId), // parameter, operation, result
    
    // ============================================
    // Constants and literals
    // ============================================
    ConstString(String, ValueId), // value, result
    ConstInt(i32, ValueId),       // value, result
    ConstBool(bool, ValueId),     // value, result
    ConstArray(Vec<String>, ValueId), // values, result(array)
    
    // ============================================
    // SSA-specific operations
    // ============================================
    Phi(Vec<(BasicBlockId, ValueId)>, ValueId), // incoming values, result
    
    // ============================================
    // Special operations
    // ============================================
    Nop,
    Error(String, Token),
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BasicBlockId(pub usize);

impl fmt::Display for BasicBlockId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "b{}", self.0)
    }
}

/// Basic block in SSA form
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BasicBlockId,
    pub label: Option<String>,
    pub instructions: Vec<Instruction>,
    pub predecessors: Vec<BasicBlockId>,
}

impl BasicBlock {
    pub fn new(id: BasicBlockId) -> Self {
        Self {
            id,
            label: None,
            instructions: Vec::new(),
            predecessors: Vec::new(),
        }
    }
    
    pub fn with_label(id: BasicBlockId, label: String) -> Self {
        Self {
            id,
            label: Some(label),
            instructions: Vec::new(),
            predecessors: Vec::new(),
        }
    }
    
    pub fn add_instruction(&mut self, instr: Instruction) {
        self.instructions.push(instr);
    }
    
    pub fn add_predecessor(&mut self, pred: BasicBlockId) {
        self.predecessors.push(pred);
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

/// Function in SSA form
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub entry_block: BasicBlockId,
    pub blocks: HashMap<BasicBlockId, BasicBlock>,
    pub values: HashMap<ValueId, Value>,
    pub next_value_id: usize,
    pub next_block_id: usize,
}

impl Function {
    pub fn new(name: String, params: Vec<String>) -> Self {
        let entry_block = BasicBlockId(0);
        let mut blocks = HashMap::new();
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
    
    pub fn create_value(&mut self, ty: ValueType) -> ValueId {
        let id = ValueId(self.next_value_id);
        self.next_value_id += 1;
        self.values.insert(id, Value::new(id, ty));
        id
    }
    
    pub fn create_value_with_name(&mut self, ty: ValueType, name: String) -> ValueId {
        let id = ValueId(self.next_value_id);
        self.next_value_id += 1;
        self.values.insert(id, Value::with_name(id, ty, name));
        id
    }
    
    pub fn create_block(&mut self) -> BasicBlockId {
        let id = BasicBlockId(self.next_block_id);
        self.next_block_id += 1;
        self.blocks.insert(id, BasicBlock::new(id));
        id
    }
    
    pub fn create_block_with_label(&mut self, label: String) -> BasicBlockId {
        let id = BasicBlockId(self.next_block_id);
        self.next_block_id += 1;
        self.blocks.insert(id, BasicBlock::with_label(id, label));
        id
    }
    
    pub fn get_block(&self, id: BasicBlockId) -> Option<&BasicBlock> {
        self.blocks.get(&id)
    }
    
    pub fn get_block_mut(&mut self, id: BasicBlockId) -> Option<&mut BasicBlock> {
        self.blocks.get_mut(&id)
    }
    
    pub fn add_instruction(&mut self, block_id: BasicBlockId, instr: Instruction) {
        if let Some(block) = self.blocks.get_mut(&block_id) {
            block.add_instruction(instr);
        }
    }
    
    pub fn add_jump(&mut self, from_block: BasicBlockId, to_block: BasicBlockId) {
        self.add_instruction(from_block, Instruction::Jump(to_block));
        if let Some(to_block) = self.blocks.get_mut(&to_block) {
            to_block.add_predecessor(from_block);
        }
    }
    
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
        
        // Sort blocks by ID for consistent output
        let mut block_ids: Vec<_> = self.blocks.keys().collect();
        block_ids.sort_by_key(|id| id.0);
        
        for block_id in block_ids {
            if let Some(block) = self.blocks.get(block_id) {
                write!(f, "{}", block)?;
            }
        }
        
        writeln!(f, "}}")
    }
}

// ============================================================================
// IR Builder
// ============================================================================

/// Builder for SSA IR
pub struct IrBuilder {
    current_function: Option<Function>,
    current_block: Option<BasicBlockId>,
}

impl IrBuilder {
    pub fn new() -> Self {
        Self {
            current_function: None,
            current_block: None,
        }
    }
    
    pub fn begin_function(&mut self, name: String, params: Vec<String>) {
        let func = Function::new(name, params);
        self.current_function = Some(func);
        self.current_block = Some(BasicBlockId(0));
    }
    
    pub fn end_function(&mut self) -> Option<Function> {
        self.current_function.take()
    }
    
    pub fn get_current_function(&mut self) -> Option<&mut Function> {
        self.current_function.as_mut()
    }
    
    pub fn create_value(&mut self, ty: ValueType) -> Option<ValueId> {
        self.current_function.as_mut().map(|f| f.create_value(ty))
    }
    
    pub fn create_block(&mut self) -> Option<BasicBlockId> {
        self.current_function.as_mut().map(|f| f.create_block())
    }
    
    pub fn set_current_block(&mut self, block_id: BasicBlockId) {
        self.current_block = Some(block_id);
    }
    
    pub fn add_instruction(&mut self, instr: Instruction) -> bool {
        if let (Some(func), Some(block_id)) = (&mut self.current_function, self.current_block) {
            func.add_instruction(block_id, instr);
            true
        } else {
            false
        }
    }
    
    pub fn add_jump(&mut self, to_block: BasicBlockId) -> bool {
        if let (Some(func), Some(from_block)) = (&mut self.current_function, self.current_block) {
            func.add_jump(from_block, to_block);
            true
        } else {
            false
        }
    }
    
    pub fn add_branch(&mut self, cond: ValueId, true_block: BasicBlockId, false_block: BasicBlockId) -> bool {
        if let (Some(func), Some(from_block)) = (&mut self.current_function, self.current_block) {
            func.add_branch(from_block, cond, true_block, false_block);
            true
        } else {
            false
        }
    }
}