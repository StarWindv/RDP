// Simple test for pipeline functionality
use rs_dash_pro::modules::parser::Parser;
use rs_dash_pro::modules::ssa_ir_generator::SsaIrGenerator;
use rs_dash_pro::modules::ssa_executor::SsaExecutor;

fn main() {
    println!("Testing simple pipeline...");
    
    // Test: echo hello
    let input = "echo hello";
    let mut parser = Parser::new(input);
    match parser.parse() {
        Ok(ast) => {
            println!("AST parsed successfully: {:?}", ast);
            
            let mut generator = SsaIrGenerator::new();
            let func = generator.generate(ast);
            println!("Generated SSA IR function");
            
            let mut executor = SsaExecutor::new();
            let result = executor.execute_function(&func);
            println!("Execution result: {}", result);
        }
        Err(e) => {
            println!("Parse error: {}", e);
        }
    }
}