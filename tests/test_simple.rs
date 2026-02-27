/// Simple tests to verify current functionality

#[cfg(test)]
mod simple_tests {
    use rs_dash_pro::modules::lexer::Lexer;
    use rs_dash_pro::modules::parser::Parser;
    use rs_dash_pro::modules::ssa_ir_generator::SsaIrGenerator;
    use rs_dash_pro::modules::ssa_executor::SsaExecutor;

    fn execute(input: &str) -> i32 {
        // Lexical analysis
        let lexer = Lexer::new(input);
        let _tokens: Vec<_> = lexer.collect();
        
        // Parsing
        let mut parser = Parser::new(input);
        let ast = match parser.parse() {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("Parse error: {}", e);
                return 1;
            }
        };
        
        // SSA IR generation
        let mut generator = SsaIrGenerator::new();
        let ssa_func = generator.generate(ast);
        
        // Execution
        let mut executor = SsaExecutor::new();
        executor.execute_function(&ssa_func)
    }

    #[test]
    fn test_simple_echo() {
        let input = "echo hello";
        let status = execute(input);
        assert_eq!(status, 0, "simple echo should succeed");
    }

    #[test]
    fn test_true_command() {
        let input = "true";
        let status = execute(input);
        assert_eq!(status, 0, "true should return 0");
    }

    #[test]
    fn test_false_command() {
        let input = "false";
        let status = execute(input);
        assert_eq!(status, 1, "false should return 1");
    }
}
