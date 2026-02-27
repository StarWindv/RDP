use std::env as std_env;
use std::io::{self, Write};
use std::process;

use rs_dash_pro::modules::lexer::Lexer;
use rs_dash_pro::modules::parser::Parser;
use rs_dash_pro::modules::ssa_executor::SsaExecutor;
use rs_dash_pro::modules::ssa_ir_generator::SsaIrGenerator;

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
    match parse_and_execute_ssa(cmd_str) {
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
            // Parse and execute the entire file as one program (not line by line)
            match parse_and_execute_ssa(&contents) {
                Ok(status) => status,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    1
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading script {}: {}", filename, e);
            1
        }
    }
}

/// Parse and execute a command line using SSA architecture
fn parse_and_execute_ssa(input: &str) -> Result<i32, String> {
    println!("DEBUG: Parsing input: {}", input);

    // Lexical analysis with lexer
    let lexer = Lexer::new(input);
    let tokens: Vec<_> = lexer.collect();

    println!("DEBUG: Lexer produced {} tokens", tokens.len());
    for (i, token) in tokens.iter().enumerate() {
        println!("DEBUG TOKEN {}: {:?}", i, token.token_type);
    }

    // Parsing
    let mut parser = Parser::new(input);
    let ast = parser.parse().map_err(|e| e.to_string())?;

    println!("DEBUG: Parser produced AST: {}", ast);

    // SSA IR generation
    let mut generator = SsaIrGenerator::new();
    let ssa_func = generator.generate(ast);

    println!("DEBUG: SSA IR generated:\n{}", ssa_func);

    // TODO: Optimization (NOP for now)

    // SSA Execution
    let mut executor = SsaExecutor::new();
    let exit_status = executor.execute_function(&ssa_func);

    println!("DEBUG: Execution completed with status: {}", exit_status);

    Ok(exit_status)
}

/// Run interactive shell
fn run_interactive() {
    println!(
        "rs-dash-pro v{} (Enhanced SSA Architecture)",
        env!("CARGO_PKG_VERSION")
    );
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

                match parse_and_execute_ssa(line) {
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
    println!("rs-dash-pro - A POSIX-compatible shell with Enhanced SSA architecture");
    println!();
    println!("Usage:");
    println!("  rs-dash-pro                 Run interactive shell");
    println!("  rs-dash-pro -c COMMAND     Execute command string");
    println!("  rs-dash-pro SCRIPT         Execute script file");
    println!("  rs-dash-pro --help         Show this help");
    println!("  rs-dash-pro --version      Show version");
    println!();
    println!("Architecture: Lexer → Parser → SSA IR Generator → Optimizer → SSA Executor");
    println!("SSA: Static Single Assignment form for better optimization");
    println!("Enhanced Lexer: Full POSIX Shell tokenization support");
}

/// Show version information
fn show_version() {
    println!("rs-dash-pro version {}", env!("CARGO_PKG_VERSION"));
    println!("A POSIX-compatible shell implementation in Rust");
    println!("Architecture: Lexer → Parser → SSA IR Generator → Optimizer → SSA Executor");
    println!("SSA: Static Single Assignment form for better optimization");
    println!("Enhanced Lexer: Full POSIX Shell tokenization support");
}
