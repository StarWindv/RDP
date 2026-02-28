use rs_dash_pro::modules::lexer::Lexer;
use rs_dash_pro::modules::parser::Parser;
use rs_dash_pro::modules::ssa_ir_generator::SsaIrGenerator;
use rs_dash_pro::modules::ssa_executor::SsaExecutor;

#[test]
fn test_simple_echo() {
    let input = "echo hello";
    println!("Testing: {}", input);
    
    let mut lexer = Lexer::new(input);
    let tokens: Vec<_> = lexer.collect();
    println!("Tokens: {:?}", tokens);
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    println!("AST: {:?}", ast);
    
    let mut generator = SsaIrGenerator::new();
    let func = generator.generate(ast);
    println!("SSA IR function generated");
    
    let mut executor = SsaExecutor::new();
    let result = executor.execute_function(&func);
    println!("Exit status: {}", result);
    
    assert_eq!(result, 0);
}

#[test]
fn test_pipeline() {
    let input = "echo hello | cat";
    println!("\nTesting pipeline: {}", input);
    
    let mut lexer = Lexer::new(input);
    let tokens: Vec<_> = lexer.collect();
    println!("Tokens: {:?}", tokens);
    
    let mut parser = Parser::new(input);
    match parser.parse() {
        Ok(ast) => {
            println!("AST: {:?}", ast);
            
            let mut generator = SsaIrGenerator::new();
            let func = generator.generate(ast);
            println!("SSA IR function generated");
            
            let mut executor = SsaExecutor::new();
            let result = executor.execute_function(&func);
            println!("Exit status: {}", result);
            
            assert_eq!(result, 0);
        }
        Err(e) => {
            println!("Parse error: {}", e.message);
            panic!("Parse failed");
        }
    }
}