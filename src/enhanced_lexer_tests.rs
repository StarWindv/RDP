//! 测试增强的Lexer功能

#[cfg(test)]
mod enhanced_lexer_tests {
    use crate::enhanced_lexer::EnhancedLexer;
    use crate::tokens::TokenType;
    
    #[test]
    fn test_basic_tokens() {
        let input = "echo hello world";
        let lexer = EnhancedLexer::new(input);
        let tokens: Vec<_> = lexer.collect();
        
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0].token_type, TokenType::Word(_)));
        assert!(matches!(tokens[1].token_type, TokenType::Word(_)));
        assert!(matches!(tokens[2].token_type, TokenType::Word(_)));
    }
    
    #[test]
    fn test_arithmetic_expansion() {
        let input = "echo $((1 + 2 * 3))";
        let lexer = EnhancedLexer::new(input);
        let tokens: Vec<_> = lexer.collect();
        
        // Should have: echo, DollarDLeftParen, Number(1), ArithmeticPlus, Number(2), ArithmeticStar, Number(3), DRightParen
        assert!(tokens.len() >= 8);
        
        // Check for arithmetic expansion start
        assert!(matches!(tokens[1].token_type, TokenType::DollarDLeftParen));
        
        // Check for arithmetic operators
        // The exact tokens depend on implementation
    }
    
    #[test]
    fn test_pipe_and_redirect() {
        let input = "ls | grep test > output.txt";
        let lexer = EnhancedLexer::new(input);
        let tokens: Vec<_> = lexer.collect();
        
        // Should have: ls, Pipe, grep, test, Great, output.txt
        assert!(tokens.len() >= 6);
        
        // Check for pipe operator
        let has_pipe = tokens.iter().any(|t| matches!(t.token_type, TokenType::Pipe));
        assert!(has_pipe, "Should have pipe operator");
        
        // Check for redirect operator
        let has_great = tokens.iter().any(|t| matches!(t.token_type, TokenType::Great));
        assert!(has_great, "Should have redirect operator");
    }
    
    #[test]
    fn test_variable_assignment() {
        let input = "VAR=value echo test";
        let lexer = EnhancedLexer::new(input);
        let tokens: Vec<_> = lexer.collect();
        
        // Should have: AssignmentWord, echo, test
        assert!(tokens.len() >= 3);
        
        // Check for assignment word
        if let TokenType::AssignmentWord(ref val) = tokens[0].token_type {
            assert_eq!(val, "VAR=value");
        } else {
            panic!("First token should be AssignmentWord");
        }
    }
    
    #[test]
    fn test_quoted_strings() {
        let input = "echo 'hello world' \"test string\"";
        let lexer = EnhancedLexer::new(input);
        let tokens: Vec<_> = lexer.collect();
        
        assert_eq!(tokens.len(), 3);
        
        // Check quoted strings
        if let TokenType::Word(ref val) = tokens[1].token_type {
            assert!(val.starts_with('\'') && val.ends_with('\''), "Should be single quoted");
        }
        
        if let TokenType::Word(ref val) = tokens[2].token_type {
            assert!(val.starts_with('"') && val.ends_with('"'), "Should be double quoted");
        }
    }
}