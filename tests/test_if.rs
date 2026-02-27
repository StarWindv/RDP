/// Test if statement parsing and execution

#[cfg(test)]
mod if_tests {
    use rs_dash_pro::modules::lexer::Lexer;
    use rs_dash_pro::modules::parser::Parser;

    #[test]
    fn test_parse_if_statement() {
        let input = "if true; then echo yes; fi";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.collect();
        
        println!("Tokens:");
        for token in &tokens {
            println!("  {:?}", token.token_type);
        }
        
        let mut parser = Parser::new(input);
        match parser.parse() {
            Ok(ast) => {
                println!("AST: {}", ast);
                println!("AST debug: {:?}", ast);
            }
            Err(e) => {
                panic!("Parse error: {}", e);
            }
        }
    }
}
