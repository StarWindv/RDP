//! Variable system with attributes and scoping
//! Implements export, readonly, local variables, and proper inheritance

use std::collections::HashMap;

/// Variable attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarAttribute {
    Exported, // Variable is exported to child processes
    ReadOnly, // Variable cannot be modified or unset
    Integer,  // Variable should be treated as an integer
    Array,    // Variable is an array
}

/// Variable value with attributes
#[derive(Debug, Clone)]
pub struct Variable {
    pub value: String,
    pub attributes: Vec<VarAttribute>,
}

impl Variable {
    /// Create a new variable
    pub fn new(value: String) -> Self {
        Self {
            value,
            attributes: Vec::new(),
        }
    }

    /// Create a new variable with attributes
    pub fn with_attributes(value: String, attributes: Vec<VarAttribute>) -> Self {
        Self { value, attributes }
    }

    /// Check if variable has an attribute
    pub fn has_attribute(&self, attr: VarAttribute) -> bool {
        self.attributes.contains(&attr)
    }

    /// Add an attribute
    pub fn add_attribute(&mut self, attr: VarAttribute) {
        if !self.has_attribute(attr) {
            self.attributes.push(attr);
        }
    }

    /// Remove an attribute
    pub fn remove_attribute(&mut self, attr: VarAttribute) {
        self.attributes.retain(|&a| a != attr);
    }

    /// Check if variable is exported
    pub fn is_exported(&self) -> bool {
        self.has_attribute(VarAttribute::Exported)
    }

    /// Check if variable is read-only
    pub fn is_readonly(&self) -> bool {
        self.has_attribute(VarAttribute::ReadOnly)
    }

    /// Check if variable is an integer
    pub fn is_integer(&self) -> bool {
        self.has_attribute(VarAttribute::Integer)
    }

    /// Check if variable is an array
    pub fn is_array(&self) -> bool {
        self.has_attribute(VarAttribute::Array)
    }
}

/// Variable scope
#[derive(Debug, Clone, Default)]
pub struct VariableScope {
    variables: HashMap<String, Variable>,
    parent: Option<Box<VariableScope>>,
}

impl VariableScope {
    /// Create a new root scope
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parent: None,
        }
    }

    /// Create a new child scope
    pub fn new_child(parent: VariableScope) -> Self {
        Self {
            variables: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    /// Get a variable from this scope or parent scopes
    pub fn get(&self, name: &str) -> Option<&Variable> {
        if let Some(var) = self.variables.get(name) {
            Some(var)
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            None
        }
    }

    /// Get a mutable variable from this scope only
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Variable> {
        self.variables.get_mut(name)
    }

    /// Set a variable in this scope
    pub fn set(&mut self, name: String, variable: Variable) -> Result<(), String> {
        // Check if variable exists in parent scope and is read-only
        if let Some(parent_var) = self.get(&name) {
            if parent_var.is_readonly() {
                return Err(format!("{}: readonly variable", name));
            }
        }

        self.variables.insert(name, variable);
        Ok(())
    }

    /// Unset a variable from this scope
    pub fn unset(&mut self, name: &str) -> Result<(), String> {
        if let Some(var) = self.variables.get(name) {
            if var.is_readonly() {
                return Err(format!("{}: readonly variable", name));
            }
        }

        self.variables.remove(name);
        Ok(())
    }

    /// Export a variable (mark as exported)
    pub fn export(&mut self, name: &str) -> Result<(), String> {
        if let Some(var) = self.variables.get_mut(name) {
            var.add_attribute(VarAttribute::Exported);
            Ok(())
        } else if let Some(parent) = &mut self.parent {
            // Try to export from parent scope
            parent.export(name)
        } else {
            Err(format!("{}: not found", name))
        }
    }

    /// Mark a variable as read-only
    pub fn readonly(&mut self, name: &str) -> Result<(), String> {
        if let Some(var) = self.variables.get_mut(name) {
            var.add_attribute(VarAttribute::ReadOnly);
            Ok(())
        } else if let Some(parent) = &mut self.parent {
            // Try to mark as readonly in parent scope
            parent.readonly(name)
        } else {
            Err(format!("{}: not found", name))
        }
    }

    /// Mark a variable as read-only with a value
    pub fn readonly_with_value(&mut self, name: String, value: String) -> Result<(), String> {
        let mut var = Variable::new(value);
        var.add_attribute(VarAttribute::ReadOnly);
        self.set(name, var)
    }

    /// Get all exported variables for environment
    pub fn get_exported_vars(&self) -> HashMap<String, String> {
        let mut result = HashMap::new();

        // Collect from this scope
        for (name, var) in &self.variables {
            if var.is_exported() {
                result.insert(name.clone(), var.value.clone());
            }
        }

        // Collect from parent scope
        if let Some(parent) = &self.parent {
            for (name, value) in parent.get_exported_vars() {
                // Don't override variables from this scope
                if !self.variables.contains_key(&name) {
                    result.insert(name, value);
                }
            }
        }

        result
    }

    /// Get all variables (including from parent scopes)
    pub fn get_all_vars(&self) -> HashMap<String, Variable> {
        let mut result = HashMap::new();

        // Add from parent first (so child can override)
        if let Some(parent) = &self.parent {
            result.extend(parent.get_all_vars());
        }

        // Add/override with variables from this scope
        for (name, var) in &self.variables {
            result.insert(name.clone(), var.clone());
        }

        result
    }

    /// Check if a variable name is valid
    pub fn is_valid_name(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        let mut chars = name.chars();
        let first = chars.next().unwrap();

        // First character must be letter or underscore
        if !first.is_ascii_alphabetic() && first != '_' {
            return false;
        }

        // Remaining characters must be alphanumeric or underscore
        chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    }

    /// Create a local variable (in current scope only)
    pub fn local(&mut self, name: String, value: Option<String>) -> Result<(), String> {
        let value = value.unwrap_or_else(String::new);
        let var = Variable::new(value);
        self.variables.insert(name, var);
        Ok(())
    }
}

/// Variable system manager
#[derive(Debug, Clone)]
pub struct VariableSystem {
    global_scope: VariableScope,
    current_scope: VariableScope,
}

impl VariableSystem {
    /// Create a new variable system
    pub fn new() -> Self {
        let global_scope = VariableScope::new();
        let current_scope = global_scope.clone();

        Self {
            global_scope,
            current_scope,
        }
    }

    /// Enter a new scope (e.g., for function execution)
    pub fn enter_scope(&mut self) {
        let new_scope = VariableScope::new_child(std::mem::take(&mut self.current_scope));
        self.current_scope = new_scope;
    }

    /// Exit current scope
    pub fn exit_scope(&mut self) -> Result<(), String> {
        if let Some(parent) = self.current_scope.parent.take() {
            self.current_scope = *parent;
            Ok(())
        } else {
            Err("Cannot exit root scope".to_string())
        }
    }

    /// Get a variable
    pub fn get(&self, name: &str) -> Option<&Variable> {
        self.current_scope.get(name)
    }

    /// Set a variable
    pub fn set(&mut self, name: String, value: String) -> Result<(), String> {
        if !VariableScope::is_valid_name(&name) {
            return Err(format!("{}: invalid variable name", name));
        }

        let var = Variable::new(value);
        self.current_scope.set(name, var)
    }

    /// Unset a variable
    pub fn unset(&mut self, name: &str) -> Result<(), String> {
        self.current_scope.unset(name)
    }

    /// Export a variable
    pub fn export(&mut self, name: &str) -> Result<(), String> {
        self.current_scope.export(name)
    }

    /// Mark a variable as read-only
    pub fn readonly(&mut self, name: &str) -> Result<(), String> {
        self.current_scope.readonly(name)
    }

    /// Mark a variable as read-only with a value
    pub fn readonly_with_value(&mut self, name: String, value: String) -> Result<(), String> {
        if !VariableScope::is_valid_name(&name) {
            return Err(format!("{}: invalid variable name", name));
        }

        self.current_scope.readonly_with_value(name, value)
    }

    /// Create a local variable
    pub fn local(&mut self, name: String, value: Option<String>) -> Result<(), String> {
        if !VariableScope::is_valid_name(&name) {
            return Err(format!("{}: invalid variable name", name));
        }

        self.current_scope.local(name, value)
    }

    /// Get all exported variables for environment
    pub fn get_exported_vars(&self) -> HashMap<String, String> {
        self.current_scope.get_exported_vars()
    }

    /// Get all variables
    pub fn get_all_vars(&self) -> HashMap<String, Variable> {
        self.current_scope.get_all_vars()
    }

    /// Get variable value as string
    pub fn get_value(&self, name: &str) -> Option<String> {
        self.get(name).map(|var| var.value.clone())
    }

    /// Check if variable is exported
    pub fn is_exported(&self, name: &str) -> bool {
        self.get(name).map(|var| var.is_exported()).unwrap_or(false)
    }

    /// Check if variable is read-only
    pub fn is_readonly(&self, name: &str) -> bool {
        self.get(name).map(|var| var.is_readonly()).unwrap_or(false)
    }
}

/// Global variable system instance
lazy_static::lazy_static! {
    static ref VARIABLE_SYSTEM: std::sync::Mutex<VariableSystem> =
        std::sync::Mutex::new(VariableSystem::new());
}

/// Get global variable system
pub fn get_variable_system() -> std::sync::MutexGuard<'static, VariableSystem> {
    VARIABLE_SYSTEM.lock().unwrap()
}

/// Initialize variable system with environment variables
pub fn init_variable_system() {
    let mut vs = VARIABLE_SYSTEM.lock().unwrap();

    // Copy environment variables and mark them as exported
    for (key, value) in std::env::vars() {
        if VariableScope::is_valid_name(&key) {
            let mut var = Variable::new(value);
            var.add_attribute(VarAttribute::Exported);
            vs.current_scope.set(key, var).unwrap();
        }
    }
}
