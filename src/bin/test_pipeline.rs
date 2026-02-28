use rs_dash_pro::modules::lexer::Lexer;
use rs_dash_pro::modules::parser::Parser;
use rs_dash_pro::modules::ssa_ir_generator::SsaIrGenerator;
use rs_dash_pro::modules::ssa_executor::SsaExecutor;

fn main() {
    println!("=== Testing simple echo ===");
    test_command("echo hello");
    
    println!("\n=== Testing pipeline ===");
    test_command("echo hello | cat");
    
    println!("\n=== Testing multi-pipeline ===");
    test_command("echo test1 | cat | grep test");
    
    println!("\n=== Testing output redirection ===");
    test_command("echo test > test_output.txt");
    
    println!("\n=== Testing input redirection ===");
    test_command("cat < test_output.txt");
    
    println!("\n=== Testing append redirection ===");
    test_command("echo line2 >> test_output.txt");
    
    println!("\n=== Testing combined redirection ===");
    test_command("echo both > combined.txt 2>&1");
}

fn test_command(input: &str) {
    println!("Testing command: {}", input);
    
    let mut lexer = Lexer::new(input);
    let tokens: Vec<_> = lexer.collect();
    println!("  Tokens: {:?}", tokens.iter().map(|t| format!("{:?}", t.token_type)).collect::<Vec<_>>());
    
    let mut parser = Parser::new(input);
    match parser.parse() {
        Ok(ast) => {
            println!("  AST parsed successfully");
            
            let mut generator = SsaIrGenerator::new();
            let func = generator.generate(ast);
            println!("  SSA IR generated");
            
            let mut executor = SsaExecutor::new();
            let result = executor.execute_function(&func);
            println!("  Exit status: {}", result);
        }
        Err(e) => {
            println!("  Parse error: {}", e.message);
        }
    }
}