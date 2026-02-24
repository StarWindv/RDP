//! Basic test for rs-dash-pro SSA architecture

use rs_dash_pro::*;

fn main() {
    println!("Testing rs-dash-pro SSA architecture...");
    
    // Test 1: Simple command
    println!("\n=== Test 1: Simple command ===");
    test_command("echo hello");
    
    // Test 2: Multiple commands
    println!("\n=== Test 2: Multiple commands ===");
    test_command("echo first; echo second");
    
    // Test 3: Pipes
    println!("\n=== Test 3: Pipes ===");
    test_command("echo test | wc -c");
    
    // Test 4: Variables
    println!("\n=== Test 4: Variables ===");
    test_command("VAR=hello; echo $VAR");
    
    // Test 5: Control structures
    println!("\n=== Test 5: Control structures ===");
    test_command("if true; then echo 'if works'; fi");
    
    println!("\nAll tests completed!");
}

fn test_command(input: &str) {
    println!("Testing command: {}", input);
    
    // Lexical analysis
    let lexer = EnhancedLexer::new(input);
    let tokens: Vec<_> = lexer.collect();
    println!("  Tokens: {}", tokens.len());
    
    // Parsing
    let mut parser = Parser::new(input);
    match parser.parse() {
        Ok(ast) => {
            println!("  AST: {}", ast);
            
            // SSA IR generation
            let mut generator = SsaIrGenerator::new();
            let ssa_func = generator.generate(ast);
            println!("  SSA IR generated");
            
            // SSA Execution
            let mut executor = SsaExecutor::new();
            let exit_status = executor.execute_function(&ssa_func);
            println!("  Exit status: {}", exit_status);
        }
        Err(e) => {
            println!("  Parse error: {}", e);
        }
    }
}