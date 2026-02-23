//! rs-dash-pro - A POSIX-compatible shell with modern architecture

use std::env as std_env;
use std::io::{self, Write};
use std::process;

mod tokens;
mod lexer;
mod ast;
mod parser;
mod ir;
mod ir_generator;
mod optimizer;
mod executor;
mod builtins;
mod env;

use lexer::Lexer;
use parser::Parser;
use ir_generator::IrGenerator;
use optimizer::Optimizer;
use executor::Executor;

/// Main function
fn main() {
    // Get command line arguments
    let args: Vec<String> = std_env::args().collect();
    
    if args.len() > 1 {
        if args[1] == "-c" && args.len() > 2 {
            // Execute command string
            let exit_status = execute_command_string(&args[2]);
            process::exit(exit_status);
        } else if args[1] == "--help" || args[1] == "-h" {
            show_help();
            process::exit(0);
        } else if args[1] == "--version" || args[1] == "-v" {
            show_version();
            process::exit(0);
        } else {
            // Assume it's a script file
            let exit_status = execute_script_file(&args[1]);
            process::exit(exit_status);
        }
    }
    
    // Interactive mode
    run_interactive();
}

/// Execute a command string
fn execute_command_string(cmd_str: &str) -> i32 {
    match parse_and_execute(cmd_str) {
        Ok(exit_status) => exit_status,
        Err(e) => {
            eprintln!("Error: {}", e);
            1
        }
    }
}

/// Execute a script file
fn execute_script_file(filename: &str) -> i32 {
    match std::fs::read_to_string(filename) {
        Ok(contents) => {
            let mut last_status = 0;
            
            for line in contents.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                
                match parse_and_execute(line) {
                    Ok(status) => last_status = status,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return 1;
                    }
                }
            }
            
            last_status
        }
        Err(e) => {
            eprintln!("Error reading script {}: {}", filename, e);
            1
        }
    }
}

/// Parse and execute a command line
fn parse_and_execute(input: &str) -> Result<i32, String> {
    // Lexical analysis
    let lexer = Lexer::new(input);
    let _tokens: Vec<_> = lexer.collect();
    
    // Debug: print tokens
    // for token in &tokens {
    //     println!("Token: {}", token);
    // }
    
    // Parsing
    let mut parser = Parser::new(input);
    let ast = parser.parse().map_err(|e| e.to_string())?;
    
    // Debug: print AST
    // println!("AST: {}", ast);
    
    // IR generation
    let generator = IrGenerator;
    let ir_program = generator.generate(ast);
    
    // Debug: print IR
    // println!("{}", ir_program);
    
    // Optimization
    let optimizer = Optimizer::new();
    let optimized_ir = optimizer.optimize(ir_program);
    
    // Execution
    let mut executor = Executor::new();
    let exit_status = executor.execute(&optimized_ir);
    
    Ok(exit_status)
}

/// Run interactive shell
fn run_interactive() {
    println!("rs-dash-pro v{}", env!("CARGO_PKG_VERSION"));
    println!("Type 'help' for help, 'exit' to quit");
    
    loop {
        // Print prompt
        print!("$ ");
        io::stdout().flush().unwrap();
        
        // Read input
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                // EOF
                println!();
                break;
            }
            Ok(_) => {
                let line = input.trim();
                if line.is_empty() {
                    continue;
                }
                
                if line == "exit" {
                    break;
                }
                
                match parse_and_execute(line) {
                    Ok(exit_status) => {
                        if exit_status != 0 {
                            eprintln!("Exit status: {}", exit_status);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
}

/// Show help message
fn show_help() {
    println!("rs-dash-pro - A POSIX-compatible shell");
    println!();
    println!("Usage:");
    println!("  rs-dash-pro                 Run interactive shell");
    println!("  rs-dash-pro -c COMMAND     Execute command string");
    println!("  rs-dash-pro SCRIPT         Execute script file");
    println!("  rs-dash-pro --help         Show this help");
    println!("  rs-dash-pro --version      Show version");
    println!();
    println!("Architecture: Lexer → Parser → IRGenerator → Optimizer → Executor");
}

/// Show version information
fn show_version() {
    println!("rs-dash-pro version {}", env!("CARGO_PKG_VERSION"));
    println!("A POSIX-compatible shell implementation in Rust");
    println!("Architecture: Lexer → Parser → IRGenerator → Optimizer → Executor");
}