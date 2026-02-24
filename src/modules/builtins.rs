//! Built-in shell commands

use std::io;
use std::io::Write;
use std::fs;
use std::process::Command;
use std::collections::HashSet;

use crate::env::ShellEnv;

/// Built-in command handler
pub struct Builtins;

impl Builtins {
    /// Execute a built-in command
    pub fn execute(&self, name: &str, args: &[String], env: &mut ShellEnv) -> i32 {
        match name {
            // POSIX special builtins
            "." => self.dot(args, env),
            ":" => self.colon(args),
            "break" => self.break_cmd(args, env),
            "continue" => self.continue_cmd(args, env),
            "eval" => self.eval(args, env),
            "exec" => self.exec(args, env),
            "exit" => self.exit(args),
            "export" => self.export(args, env),
            "readonly" => self.readonly(args, env),
            "set" => self.set(args, env),
            "shift" => self.shift(args, env),
            "times" => self.times(env),
            "trap" => self.trap(args, env),
            "unset" => self.unset(args, env),
            
            // Other builtins
            "cd" => self.cd(args, env),
            "pwd" => self.pwd(env),
            "echo" => self.echo(args, env),
            "true" => 0,
            "false" => 1,
            "help" => self.help(),
            
            // POSIX standard utility builtins (P1 priority)
            "alias" => self.alias(args, env),
            "command" => self.command(args, env),
            "getopts" => self.getopts(args, env),
            "hash" => self.hash(args, env),
            "kill" => self.kill(args, env),
            "read" => self.read(args, env),
            "type" => self.type_cmd(args, env),
            "umask" => self.umask(args, env),
            "ulimit" => self.ulimit(args, env),
            "printf" => self.printf(args, env),
            
            _ => {
                eprintln!("{}: command not found", name);
                127
            }
        }
    }
    
    /// Change directory
    fn cd(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        let target = if args.is_empty() {
            // Go to home directory
            env.get_var("HOME").cloned().unwrap_or_else(|| "/".to_string())
        } else if args[0] == "-" {
            // Go to previous directory (OLDPWD)
            env.get_var("OLDPWD").cloned().unwrap_or_else(|| env.current_dir.clone())
        } else {
            args[0].clone()
        };
        
        // Save current directory as OLDPWD
        env.set_var("OLDPWD".to_string(), env.current_dir.clone());
        
        // Change directory
        match std::env::set_current_dir(&target) {
            Ok(_) => {
                // Update current directory in environment
                if let Ok(cwd) = std::env::current_dir() {
                    env.current_dir = cwd.to_string_lossy().to_string();
                }
                0
            }
            Err(e) => {
                eprintln!("cd: {}: {}", target, e);
                1
            }
        }
    }
    
    /// Print working directory
    fn pwd(&self, env: &ShellEnv) -> i32 {
        println!("{}", env.current_dir);
        0
    }
    
    /// Echo arguments
    fn echo(&self, args: &[String], _env: &ShellEnv) -> i32 {
        let mut first = true;
        for arg in args {
            if !first {
                print!(" ");
            }
            // Arguments are already expanded by the executor
            print!("{}", arg);
            first = false;
        }
        println!();
        0
    }
    
    /// Exit shell
    fn exit(&self, args: &[String]) -> i32 {
        let exit_code = if args.is_empty() {
            0
        } else {
            args[0].parse().unwrap_or(1)
        };
        
        std::process::exit(exit_code);
    }
    
    /// Show help
    fn help(&self) -> i32 {
        println!("rs-dash-pro - A POSIX-compatible shell");
        println!();
        println!("Built-in commands:");
        println!("  cd [dir]      Change directory");
        println!("  pwd           Print working directory");
        println!("  echo [args]   Print arguments");
        println!("  exit [n]      Exit shell with status n");
        println!("  true          Return success");
        println!("  false         Return failure");
        println!("  help          Show this help");
        println!();
        println!("External commands are also supported via PATH.");
        0
    }
    
    /// Check if a command is a built-in
    pub fn is_builtin(&self, name: &str) -> bool {
        matches!(
            name,
            // POSIX special builtins
            "." | ":" | "break" | "continue" | "eval" | "exec" | "exit" | 
            "export" | "readonly" | "set" | "shift" | "times" | "trap" | "unset" |
            
            // Other builtins
            "cd" | "pwd" | "echo" | "true" | "false" | "help" |
            
            // POSIX standard utility builtins
            "alias" | "command" | "getopts" | "hash" | "kill" | "read" |
            "type" | "umask" | "ulimit" | "printf"
        )
    }
    
    // ============================================
    // POSIX Special Builtins Implementation
    // ============================================
    
    /// . (dot) - source a file
    fn dot(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            eprintln!(".: filename argument required");
            return 1;
        }
        
        let filename = &args[0];
        match fs::read_to_string(filename) {
            Ok(content) => {
                println!("DEBUG: Sourcing file: {}", filename);
                
                // Split content into lines and execute each line
                let mut last_status = 0;
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    
                    println!("DEBUG: Executing sourced line: {}", line);
                    
                    // TODO: Actually parse and execute the command
                    // For now, just check if it's an export command
                    if line.starts_with("export ") {
                        let export_line = &line[7..]; // Remove "export "
                        let parts: Vec<&str> = export_line.splitn(2, '=').collect();
                        if parts.len() == 2 {
                            let var_name = parts[0].trim();
                            let var_value = parts[1].trim().trim_matches('"');
                            println!("DEBUG: Setting variable {} = {}", var_name, var_value);
                            env.set_var(var_name.to_string(), var_value.to_string());
                            // Mark as exported
                            env.set_var(format!("__exported_{}", var_name), "1".to_string());
                        }
                    } else if line.starts_with("echo ") {
                        let echo_line = &line[5..]; // Remove "echo "
                        // Expand variables in the echo line
                        let expanded = env.expand_variables(echo_line.trim_matches('"'));
                        println!("{}", expanded);
                    }
                }
                
                last_status
            }
            Err(e) => {
                eprintln!(".: {}: {}", filename, e);
                1
            }
        }
    }
    
    /// : (colon) - null command, always succeeds
    fn colon(&self, _args: &[String]) -> i32 {
        0
    }
    
    /// break - exit from a loop
    fn break_cmd(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        let n = if args.is_empty() {
            1
        } else {
            args[0].parse().unwrap_or(1)
        };
        
        // Store break count in environment
        env.set_var("BREAK_LEVEL".to_string(), n.to_string());
        0
    }
    
    /// continue - continue loop iteration
    fn continue_cmd(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        let n = if args.is_empty() {
            1
        } else {
            args[0].parse().unwrap_or(1)
        };
        
        // Store continue count in environment
        env.set_var("CONTINUE_LEVEL".to_string(), n.to_string());
        0
    }
    
    /// eval - evaluate arguments as shell commands
    fn eval(&self, args: &[String], _env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            return 0;
        }
        
        // Join all arguments with spaces
        let cmd = args.join(" ");
        println!("eval: executing '{}'", cmd);
        
        // TODO: Actually parse and execute the command
        // For now, just return success
        0
    }
    
    /// exec - replace shell with command
    fn exec(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            // Just return success if no arguments
            return 0;
        }
        
        let command = &args[0];
        let command_args = &args[1..];
        
        println!("exec: executing '{}' with args {:?}", command, command_args);
        
        // TODO: Actually replace shell process
        // For now, just execute as external command
        match Command::new(command)
            .args(command_args)
            .current_dir(&env.current_dir)
            .status() {
            Ok(status) => {
                if let Some(code) = status.code() {
                    code
                } else {
                    128
                }
            }
            Err(e) => {
                eprintln!("exec: {}: {}", command, e);
                127
            }
        }
    }
    
    /// export - set export attribute for variables
    fn export(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        println!("DEBUG BUILTIN export: args = {:?}", args);
        if args.is_empty() {
            // Print all exported variables
            for (key, value) in &env.vars {
                println!("export {}='{}'", key, value);
            }
            return 0;
        }
        
        for arg in args {
            println!("DEBUG BUILTIN export: processing arg = {}", arg);
            if arg.contains('=') {
                // VAR=value format
                let parts: Vec<&str> = arg.splitn(2, '=').collect();
                let var = parts[0];
                let val = if parts.len() > 1 { parts[1] } else { "" };
                println!("DEBUG BUILTIN export: setting var {} = {}", var, val);
                env.set_var(var.to_string(), val.to_string());
                // Mark as exported (in real shell, we'd track this separately)
                env.set_var(format!("__exported_{}", var), "1".to_string());
            } else {
                // Just variable name, mark as exported
                println!("DEBUG BUILTIN export: marking {} as exported", arg);
                env.set_var(format!("__exported_{}", arg), "1".to_string());
            }
        }
        
        0
    }
    
    /// readonly - make variables read-only
    fn readonly(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            // Print all read-only variables
            for (key, value) in &env.vars {
                if key.starts_with("__readonly_") {
                    let var_name = &key[11..]; // Remove "__readonly_" prefix
                    println!("readonly {}='{}'", var_name, value);
                }
            }
            return 0;
        }
        
        for arg in args {
            if arg.contains('=') {
                // VAR=value format
                let parts: Vec<&str> = arg.splitn(2, '=').collect();
                let var = parts[0];
                let val = if parts.len() > 1 { parts[1] } else { "" };
                env.set_var(var.to_string(), val.to_string());
                // Mark as read-only
                env.set_var(format!("__readonly_{}", var), "1".to_string());
            } else {
                // Just variable name, mark as read-only
                env.set_var(format!("__readonly_{}", arg), "1".to_string());
            }
        }
        
        0
    }
    
    /// set - set shell options
    fn set(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            // Print all variables
            for (key, value) in &env.vars {
                println!("{}='{}'", key, value);
            }
            return 0;
        }
        
        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            
            if arg == "--" {
                // End of options
                i += 1;
                break;
            }
            
            if arg.starts_with('-') && !arg.starts_with("--") {
                // Handle options
                for ch in arg[1..].chars() {
                    match ch {
                        'e' => env.set_var("__option_errexit".to_string(), "1".to_string()),
                        'u' => env.set_var("__option_nounset".to_string(), "1".to_string()),
                        'x' => env.set_var("__option_xtrace".to_string(), "1".to_string()),
                        _ => {
                            eprintln!("set: invalid option -{}", ch);
                            return 1;
                        }
                    }
                }
            } else if arg.starts_with('+') {
                // Clear options
                for ch in arg[1..].chars() {
                    match ch {
                        'e' => env.unset_var("__option_errexit"),
                        'u' => env.unset_var("__option_nounset"),
                        'x' => env.unset_var("__option_xtrace"),
                        _ => {
                            eprintln!("set: invalid option +{}", ch);
                            return 1;
                        }
                    }
                }
                // i is incremented at the end of the loop
            } else {
                // Set positional parameters
                let mut pos_args = Vec::new();
                for j in i..args.len() {
                    pos_args.push(args[j].clone());
                }
                env.set_var("__positional_args".to_string(), pos_args.join(" "));
                break;
            }
            
            i += 1;
        }
        
        0
    }
    
    /// shift - shift positional parameters
    fn shift(&self, args: &[String], _env: &mut ShellEnv) -> i32 {
        let n = if args.is_empty() {
            1
        } else {
            args[0].parse().unwrap_or(1)
        };
        
        // TODO: Actually shift positional parameters
        // For now, just log the operation
        println!("shift: shifting {} positional parameters", n);
        0
    }
    
    /// times - print process times
    fn times(&self, _env: &mut ShellEnv) -> i32 {
        // TODO: Get actual process times
        // For now, print dummy values
        println!("0m0.000s 0m0.000s");
        println!("0m0.000s 0m0.000s");
        0
    }
    
    /// trap - handle signals
    fn trap(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            // Print current traps
            println!("trap: no traps set");
            return 0;
        }
        
        // Parse trap command
        // Format: trap [action] signal...
        let action = if args[0] == "-" {
            // Reset to default
            "".to_string()
        } else if args[0] == "--" {
            // Reset all signals
            for sig in &["INT", "TERM", "EXIT", "ERR"] {
                env.set_var(format!("__trap_{}", sig), "".to_string());
            }
            return 0;
        } else {
            args[0].clone()
        };
        
        let signals = if args[0] == "-" || args[0] == "--" {
            &args[1..]
        } else {
            &args[1..]
        };
        
        if signals.is_empty() {
            // No signals specified, use default
            env.set_var("__trap_EXIT".to_string(), action);
        } else {
            for sig in signals {
                env.set_var(format!("__trap_{}", sig), action.clone());
            }
        }
        
        0
    }
    
    /// unset - unset variables
    fn unset(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            eprintln!("unset: variable name required");
            return 1;
        }
        
        for arg in args {
            env.unset_var(arg);
            // Also unset special attributes
            env.unset_var(&format!("__exported_{}", arg));
            env.unset_var(&format!("__readonly_{}", arg));
        }
        
        0
    }
    
    // ============================================
    // POSIX Standard Utility Builtins (P1 priority)
    // ============================================
    
    /// alias - define or display aliases
    fn alias(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            // Print all aliases
            for (key, value) in &env.vars {
                if key.starts_with("__alias_") {
                    let alias_name = &key[8..]; // Remove "__alias_" prefix
                    println!("alias {}='{}'", alias_name, value);
                }
            }
            return 0;
        }
        
        for arg in args {
            if arg.contains('=') {
                let parts: Vec<&str> = arg.splitn(2, '=').collect();
                let alias_name = parts[0];
                let alias_value = if parts.len() > 1 { parts[1] } else { "" };
                env.set_var(format!("__alias_{}", alias_name), alias_value.to_string());
            } else {
                // Just print specific alias
                if let Some(value) = env.get_var(&format!("__alias_{}", arg)) {
                    println!("alias {}='{}'", arg, value);
                }
            }
        }
        
        0
    }
    
    /// command - execute a simple command
    fn command(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            eprintln!("command: command name required");
            return 1;
        }
        
        // Skip aliases and functions, execute directly
        let command = &args[0];
        let command_args = &args[1..];
        
        // Check if it's a builtin
        if self.is_builtin(command) && command != "command" {
            // Execute builtin directly
            return self.execute(command, command_args, env);
        }
        
        // Otherwise execute as external command
        match env.find_in_path(command) {
            Some(full_path) => {
                match Command::new(&full_path)
                    .args(command_args)
                    .current_dir(&env.current_dir)
                    .status() {
                    Ok(status) => {
                        if let Some(code) = status.code() {
                            code
                        } else {
                            128
                        }
                    }
                    Err(e) => {
                        eprintln!("command: {}: {}", command, e);
                        127
                    }
                }
            }
            None => {
                eprintln!("command: {}: command not found", command);
                127
            }
        }
    }
    
    /// getopts - parse command options
    fn getopts(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        if args.len() < 2 {
            eprintln!("getopts: usage: getopts optstring name [args]");
            return 1;
        }
        
        let optstring = &args[0];
        let varname = &args[1];
        let remaining_args = if args.len() > 2 { &args[2..] } else { &[] as &[String] };
        
        println!("getopts: parsing options '{}' for variable '{}' with args {:?}", 
                optstring, varname, remaining_args);
        
        // TODO: Implement actual option parsing
        // For now, just set OPTIND
        env.set_var("OPTIND".to_string(), "1".to_string());
        
        0
    }
    
    /// hash - remember command locations
    fn hash(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            // Print hash table
            println!("hash: hash table empty");
            return 0;
        }
        
        for arg in args {
            if arg == "-r" {
                // Clear hash table
                println!("hash: clearing hash table");
            } else if arg == "-p" {
                // Add specific path
                // Format: -p path name
                println!("hash: adding path for command");
            } else {
                // Look up command
                match env.find_in_path(arg) {
                    Some(path) => {
                        println!("{} {}", arg, path);
                    }
                    None => {
                        eprintln!("hash: {}: command not found", arg);
                        return 1;
                    }
                }
            }
        }
        
        0
    }
    
    /// kill - send signal to process
    fn kill(&self, args: &[String], _env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            eprintln!("kill: usage: kill [-s sigspec] pid...");
            return 1;
        }
        
        println!("kill: sending signal to processes: {:?}", args);
        // TODO: Implement actual signal sending
        0
    }
    
    /// read - read input
    fn read(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        let mut var_name = "REPLY".to_string();
        let mut prompt = None;
        
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-p" => {
                    i += 1;
                    if i < args.len() {
                        prompt = Some(args[i].clone());
                    }
                }
                "-r" => {
                    // Raw mode - don't interpret backslashes
                    // TODO: Implement
                }
                arg if !arg.starts_with('-') => {
                    var_name = arg.to_string();
                }
                _ => {}
            }
            i += 1;
        }
        
        if let Some(p) = prompt {
            print!("{}", p);
            io::stdout().flush().unwrap();
        }
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim_end_matches('\n').trim_end_matches('\r').to_string();
                env.set_var(var_name, input);
                0
            }
            Err(e) => {
                eprintln!("read: {}", e);
                1
            }
        }
    }
    
    /// type - describe command type
    fn type_cmd(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            eprintln!("type: command name required");
            return 1;
        }
        
        for arg in args {
            if self.is_builtin(arg) {
                println!("{} is a shell builtin", arg);
            } else if env.vars.contains_key(&format!("__alias_{}", arg)) {
                println!("{} is an alias", arg);
            } else if env.vars.contains_key(&format!("__function_{}", arg)) {
                println!("{} is a function", arg);
            } else if env.find_in_path(arg).is_some() {
                println!("{} is {}", arg, env.find_in_path(arg).unwrap());
            } else {
                println!("{}: not found", arg);
                return 1;
            }
        }
        
        0
    }
    
    /// umask - set file creation mask
    fn umask(&self, args: &[String], env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            // Print current umask
            println!("0022");
            return 0;
        }
        
        let mask_str = &args[0];
        println!("umask: setting mask to {}", mask_str);
        env.set_var("UMASK".to_string(), mask_str.clone());
        0
    }
    
    /// ulimit - control resource limits
    fn ulimit(&self, args: &[String], _env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            // Print all limits
            println!("ulimit: unlimited");
            return 0;
        }
        
        println!("ulimit: setting limits: {:?}", args);
        // TODO: Implement actual resource limits
        0
    }
    
    /// printf - format and print data
    fn printf(&self, args: &[String], _env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            eprintln!("printf: format string required");
            return 1;
        }
        
        let format = &args[0];
        let values = &args[1..];
        
        // Simple printf implementation
        let mut result = String::new();
        let mut format_chars = format.chars().peekable();
        let mut value_idx = 0;
        
        while let Some(c) = format_chars.next() {
            if c == '%' {
                if let Some(next) = format_chars.peek() {
                    match next {
                        's' => {
                            format_chars.next(); // Consume 's'
                            if value_idx < values.len() {
                                result.push_str(&values[value_idx]);
                                value_idx += 1;
                            } else {
                                result.push_str("(null)");
                            }
                        }
                        'd' | 'i' => {
                            format_chars.next(); // Consume format char
                            if value_idx < values.len() {
                                let val = values[value_idx].parse::<i32>().unwrap_or(0);
                                result.push_str(&val.to_string());
                                value_idx += 1;
                            } else {
                                result.push_str("0");
                            }
                        }
                        '%' => {
                            format_chars.next(); // Consume second '%'
                            result.push('%');
                        }
                        _ => {
                            // Unknown format specifier, just include it
                            result.push(c);
                        }
                    }
                } else {
                    result.push(c);
                }
            } else if c == '\\' {
                // Handle escape sequences
                if let Some(next) = format_chars.next() {
                    match next {
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        'r' => result.push('\r'),
                        '\\' => result.push('\\'),
                        _ => {
                            result.push('\\');
                            result.push(next);
                        }
                    }
                } else {
                    result.push('\\');
                }
            } else {
                result.push(c);
            }
        }
        
        print!("{}", result);
        io::stdout().flush().unwrap();
        0
    }
}