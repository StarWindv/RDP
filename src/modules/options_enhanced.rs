//! Enhanced shell options system
//! Implements full POSIX shell options with inheritance, persistence, and better error handling

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::modules::options::ShellOption;

impl EnhancedShellOptions {
    /// Create new root shell options with defaults
    pub fn new() -> Self {
        let mut options = HashMap::new();
        
        // Set default values
        for opt in &[
            ShellOption::ErrExit,
            ShellOption::NoUnset,
            ShellOption::XTrace,
            ShellOption::NoExec,
            ShellOption::Verbose,
            ShellOption::AllExport,
            ShellOption::IgnoreEof,
            ShellOption::Interactive,
            ShellOption::Monitor,
            ShellOption::NoClobber,
            ShellOption::NoGlob,
            ShellOption::NoLog,
            ShellOption::Notify,
            ShellOption::Physical,
            ShellOption::Posix,
            ShellOption::Vi,
        ] {
            options.insert(*opt, opt.default_value());
        }
        
        Self {
            options,
            positional_params: Vec::new(),
            parent: None,
            changed_options: HashMap::new(),
        }
    }
    
    /// Create a child options instance (for subshells, functions)
    pub fn create_child(&self) -> Self {
        let parent = Arc::new(Mutex::new(self.clone()));
        Self {
            options: self.options.clone(),
            positional_params: self.positional_params.clone(),
            parent: Some(parent),
            changed_options: HashMap::new(),
        }
    }
    
    /// Check if an option is enabled
    pub fn is_enabled(&self, option: ShellOption) -> bool {
        *self.options.get(&option).unwrap_or(&false)
    }
    
    /// Check if option is inherited from parent
    pub fn is_inherited(&self, option: ShellOption) -> bool {
        !self.changed_options.contains_key(&option)
    }
    
    /// Set an option
    pub fn set_option(&mut self, option: ShellOption, enabled: bool) {
        self.options.insert(option, enabled);
        self.changed_options.insert(option, enabled);
    }
    
    /// Reset an option to parent value
    pub fn reset_option(&mut self, option: ShellOption) {
        if let Some(parent) = &self.parent {
            let parent_guard = parent.lock().unwrap();
            if let Some(&parent_value) = parent_guard.options.get(&option) {
                self.options.insert(option, parent_value);
            }
        }
        self.changed_options.remove(&option);
    }
    
    /// Reset all options to parent values
    pub fn reset_all_options(&mut self) {
        if let Some(parent) = &self.parent {
            let parent_guard = parent.lock().unwrap();
            self.options = parent_guard.options.clone();
        }
        self.changed_options.clear();
    }
    
    /// Set positional parameters
    pub fn set_positional_params(&mut self, params: Vec<String>) {
        self.positional_params = params;
    }
    
    /// Get positional parameters
    pub fn get_positional_params(&self) -> &[String] {
        &self.positional_params
    }
    
    /// Shift positional parameters
    pub fn shift_positional_params(&mut self, n: usize) -> Vec<String> {
        if n == 0 || self.positional_params.is_empty() {
            return Vec::new();
        }
        
        let shift_count = std::cmp::min(n, self.positional_params.len());
        let shifted: Vec<String> = self.positional_params.drain(0..shift_count).collect();
        
        // If all params were shifted, add an empty string
        if self.positional_params.is_empty() {
            self.positional_params.push(String::new());
        }
        
        shifted
    }
    
    /// Get all options as formatted strings
    pub fn format_all_options(&self) -> Vec<String> {
        let mut result = Vec::new();
        
        // Format current options
        for (opt, &enabled) in &self.options {
            let status = if enabled { "on" } else { "off" };
            let inherited = if self.is_inherited(*opt) { " (inherited)" } else { "" };
            result.push(format!("{}: {}{}", opt.long_name(), status, inherited));
        }
        
        result.sort();
        result
    }
    
    /// Get changed options as formatted strings
    pub fn format_changed_options(&self) -> Vec<String> {
        let mut result = Vec::new();
        
        for (opt, &enabled) in &self.changed_options {
            let status = if enabled { "on" } else { "off" };
            result.push(format!("{}: {}", opt.long_name(), status));
        }
        
        result.sort();
        result
    }
    
    /// Parse and apply options from command line arguments with better error handling
    pub fn apply_options_with_context(&mut self, args: &[String], context: &str) -> Result<usize, String> {
        let mut i = 0;
        let mut option_errors = Vec::new();
        
        while i < args.len() {
            let arg = &args[i];
            
            if arg == "--" {
                // End of options
                i += 1;
                break;
            }
            
            if arg.starts_with('-') && !arg.starts_with("--") && arg != "-" && arg != "-o" {
                // Handle short options: -e, -u, -x, etc.
                for ch in arg[1..].chars() {
                    match ShellOption::from_short(ch) {
                        Some(opt) => {
                            self.set_option(opt, true);
                        }
                        None => {
                            option_errors.push(format!("-{}: invalid option", ch));
                        }
                    }
                }
            } else if arg.starts_with('+') && arg != "+o" {
                // Clear options: +e, +u, +x, etc.
                for ch in arg[1..].chars() {
                    match ShellOption::from_short(ch) {
                        Some(opt) => {
                            self.set_option(opt, false);
                        }
                        None => {
                            option_errors.push(format!("+{}: invalid option", ch));
                        }
                    }
                }
            } else if arg == "-o" {
                // Long option: -o option_name
                if i + 1 >= args.len() {
                    return Err(format!("{}: -o requires an argument", context));
                }
                
                let opt_name = &args[i + 1];
                match ShellOption::from_long(opt_name) {
                    Some(opt) => {
                        self.set_option(opt, true);
                    }
                    None => {
                        option_errors.push(format!("{}: invalid option", opt_name));
                    }
                }
                
                i += 1; // Skip the option name
            } else if arg == "+o" {
                // Clear long option: +o option_name
                if i + 1 >= args.len() {
                    return Err(format!("{}: +o requires an argument", context));
                }
                
                let opt_name = &args[i + 1];
                match ShellOption::from_long(opt_name) {
                    Some(opt) => {
                        self.set_option(opt, false);
                    }
                    None => {
                        option_errors.push(format!("{}: invalid option", opt_name));
                    }
                }
                
                i += 1; // Skip the option name
            } else if arg == "-" {
                // Unset positional parameters
                self.positional_params.clear();
            } else {
                // Not an option, start of positional parameters
                break;
            }
            
            i += 1;
        }
        
        if !option_errors.is_empty() {
            return Err(format!("{}: {}", context, option_errors.join(", ")));
        }
        
        Ok(i)
    }
    
    /// Apply positional parameters after options
    pub fn apply_positional_params(&mut self, args: &[String], start_index: usize) {
        self.positional_params = args[start_index..].to_vec();
    }
    
    /// Export options to environment variables (for child processes)
    pub fn export_to_env(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();
        
        // Export relevant options as environment variables
        if self.is_enabled(ShellOption::ErrExit) {
            env.insert("SHELLOPTS".to_string(), "errexit".to_string());
        }
        if self.is_enabled(ShellOption::NoUnset) {
            env.insert("SHELLOPTS".to_string(), "nounset".to_string());
        }
        if self.is_enabled(ShellOption::XTrace) {
            env.insert("SHELLOPTS".to_string(), "xtrace".to_string());
        }
        
        // Export positional parameters as $1, $2, etc.
        for (i, param) in self.positional_params.iter().enumerate() {
            env.insert(format!("{}", i + 1), param.clone());
        }
        
        env
    }
    
    /// Import options from environment variables
    pub fn import_from_env(&mut self, env: &HashMap<String, String>) {
        if let Some(shellopts) = env.get("SHELLOPTS") {
            for opt_str in shellopts.split(':') {
                if let Some(opt) = ShellOption::from_long(opt_str) {
                    self.set_option(opt, true);
                }
            }
        }
    }
    
    /// Validate option combinations
    pub fn validate_options(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // Check for conflicting options
        if self.is_enabled(ShellOption::NoExec) && self.is_enabled(ShellOption::XTrace) {
            errors.push("noexec and xtrace cannot be used together".to_string());
        }
        
        if self.is_enabled(ShellOption::Interactive) && self.is_enabled(ShellOption::NoExec) {
            errors.push("interactive shells cannot use noexec".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Get option description for help
    pub fn get_option_help(&self, option_name: &str) -> Option<String> {
        if let Some(opt) = ShellOption::from_long(option_name) {
            Some(format!("{} (-{}): {}", 
                opt.long_name(), 
                opt.short_name().unwrap_or(' '),
                opt.description()))
        } else {
            None
        }
    }
}

/// Global enhanced shell options instance
lazy_static::lazy_static! {
    static ref ENHANCED_SHELL_OPTIONS: Arc<Mutex<EnhancedShellOptions>> = 
        Arc::new(Mutex::new(EnhancedShellOptions::new()));
}

/// Get global enhanced shell options
pub fn get_enhanced_options() -> Arc<Mutex<EnhancedShellOptions>> {
    ENHANCED_SHELL_OPTIONS.clone()
}

/// Initialize enhanced shell options
pub fn init_enhanced_options() {
    // Already initialized by lazy_static
}

/// Built-in set command implementation
pub fn set_builtin(args: &[String]) -> i32 {
    let options = get_enhanced_options();
    let mut opts_guard = options.lock().unwrap();
    
    if args.is_empty() {
        // Display all variables and functions
        // For now, just display options
        println!("Current shell options:");
        for line in opts_guard.format_all_options() {
            println!("  {}", line);
        }
        return 0;
    }
    
    match opts_guard.apply_options_with_context(args, "set") {
        Ok(opt_end) => {
            // Apply positional parameters if any
            if opt_end < args.len() {
                opts_guard.apply_positional_params(args, opt_end);
            }
            
            // Validate options
            match opts_guard.validate_options() {
                Ok(_) => 0,
                Err(errors) => {
                    for error in errors {
                        eprintln!("set: {}", error);
                    }
                    1
                }
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            1
        }
    }
}

/// Built-in shopt command (shell options)
pub fn shopt_builtin(args: &[String]) -> i32 {
    let options = get_enhanced_options();
    let opts_guard = options.lock().unwrap();
    
    if args.is_empty() {
        // List all options
        for line in opts_guard.format_all_options() {
            println!("{}", line);
        }
        return 0;
    }
    
    match args[0].as_str() {
        "-s" | "--set" => {
            // Set options
            if args.len() < 2 {
                eprintln!("shopt: option name required");
                return 1;
            }
            
            let mut opts_guard = options.lock().unwrap();
            for opt_name in &args[1..] {
                if let Some(opt) = ShellOption::from_long(opt_name) {
                    opts_guard.set_option(opt, true);
                } else {
                    eprintln!("shopt: invalid option: {}", opt_name);
                    return 1;
                }
            }
            0
        }
        "-u" | "--unset" => {
            // Unset options
            if args.len() < 2 {
                eprintln!("shopt: option name required");
                return 1;
            }
            
            let mut opts_guard = options.lock().unwrap();
            for opt_name in &args[1..] {
                if let Some(opt) = ShellOption::from_long(opt_name) {
                    opts_guard.set_option(opt, false);
                } else {
                    eprintln!("shopt: invalid option: {}", opt_name);
                    return 1;
                }
            }
            0
        }
        "-q" | "--quiet" => {
            // Quiet mode - just return status
            if args.len() < 2 {
                eprintln!("shopt: option name required");
                return 1;
            }
            
            for opt_name in &args[1..] {
                if let Some(opt) = ShellOption::from_long(opt_name) {
                    if !opts_guard.is_enabled(opt) {
                        return 1;
                    }
                } else {
                    return 1;
                }
            }
            0
        }
        opt_name => {
            // Check if option is set
            if let Some(opt) = ShellOption::from_long(opt_name) {
                if opts_guard.is_enabled(opt) {
                    println!("{} on", opt_name);
                    0
                } else {
                    println!("{} off", opt_name);
                    1
                }
            } else {
                eprintln!("shopt: invalid option: {}", opt_name);
                1
            }
        }
    }
}

/// Check if errexit option is enabled (enhanced version)
pub fn errexit_enabled_enhanced() -> bool {
    let options = get_enhanced_options();
    let opts_guard = options.lock().unwrap();
    opts_guard.is_enabled(ShellOption::ErrExit)
}

/// Check if nounset option is enabled (enhanced version)
pub fn nounset_enabled_enhanced() -> bool {
    let options = get_enhanced_options();
    let opts_guard = options.lock().unwrap();
    opts_guard.is_enabled(ShellOption::NoUnset)
}

/// Check if xtrace option is enabled (enhanced version)
pub fn xtrace_enabled_enhanced() -> bool {
    let options = get_enhanced_options();
    let opts_guard = options.lock().unwrap();
    opts_guard.is_enabled(ShellOption::XTrace)
}

/// Check if noexec option is enabled (enhanced version)
pub fn noexec_enabled_enhanced() -> bool {
    let options = get_enhanced_options();
    let opts_guard = options.lock().unwrap();
    opts_guard.is_enabled(ShellOption::NoExec)
}

/// Check if verbose option is enabled (enhanced version)
pub fn verbose_enabled_enhanced() -> bool {
    let options = get_enhanced_options();
    let opts_guard = options.lock().unwrap();
    opts_guard.is_enabled(ShellOption::Verbose)
}

/// Get current positional parameters (enhanced version)
pub fn get_positional_params_enhanced() -> Vec<String> {
    let options = get_enhanced_options();
    let opts_guard = options.lock().unwrap();
    opts_guard.get_positional_params().to_vec()
}

/// Set positional parameters (enhanced version)
pub fn set_positional_params_enhanced(params: Vec<String>) {
    let options = get_enhanced_options();
    let mut opts_guard = options.lock().unwrap();
    opts_guard.set_positional_params(params);
}

/// Create child options for subshell or function
pub fn create_child_options() -> EnhancedShellOptions {
    let options = get_enhanced_options();
    let opts_guard = options.lock().unwrap();
    opts_guard.create_child()
}