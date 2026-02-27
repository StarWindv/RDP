// Test job control functionality

fn main() {
    println!("Testing job control functionality...");
    
    // Test 1: Simple command execution
    println!("\nTest 1: Simple command execution");
    
    use rs_dash_pro::modules::lexer::Lexer;
    use rs_dash_pro::modules::parser::Parser;
    use rs_dash_pro::modules::ssa_ir_generator::SsaIrGenerator;
    use rs_dash_pro::modules::ssa_executor::SsaExecutor;
    
    let input = "echo hello world";
    println!("Parsing: {}", input);
    
    let lexer = Lexer::new(input);
    let tokens: Vec<_> = lexer.collect();
    println!("Tokens: {}", tokens.len());
    
    let mut parser = Parser::new(input);
    match parser.parse() {
        Ok(ast) => {
            println!("AST generated successfully");
            
            // Generate SSA IR
            let mut generator = SsaIrGenerator::new();
            let func = generator.generate(ast);
            println!("SSA IR generated: {} blocks", func.blocks.len());
            
            // Execute
            let mut executor = SsaExecutor::new();
            let status = executor.execute_function(&func);
            println!("Exit status: {}", status);
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
        }
    }
    
    // Test 2: Background execution
    println!("\nTest 2: Background execution (simulated)");
    let input2 = "sleep 2 &";
    println!("Parsing: {}", input2);
    
    let mut parser2 = Parser::new(input2);
    match parser2.parse() {
        Ok(ast) => {
            println!("AST for background command generated");
            // Note: Background execution would generate Fork and Exec instructions
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
        }
    }
    
    println!("\nJob control test completed.");
}