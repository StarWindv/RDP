use rewrite::rs_dash_pro::modules::lexer::Lexer;
use rewrite::rs_dash_pro::modules::parser::Parser;
use rewrite::rs_dash_pro::modules::ssa_ir_generator::SsaIrGenerator;
use rewrite::rs_dash_pro::modules::ssa_executor::SsaExecutor;

fn main() {
    // Test simple echo command
    let input = "echo hello";
    println!("Testing: {}", input);
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();
    
    let mut generator = SsaIrGenerator::new();
    let func = generator.generate(ast);
    
    let mut executor = SsaExecutor::new();
    let result = executor.execute_function(&func);
    
    println!("Exit status: {}", result);
    
    // Test pipeline
    let input2 = "echo hello | cat";
    println!("\nTesting: {}", input2);
    
    let mut lexer2 = Lexer::new(input2);
    let tokens2 = lexer2.tokenize().unwrap();
    
    let mut parser2 = Parser::new(tokens2);
    let ast2 = parser2.parse().unwrap();
    
    let mut generator2 = SsaIrGenerator::new();
    let func2 = generator2.generate(ast2);
    
    let mut executor2 = SsaExecutor::new();
    let result2 = executor2.execute_function(&func2);
    
    println!("Exit status: {}", result2);
}